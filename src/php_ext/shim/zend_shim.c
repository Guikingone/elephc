/* zend_shim.c — a minimal stand-in for the Zend engine, enough to run a real
 * compiled PECL extension (simdjson) with NO PHP present.
 *
 * This is a feasibility prototype for the question: can Elephc host real PHP
 * extensions by supplying a shim, instead of reimplementing each extension?
 *
 * Scope, stated honestly:
 *   - Implemented for real: the allocator, zend_string, and the HashTable
 *     write-side that simdjson actually drives while materialising parsed JSON.
 *   - Stubbed with a hard abort(): everything on paths this prototype does not
 *     exercise (arg parsing, class registration, object model). They must be
 *     linked because php_simdjson.o references them, but reaching one is a bug,
 *     not a silent degradation — hence abort() rather than a quiet no-op.
 *
 * Arrays are always built in HASH mode (never PHP 8.2+ packed layout), so a
 * single Bucket walk reads everything back. That is a deliberate prototype
 * simplification, and it is why the reader below does not use ZEND_HASH_FOREACH.
 */

#include "php.h"
#include "zend_exceptions.h"
#include "zend_interfaces.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* zend_alloc.h turns these into macros that dispatch to size-binned variants
 * (_emalloc_16, _emalloc_24, …). We are *defining* the real symbols, so the
 * macros have to go first. */
#undef _emalloc
#undef _efree
#undef _ecalloc
#undef _erealloc

#define SHIM_UNIMPLEMENTED(name)                                               \
    do {                                                                       \
        fprintf(stderr, "shim: '%s' reached — not implemented in this "        \
                        "prototype (this is a bug, not a no-op)\n", (name));   \
        abort();                                                               \
    } while (0)

/* ---------------------------------------------------------------- globals -- */

ZEND_API zend_string *zend_empty_string = NULL;
ZEND_API zend_string *zend_one_char_string[256];
ZEND_API const HashTable zend_empty_array;
const ZEND_API zend_object_handlers std_object_handlers;
ZEND_API zend_class_entry *zend_ce_value_error = NULL;
ZEND_API zend_class_entry *spl_ce_RuntimeException = NULL;

/* Counters, so the driver can prove the extension really drove the shim. */
size_t shim_allocs = 0, shim_frees = 0, shim_strings = 0, shim_arrays = 0;

/* Real PHP reads the current frame from EG(current_execute_data). We have no
 * executor globals, so the caller publishes the frame here instead — this is
 * exactly the seam where Elephc would hand its own call frame to an extension. */
zend_execute_data *shim_current_execute_data = NULL;

/* -------------------------------------------------------------- allocator -- */

/* Request memory routes through the arena (zend_shim_arena.c): reclaimed
 * wholesale at the request boundary, which is what makes a bailout survivable
 * — skipped frees and skipped C++ destructors then cost nothing. */
extern void *shim_alloc(size_t size, int persistent);
extern void shim_free(void *ptr);
extern void *shim_realloc(void *ptr, size_t size);

ZEND_API void *ZEND_FASTCALL _emalloc(size_t size) {
    shim_allocs++;
    return shim_alloc(size, 0);
}
ZEND_API void ZEND_FASTCALL _efree(void *ptr) { if (ptr) { shim_frees++; shim_free(ptr); } }
ZEND_API void *ZEND_FASTCALL _ecalloc(size_t n, size_t l) {
    size_t total = n * l; void *p = _emalloc(total); memset(p, 0, total); return p;
}
ZEND_API void *ZEND_FASTCALL _erealloc(void *ptr, size_t size) { return shim_realloc(ptr, size); }

/* ------------------------------------------------------------ zend_string -- */

static zend_string *shim_string_alloc_ex(const char *str, size_t len, bool interned) {
    zend_string *s = (zend_string *)_emalloc(_ZSTR_STRUCT_SIZE(len));
    GC_SET_REFCOUNT(s, 1);
    GC_TYPE_INFO(s) = interned ? (GC_STRING | (IS_STR_INTERNED << GC_FLAGS_SHIFT)) : GC_STRING;
    ZSTR_H(s) = 0;
    ZSTR_LEN(s) = len;
    if (len) memcpy(ZSTR_VAL(s), str, len);
    ZSTR_VAL(s)[len] = '\0';
    shim_strings++;
    return s;
}
static zend_string *shim_string_alloc(const char *str, size_t len) {
    return shim_string_alloc_ex(str, len, false);
}

static zend_string *ZEND_FASTCALL shim_init_interned(const char *str, size_t size, bool permanent) {
    (void)permanent;
    return shim_string_alloc_ex(str, size, true);
}
ZEND_API zend_string_init_interned_func_t zend_string_init_interned = shim_init_interned;

/* ------------------------------------------------------------- hashtable --- */
/* Hash mode only: arData is a Bucket array; lookup is linear. Correctness over
 * speed — this prototype answers "does it run", not "is it fast". */

static void shim_ht_init(HashTable *ht, uint32_t size) {
    if (size < 8) size = 8;
    memset(ht, 0, sizeof(*ht));
    HT_FLAGS(ht) = HASH_FLAG_STATIC_KEYS;
    ht->nTableSize = size;
    ht->nTableMask = (uint32_t)-(int32_t)size;
    ht->nNumUsed = 0;
    ht->nNumOfElements = 0;
    ht->nNextFreeElement = 0;
    ht->arData = (Bucket *)_ecalloc(size, sizeof(Bucket));
    ht->pDestructor = NULL;
    shim_arrays++;
}

static void shim_ht_grow(HashTable *ht) {
    if (ht->nNumUsed < ht->nTableSize) return;
    uint32_t ns = ht->nTableSize * 2;
    Bucket *nd = (Bucket *)_ecalloc(ns, sizeof(Bucket));
    memcpy(nd, ht->arData, ht->nNumUsed * sizeof(Bucket));
    _efree(ht->arData);
    ht->arData = nd;
    ht->nTableSize = ns;
}

ZEND_API HashTable *ZEND_FASTCALL _zend_new_array_0(void) {
    HashTable *ht = (HashTable *)_emalloc(sizeof(HashTable));
    shim_ht_init(ht, 8);
    return ht;
}
ZEND_API HashTable *ZEND_FASTCALL _zend_new_array(uint32_t size) {
    HashTable *ht = (HashTable *)_emalloc(sizeof(HashTable));
    shim_ht_init(ht, size);
    return ht;
}

int shim_trace = 0; /* set from the driver to watch what the extension drives */

static zval *shim_ht_append(HashTable *ht, zend_string *key, zend_ulong h, zval *v) {
    if (shim_trace) {
        fprintf(stderr, "  [shim] append ht=%p key=%s h=%llu type=%d used=%u\n",
                (void *)ht, key ? ZSTR_VAL(key) : "(int)", (unsigned long long)h,
                (int)Z_TYPE_P(v), ht->nNumUsed);
    }
    shim_ht_grow(ht);
    Bucket *b = ht->arData + ht->nNumUsed;
    /* The real zend_hash_update takes a reference on the key: callers such as
     * simdjson do `zend_string_init` → update → `zend_string_release_ex`, so
     * without this addref the key is freed out from under the table. */
    if (key && !ZSTR_IS_INTERNED(key)) GC_ADDREF(key);
    b->key = key;
    b->h = h;
    ZVAL_COPY_VALUE(&b->val, v);
    ht->nNumUsed++;
    ht->nNumOfElements++;
    if (!key && (zend_long)h >= ht->nNextFreeElement) ht->nNextFreeElement = (zend_long)h + 1;
    return &b->val;
}

ZEND_API zval *ZEND_FASTCALL zend_hash_update(HashTable *ht, zend_string *key, zval *pData) {
    for (uint32_t i = 0; i < ht->nNumUsed; i++) {
        Bucket *b = ht->arData + i;
        if (b->key && ZSTR_LEN(b->key) == ZSTR_LEN(key)
            && memcmp(ZSTR_VAL(b->key), ZSTR_VAL(key), ZSTR_LEN(key)) == 0) {
            ZVAL_COPY_VALUE(&b->val, pData);
            return &b->val;
        }
    }
    return shim_ht_append(ht, key, 0, pData);
}
ZEND_API zval *ZEND_FASTCALL zend_hash_index_update(HashTable *ht, zend_ulong h, zval *pData) {
    for (uint32_t i = 0; i < ht->nNumUsed; i++) {
        Bucket *b = ht->arData + i;
        if (!b->key && b->h == h) { ZVAL_COPY_VALUE(&b->val, pData); return &b->val; }
    }
    return shim_ht_append(ht, NULL, h, pData);
}
ZEND_API zval *ZEND_FASTCALL zend_hash_next_index_insert(HashTable *ht, zval *pData) {
    return shim_ht_append(ht, NULL, (zend_ulong)ht->nNextFreeElement, pData);
}
ZEND_API zval *ZEND_FASTCALL zend_hash_find(const HashTable *ht, zend_string *key) {
    for (uint32_t i = 0; i < ht->nNumUsed; i++) {
        Bucket *b = ht->arData + i;
        if (b->key && ZSTR_LEN(b->key) == ZSTR_LEN(key)
            && memcmp(ZSTR_VAL(b->key), ZSTR_VAL(key), ZSTR_LEN(key)) == 0) {
            return &b->val;
        }
    }
    return NULL;
}
ZEND_API bool ZEND_FASTCALL _zend_handle_numeric_str_ex(const char *key, size_t length, zend_ulong *idx) {
    if (!length || length > 19) return 0;
    for (size_t i = 0; i < length; i++) if (key[i] < '0' || key[i] > '9') return 0;
    if (length > 1 && key[0] == '0') return 0;
    *idx = strtoull(key, NULL, 10);
    return 1;
}

/* -------------------------------------------------------------- zval dtor -- */

static void shim_free_ht(HashTable *ht);
static void shim_free_object(zend_object *obj);

ZEND_API void ZEND_FASTCALL rc_dtor_func(zend_refcounted *p) {
    zend_uchar t = GC_TYPE(p);
    if (t == IS_STRING) { if (GC_DELREF(p) == 0) _efree(p); return; }
    if (t == IS_ARRAY)  { shim_free_ht((HashTable *)p); return; }
    if (t == IS_OBJECT) { shim_free_object((zend_object *)p); return; }
}
static void shim_release_string(zend_string *s) {
    if (!s || ZSTR_IS_INTERNED(s)) return;   /* interned strings are never freed */
    if (GC_DELREF(s) == 0) _efree(s);
}
static void shim_zval_dtor(zval *z) {
    if (!Z_REFCOUNTED_P(z)) return;          /* interned strings, immutable arrays */
    if (Z_TYPE_P(z) == IS_STRING) { shim_release_string(Z_STR_P(z)); }
    else if (Z_TYPE_P(z) == IS_ARRAY) { shim_free_ht(Z_ARRVAL_P(z)); }
    else if (Z_TYPE_P(z) == IS_OBJECT) { shim_free_object(Z_OBJ_P(z)); }
}
static void shim_free_ht(HashTable *ht) {
    if (ht == &zend_empty_array) return;
    for (uint32_t i = 0; i < ht->nNumUsed; i++) {
        Bucket *b = ht->arData + i;
        shim_release_string(b->key);
        shim_zval_dtor(&b->val);
    }
    _efree(ht->arData);
    _efree(ht);
}
ZEND_API void ZEND_FASTCALL zval_ptr_dtor(zval *zval_ptr) { shim_zval_dtor(zval_ptr); }

/* PHP's headers redirect snprintf/vsnprintf to these; extensions (apcu
 * included) call them. Both must be provided, since one calls the other. */
#undef snprintf
#undef vsnprintf
PHPAPI int ap_php_vsnprintf(char *buf, size_t len, const char *format, va_list ap) {
    return vsnprintf(buf, len, format, ap);
}
PHPAPI int ap_php_snprintf(char *buf, size_t len, const char *format, ...) {
    va_list ap; va_start(ap, format);
    int n = ap_php_vsnprintf(buf, len, format, ap);
    va_end(ap);
    return n;
}

/* ------------------------------------------------------- inert / stubbed --- */
/* phpinfo output has no meaning outside PHP: harmless no-ops. */
PHPAPI void php_info_print_table_start(void) {}
PHPAPI void php_info_print_table_end(void) {}
PHPAPI void php_info_print_table_header(int num_cols, ...) { (void)num_cols; }
PHPAPI void php_info_print_table_row(int num_cols, ...) { (void)num_cols; }

/* ----------------------------------------------------------- object model -- */
/* stdClass-equivalent: properties live in a lazily created HashTable, which is
 * what zend_std_write_property does for a class with no declared properties. */

static zend_class_entry shim_std_class;
static uint32_t shim_object_handle = 0;
size_t shim_objects = 0;

ZEND_API void object_init(zval *arg) {
    zend_object *obj = (zend_object *)_emalloc(sizeof(zend_object));
    memset(obj, 0, sizeof(*obj));
    GC_SET_REFCOUNT(obj, 1);
    GC_TYPE_INFO(obj) = GC_OBJECT;
    obj->handle = ++shim_object_handle;
    obj->ce = &shim_std_class;
    obj->handlers = &std_object_handlers;
    obj->properties = NULL;              /* created on first write */
    ZVAL_OBJ(arg, obj);
    shim_objects++;
}

ZEND_API zval *zend_std_write_property(zend_object *obj, zend_string *name, zval *value, void **cache_slot) {
    (void)cache_slot;
    if (!obj->properties) obj->properties = _zend_new_array_0();
    /* Like the key in zend_hash_update, the *value* is taken by reference here:
     * simdjson calls zval_ptr_dtor_nogc(&value) immediately afterwards. Without
     * this addref the stored property is freed underneath the object. */
    if (Z_REFCOUNTED_P(value)) Z_ADDREF_P(value);
    zend_hash_update(obj->properties, name, value);
    return value;
}

static void shim_free_object(zend_object *obj) {
    if (GC_DELREF(obj) != 0) return;
    if (obj->properties) shim_free_ht(obj->properties);
    _efree(obj);
}
ZEND_API zend_class_entry *zend_register_internal_class_ex(zend_class_entry *ce, zend_class_entry *p) {
    (void)ce; (void)p; SHIM_UNIMPLEMENTED("zend_register_internal_class_ex");
}
ZEND_API zend_constant *zend_register_long_constant(const char *name, size_t name_len, zend_long lval, int flags, int module_number) {
    (void)name; (void)name_len; (void)lval; (void)flags; (void)module_number;
    return NULL; /* constants are only meaningful to a PHP compiler */
}
ZEND_API ZEND_COLD void zend_throw_error(zend_class_entry *ce, const char *format, ...) {
    (void)ce; fprintf(stderr, "shim: zend_throw_error: %s\n", format); abort();
}
ZEND_API ZEND_COLD zend_object *zend_throw_exception(zend_class_entry *ce, const char *message, zend_long code) {
    (void)ce; (void)code; fprintf(stderr, "shim: exception: %s\n", message ? message : "(null)"); abort();
}
/* Real implementation, not a stub: simdjson_decode() calls
 * zend_parse_parameters(ZEND_NUM_ARGS(), "S|bl", ...) — the classic ZPP path,
 * not the FAST_ZPP macros. Supports the spec letters this prototype needs:
 *   S = zend_string*   s = char*+len   l = zend_long   d = double
 *   b = bool           a = array zval* z = raw zval*   | = optional from here
 * Anything else is refused loudly rather than silently mis-parsed. */
ZEND_API zend_result zend_parse_parameters(uint32_t num_args, const char *type_spec, ...) {
    va_list va;
    va_start(va, type_spec);

    uint32_t min_args = 0, max_args = 0;
    for (const char *p = type_spec; *p; p++) {
        if (*p == '|') continue;
        max_args++;
        if (!strchr(type_spec, '|') || (size_t)(p - type_spec) < (size_t)(strchr(type_spec, '|') - type_spec))
            min_args++;
    }
    if (num_args < min_args || num_args > max_args) {
        fprintf(stderr, "shim: %s() expects %u..%u args, got %u\n",
                "function", min_args, max_args, num_args);
        va_end(va);
        return FAILURE;
    }

    uint32_t argno = 0;
    for (const char *p = type_spec; *p; p++) {
        if (*p == '|') continue;
        argno++;
        if (argno > num_args) break;             /* optional, not supplied */
        zval *arg = ZEND_CALL_ARG(shim_current_execute_data, argno);
        ZVAL_DEREF(arg);
        switch (*p) {
            case 'S': {
                zend_string **dest = va_arg(va, zend_string **);
                if (Z_TYPE_P(arg) != IS_STRING) { va_end(va); return FAILURE; }
                *dest = Z_STR_P(arg);
                break;
            }
            case 's': {
                char **d = va_arg(va, char **);
                size_t *l = va_arg(va, size_t *);
                if (Z_TYPE_P(arg) != IS_STRING) { va_end(va); return FAILURE; }
                *d = Z_STRVAL_P(arg); *l = Z_STRLEN_P(arg);
                break;
            }
            case 'l': {
                zend_long *d = va_arg(va, zend_long *);
                if (Z_TYPE_P(arg) != IS_LONG) { va_end(va); return FAILURE; }
                *d = Z_LVAL_P(arg);
                break;
            }
            case 'd': {
                double *d = va_arg(va, double *);
                if (Z_TYPE_P(arg) == IS_DOUBLE) *d = Z_DVAL_P(arg);
                else if (Z_TYPE_P(arg) == IS_LONG) *d = (double)Z_LVAL_P(arg);
                else { va_end(va); return FAILURE; }
                break;
            }
            case 'b': {
                bool *d = va_arg(va, bool *);
                if (Z_TYPE_P(arg) == IS_TRUE) *d = 1;
                else if (Z_TYPE_P(arg) == IS_FALSE) *d = 0;
                else { va_end(va); return FAILURE; }
                break;
            }
            case 'a': {
                zval **d = va_arg(va, zval **);
                if (Z_TYPE_P(arg) != IS_ARRAY) { va_end(va); return FAILURE; }
                *d = arg;
                break;
            }
            case 'z': { zval **d = va_arg(va, zval **); *d = arg; break; }
            default:
                fprintf(stderr, "shim: unsupported zend_parse_parameters spec '%c'\n", *p);
                va_end(va);
                abort();
        }
    }
    va_end(va);
    return SUCCESS;
}

/* ------------------------------------------- calling back into host code --- */
/* This is the seam that matters for Elephc: an extension wants to invoke a PHP
 * callable, but under Elephc "PHP code" is AOT-compiled native code, not VM
 * opcodes. We model a host function the way PHP models an internal function —
 * a zend_function whose internal_function.handler is a native pointer — which
 * is exactly how Elephc could present its compiled functions to extensions. */

ZEND_API zend_result zend_call_function(zend_fcall_info *fci, zend_fcall_info_cache *fci_cache) {
    zend_function *fn = fci_cache ? fci_cache->function_handler : NULL;

    /* Resolve by name when the caller did not pre-resolve (the common case:
     * the extension received a callable zval straight from user code). */
    if (!fn && Z_TYPE(fci->function_name) == IS_PTR) {
        fn = (zend_function *)Z_PTR(fci->function_name);
    }
    if (!fn) {
        fprintf(stderr, "shim: zend_call_function with an unresolvable callable\n");
        return FAILURE;
    }
    if (fn->type != ZEND_INTERNAL_FUNCTION || !fn->internal_function.handler) {
        fprintf(stderr, "shim: only native host functions are callable in this prototype\n");
        return FAILURE;
    }

    /* Build the frame the handler expects, same layout as a real call. */
    uint32_t nargs = fci->param_count;
    zval *frame = (zval *)_ecalloc(ZEND_CALL_FRAME_SLOT + (nargs ? nargs : 1), sizeof(zval));
    zend_execute_data *ex = (zend_execute_data *)frame;
    ex->func = fn;
    ZEND_CALL_NUM_ARGS(ex) = nargs;
    for (uint32_t i = 0; i < nargs; i++) {
        ZVAL_COPY_VALUE(ZEND_CALL_ARG(ex, i + 1), &fci->params[i]);
    }

    zend_execute_data *saved = shim_current_execute_data;
    shim_current_execute_data = ex;
    if (fci->retval) ZVAL_NULL(fci->retval);
    fn->internal_function.handler(ex, fci->retval);
    shim_current_execute_data = saved;

    _efree(frame);
    return SUCCESS;
}

/* ---------------------------------------------------------------- startup -- */

void shim_startup(void) {
    memset(&shim_std_class, 0, sizeof(shim_std_class));
    shim_std_class.type = ZEND_INTERNAL_CLASS;
    shim_std_class.name = shim_string_alloc_ex("stdClass", sizeof("stdClass") - 1, true);
    shim_std_class.default_properties_count = 0;

    /* These are interned in real PHP: release must be a no-op on them. */
    zend_empty_string = shim_string_alloc_ex("", 0, true);
    for (int i = 0; i < 256; i++) { char c = (char)i; zend_one_char_string[i] = shim_string_alloc_ex(&c, 1, true); }
    shim_ht_init((HashTable *)&zend_empty_array, 8);
}
