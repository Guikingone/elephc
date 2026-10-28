/* Real PHP embed SAPI: repeated requests, with overlapping ZTS requests on eight threads. */
#include <sapi/embed/php_embed.h>
#include <Zend/zend_exceptions.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>

#if defined(__SANITIZE_ADDRESS__)
#define EP_SANITIZED_ALLOCATOR 1
#elif defined(__clang__)
#if __has_feature(address_sanitizer)
#define EP_SANITIZED_ALLOCATOR 1
#endif
#endif

#if defined(EP_SANITIZED_ALLOCATOR)
#include <sanitizer/allocator_interface.h>
#elif defined(__GLIBC__)
#include <malloc.h>
#elif defined(__APPLE__)
#include <malloc/malloc.h>
#else
#error "The PHP lifecycle test requires live malloc statistics on this platform"
#endif

#if defined(__x86_64__) && defined(__linux__)
/* PHP's supplied Valgrind replacement targets the CLI's NONE soname. Embed
 * libphp.so needs the same bounded comparison registered for its own soname.
 * Keep all Memcheck diagnostics enabled, including extension memory accesses.
 * See php-src/Zend/zend_string.c, I_REPLACE_SONAME_FNNAME_ZU. */
bool ZEND_FASTCALL _vgr00000ZU_libphpZdso_zend_string_equal_val(const zend_string *a, const zend_string *b)
{
    return memcmp(ZSTR_VAL(a), ZSTR_VAL(b), ZSTR_LEN(a)) == 0;
}
#endif

static atomic_int failures;
static atomic_uint request_ids;

/* Reachable allocations also count: PHP can retain a fatal Fiber's stack mapping. */
static size_t live_malloc_bytes(void)
{
#if defined(EP_SANITIZED_ALLOCATOR)
    return __sanitizer_get_current_allocated_bytes();
#elif defined(__GLIBC__)
    struct mallinfo2 stats = mallinfo2();
    return stats.uordblks + stats.hblkhd;
#else
    malloc_statistics_t stats;
    malloc_zone_statistics(NULL, &stats);
    return stats.size_in_use;
#endif
}

/* A shutdown-flushed output callback can park a Fiber after the destructor pass. */
static void run_late_userland_fiber_request(void)
{
    static const char code[] =
        "ob_start(function ($bytes) {"
        "  if ($bytes === '') { return ''; }"
        "  $GLOBALS['late_fiber'] = new Fiber(function () {"
        "    set_error_handler(function () { Fiber::suspend('parked'); return true; });"
        "    try { phpext_pause('late userland shutdown'); } catch (RuntimeException $error) {}"
        "  });"
        "  if ($GLOBALS['late_fiber']->start() !== 'parked') { throw new Exception('late Fiber did not park'); }"
        "  return '';"
        "}); echo 'late';";
    if (php_request_startup() == FAILURE) { atomic_fetch_add(&failures, 1); return; }
    if (zend_eval_string(code, NULL, "phpext late userland Fiber") == FAILURE || EG(exception)) {
        atomic_fetch_add(&failures, 1);
    }
    php_request_shutdown(NULL);
}

/* A fatal on the main context abandons one or two parked native activations. */
static void run_abandoned_requests(void)
{
    static const char code[] =
        "error_reporting(0); ini_set('display_errors', '0'); ini_set('log_errors', '0');"
        "$GLOBALS['fatal_fiber'] = new Fiber(function () use ($nested) {"
        "  set_error_handler(function () use ($nested) {"
        "    if ($nested) {"
        "      set_error_handler(function () { Fiber::suspend('parked'); return true; });"
        "      try { phpext_pause('nested abandoned'); } catch (RuntimeException $error) {}"
        "    } else { Fiber::suspend('parked'); } return true;"
        "  });"
        "  try { phpext_pause('abandoned'); } catch (RuntimeException $error) {}"
        "});"
        "if ($GLOBALS['fatal_fiber']->start() !== 'parked') { throw new Exception('Fiber did not park'); }"
        "if ($held) { elephc_late_hold($GLOBALS['fatal_fiber']); }"
        "set_error_handler(static function () { return false; });"
        "trigger_error('intentional fatal request', E_USER_ERROR);";
    php_request_shutdown(NULL);
    run_late_userland_fiber_request();
    size_t baseline = live_malloc_bytes();
    for (unsigned request = 0; request < 8; request++) {
        if (php_request_startup() == FAILURE) { atomic_fetch_add(&failures, 1); break; }
        unsigned expected_resumes = (request / 4) * 2 + (request % 4 < 2 ? request % 4 : 2);
        char script[sizeof(code) + 160];
        snprintf(script, sizeof(script),
            "if (elephc_late_resume_count() !== %u) { throw new Exception('late resume missing'); } "
            "$nested = %s; $held = %s; %s", expected_resumes,
            request % 2 ? "true" : "false", request % 4 < 2 ? "true" : "false", code);
        volatile bool bailed = false;
        zend_first_try {
            (void)zend_eval_string(script, NULL, "phpext abandoned request");
        } zend_catch {
            bailed = true;
        } zend_end_try();
        if (!bailed) { atomic_fetch_add(&failures, 1); }
        php_request_shutdown(NULL);
        size_t current = live_malloc_bytes();
        /* The fixture uses a 512 KiB native heap; abandoned images grow well past 4 MiB. */
        if (current > baseline && current - baseline > 4 * 1024 * 1024) {
            fprintf(stderr, "abandoned native calls retained %zu malloc bytes after %u requests\n",
                current - baseline, request + 1);
            atomic_fetch_add(&failures, 1);
            break;
        }
    }
    if (php_request_startup() == FAILURE) {
        atomic_fetch_add(&failures, 1);
    } else if (zend_eval_string("if (elephc_late_resume_count() !== 4) { throw new Exception('final late resume missing'); }",
        NULL, "phpext final late resume") == FAILURE || EG(exception)) {
        atomic_fetch_add(&failures, 1);
    }
}

static const char request_code[] =
    "ob_start(function ($bytes) use ($token) {"
    "  if ($bytes === '') { return ''; }"
    "  if ($bytes !== $token || phpext_mirror([$token]) !== [$token]) { throw new Exception('output context corrupted'); }"
    "  return '';"
    "}, 1);"
    "for ($i = 1, $limit = (int)(getenv('ELEPHC_EMBED_ITERATIONS') ?: 24); $i <= $limit; $i++) {"
    "  if (phpext_counter() !== $i) { throw new Exception('request static leaked'); }"
    "  $value = ['binary' => \"a\\0b\", 7 => [null, true, 1.5]];"
    "  if (phpext_mirror($value) !== $value) { throw new Exception('array corrupted'); }"
    "  if (phpext_label(\"x\\0y\") !== \"label:x\\0y\") { throw new Exception('string corrupted'); }"
    "  if (phpext_output($token) !== $token) { throw new Exception('output result corrupted'); }"
    "  try { phpext_failure(); throw new Exception('missing native exception'); }"
    "  catch (RuntimeException $e) { if ($e->getCode() !== 42) { throw new Exception('wrong exception'); } }"
    "} ob_end_clean();"
    "try { phpext_recurse(100000); throw new Exception('missing native stack guard'); }"
    "catch (Error $e) { if (!str_contains($e->getMessage(), 'runtime failed')) { throw $e; } }"
    "if (phpext_counter() !== $limit + 1) { throw new Exception('failed stack recovery'); }"
    "require getenv('ELEPHC_EMBED_FIBERS');";

/* Each thread owns Zend globals and four complete request lifecycles. */
static void *run_requests(void *unused)
{
    (void)unused;
#ifdef ZTS
    ts_resource(0);
    ZEND_TSRMLS_CACHE_UPDATE();
#endif
    for (unsigned request = 0; request < 4; request++) {
        zend_first_try {
            if (php_request_startup() == FAILURE) {
                atomic_fetch_add(&failures, 1);
                break;
            }
            char code[sizeof(request_code) + 80];
            snprintf(code, sizeof(code), "$token = \"request:%u\\0bytes\"; %s",
                atomic_fetch_add(&request_ids, 1), request_code);
            if (zend_eval_string(code, NULL, "phpext request isolation") == FAILURE || EG(exception)) {
                atomic_fetch_add(&failures, 1);
                if (EG(exception)) { zend_exception_error(EG(exception), E_WARNING); }
            }
            php_request_shutdown(NULL);
        } zend_catch {
            atomic_fetch_add(&failures, 1);
        } zend_end_try();
    }
#ifdef ZTS
    ts_free_thread();
#endif
    return NULL;
}

/* The process initializes PHP once, then tests real RINIT/RSHUTDOWN and final MSHUTDOWN. */
int main(int argc, char **argv)
{
    if (argc != 2) { fprintf(stderr, "usage: embed-test path/to/php.ini\n"); return 2; }
    php_embed_module.php_ini_path_override = argv[1];
    php_embed_module.php_ini_ignore_cwd = 1;
    if (php_embed_init(1, argv) == FAILURE) { return 3; }
#ifdef ZTS
    pthread_t threads[8];
    pthread_attr_t attributes;
    if (pthread_attr_init(&attributes) || pthread_attr_setstacksize(&attributes, 512 * 1024)) { return 5; }
    unsigned started = 0;
    for (; started < 8; started++) {
        if (pthread_create(&threads[started], &attributes, run_requests, NULL)) {
            atomic_fetch_add(&failures, 1);
            break;
        }
    }
    for (unsigned i = 0; i < started; i++) { pthread_join(threads[i], NULL); }
    pthread_attr_destroy(&attributes);
#else
    php_request_shutdown(NULL);
    run_requests(NULL);
    if (php_request_startup() == FAILURE) { return 4; }
#endif
    run_abandoned_requests();
    php_embed_shutdown();
    if (atomic_load(&failures)) {
        fprintf(stderr, "PHP extension request failures: %d\n", atomic_load(&failures));
        return 1;
    }
    puts("request isolation ok");
    fflush(stdout);
    /* Let Apple's leaks inspect the process after all module/thread/request teardown. */
    if (getenv("ELEPHC_EMBED_LEAK_READY")) { (void)getchar(); }
    return 0;
}
