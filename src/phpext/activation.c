/* Native execution uses no Zend API. The mutex protects complete request snapshots. */
#include <pthread.h>
#include <Zend/zend_exceptions.h>

extern unsigned char elephc_state_begin[], elephc_state_end[];
extern unsigned char elephc_state_zero_begin[], elephc_state_zero_end[];
extern const uintptr_t ep_class_name_count __asm__("_class_name_count");
extern const uintptr_t ep_class_name_entries[] __asm__("_class_name_entries");
extern uintptr_t ep_native_stack_limit __asm__("_stack_limit");
extern uintptr_t ep_native_stack_limit_main __asm__("_stack_limit_main");
extern uintptr_t ep_boundary_status __asm__("_elephc_boundary_status");

typedef void (*ep_native_function)(const zval *const *);
static pthread_mutex_t ep_native_lock = PTHREAD_MUTEX_INITIALIZER;
static unsigned char *ep_template;
static size_t ep_data_size, ep_zero_size;
static ep_call *ep_current;

/* Initialized and zero-filled native storage need not be adjacent in the loaded module. */
static void ep_state_save(unsigned char *image)
{
    memcpy(image, elephc_state_begin, ep_data_size);
    memcpy(image + ep_data_size, elephc_state_zero_begin, ep_zero_size);
}

static void ep_state_restore(const unsigned char *image)
{
    memcpy(elephc_state_begin, image, ep_data_size);
    memcpy(elephc_state_zero_begin, image + ep_data_size, ep_zero_size);
}

/* The initial zero region is implicit, so MINIT stores only initialized bytes. */
static void ep_state_reset(void)
{
    memcpy(elephc_state_begin, ep_template, ep_data_size);
    memset(elephc_state_zero_begin, 0, ep_zero_size);
}

static int ep_module_startup(void)
{
    ep_data_size = (uintptr_t)elephc_state_end - (uintptr_t)elephc_state_begin;
    ep_zero_size = (uintptr_t)elephc_state_zero_end - (uintptr_t)elephc_state_zero_begin;
    if (ep_zero_size > SIZE_MAX - ep_data_size) { return FAILURE; }
    ep_state_size = ep_data_size + ep_zero_size;
    ep_template = malloc(ep_data_size ? ep_data_size : 1);
    if (!ep_template) { return FAILURE; }
    memcpy(ep_template, elephc_state_begin, ep_data_size);
    return SUCCESS;
}

/* Zend knows the current thread or Fiber stack, unlike process RLIMIT_STACK. */
static uintptr_t ep_stack_floor(void)
{
    if (EG(stack_limit)) { return (uintptr_t)EG(stack_limit); }
#if defined(__APPLE__)
    uintptr_t top = (uintptr_t)pthread_get_stackaddr_np(pthread_self());
    size_t size = pthread_get_stacksize_np(pthread_self());
    return size > 65536 && top > size ? top - size + 65536 : 0;
#elif defined(__linux__)
    pthread_attr_t attributes;
    void *base = NULL;
    size_t size = 0;
    if (pthread_getattr_np(pthread_self(), &attributes)) { return 0; }
    int result = pthread_attr_getstack(&attributes, &base, &size);
    pthread_attr_destroy(&attributes);
    return !result && size > 65536 ? (uintptr_t)base + 65536 : 0;
#else
    return 0;
#endif
}

/* This callback only reads the activation record and cannot allocate or bailout. */
uintptr_t elephc_php_stack_floor(void)
{
    return ep_current->stack_floor;
}

ZEND_BEGIN_MODULE_GLOBALS(@IDENT@)
    unsigned char *snapshot;
    ep_call *active;
    bool initialized;
ZEND_END_MODULE_GLOBALS(@IDENT@)
ZEND_DECLARE_MODULE_GLOBALS(@IDENT@)
#define EP_GLOBAL(member) ZEND_MODULE_GLOBALS_ACCESSOR(@IDENT@, member)

/* Requests that never invoke an export do not allocate or touch the native heap. */
static int ep_request_startup(void)
{
    ep_call_abandon_chain(EP_GLOBAL(active));
    free(EP_GLOBAL(snapshot));
    EP_GLOBAL(snapshot) = NULL;
    EP_GLOBAL(active) = NULL;
    EP_GLOBAL(initialized) = true;
    return SUCCESS;
}

/* Native warning lines become length-aware messages for the host error handler. */
static void ep_issue_warning(ep_call *call)
{
    const char *message = call->output;
    size_t length = call->output_len;
    int level = E_WARNING;
    size_t prefix = 0;
    if (length >= 9 && memcmp(message, "Warning: ", 9) == 0) { prefix = 9; }
    else if (length >= 8 && memcmp(message, "Notice: ", 8) == 0) { prefix = 8; level = E_NOTICE; }
    else if (length >= 12 && memcmp(message, "Deprecated: ", 12) == 0) { prefix = 12; level = E_DEPRECATED; }
    message += prefix;
    length -= prefix;
    while (length && message[length - 1] == '\n') { length--; }
    call->warning = zend_string_init(message, length, false);
    zend_error_zstr(level, call->warning);
}

/* A real exit remains authoritative; Fiber destruction uses ordinary finally semantics. */
static bool ep_is_host_exit(void)
{
    return EG(exception) && zend_is_unwind_exit(EG(exception));
}

/* Infrastructure statuses remain authoritative across native finally return and throw. */
static const char *ep_infrastructure_error(uintptr_t status)
{
    switch (status) {
        case 1: return "PHP extension result exceeds the maximum nesting depth (64)";
        case 3: return "elephc heap allocation failed";
        case 4: return "elephc native runtime failed";
        default: return NULL;
    }
}

/* Retire a host exception only after the native image is parked and Zend is callable. */
static void ep_clear_discarded_host_exception(ep_call *call)
{
    if (!call->discarded_host_exception || ep_is_host_exit()) { return; }
    if (EG(exception)) { zend_clear_exception(); }
    call->host_exception = false;
    call->discarded_host_exception = false;
    call->previous_exceptions = NULL;
    call->previous_exceptions_tail = NULL;
    call->previous_exception_anchor = NULL;
}

/* Park the native image around a guarded Zend callback, including nested module calls. */
static int ep_deliver(const char *bytes, size_t length, bool warning)
{
    ep_call *call = ep_current;
    if (call->error || call->host_bailout || ep_is_host_exit()) { return 1; }
    if (!length) { return 0; }
    if (!call->checkpoint) {
        /* Recovery storage is bounded independently and does not consume transport bytes. */
        call->checkpoint = malloc(ep_state_size ? ep_state_size : 1);
        if (!call->checkpoint) { call->error = "PHP extension checkpoint allocation failed"; return 1; }
        memcpy(call->checkpoint, call->snapshot, ep_state_size);
    }
    if (length > call->output_capacity) {
        char *buffer = realloc(call->output, length);
        if (!buffer) { call->error = "PHP extension output allocation failed"; return 1; }
        call->output = buffer;
        call->output_capacity = length;
    }
    memcpy(call->output, bytes, length);
    call->output_len = length;
    ep_state_save(call->snapshot);
    ep_state_reset();
    if (warning) { call->parked_warning_buffer = (char *)bytes; }
    ep_current = NULL;
    pthread_mutex_unlock(&ep_native_lock);
    zend_try {
        ep_clear_discarded_host_exception(call);
    } zend_catch {
        call->host_bailout = true;
    } zend_end_try();
    /* Native finally blocks run while the first host callback exception is pending.
     * Temporarily park it so later cleanup output and warnings can invoke PHP just as
     * Zend does, then restore it behind any replacement exception. */
    zend_object *saved_exception_object = EG(exception);
    bool saved_exception = saved_exception_object != NULL;
    volatile bool saved_exception_can_anchor = saved_exception
        && !zend_is_unwind_exit(saved_exception_object)
        && !zend_is_graceful_exit(saved_exception_object);
    volatile bool saved_exception_parked = false;
    if (!call->host_bailout) {
        if (saved_exception) { zend_exception_save(); saved_exception_parked = true; }
        zend_try {
            if (warning) {
                ep_issue_warning(call);
            } else {
                PHPWRITE(call->output, length);
            }
        } zend_catch {
            call->host_bailout = true;
        } zend_end_try();
    }
    if (call->warning) { zend_string_release(call->warning); call->warning = NULL; }
    call->parked_warning_buffer = NULL;
    bool callback_exception = EG(exception) != NULL;
    if (saved_exception_parked) { zend_exception_restore(); }
    if (callback_exception) {
        call->previous_exception_anchor = saved_exception_can_anchor ? saved_exception_object : NULL;
    }
    call->host_exception = EG(exception) != NULL;
    /* The module-local normal mutex remains initialized for every active request. */
    pthread_mutex_lock(&ep_native_lock);
    ep_state_restore(call->snapshot);
    ep_native_stack_limit = ep_native_stack_limit_main = call->stack_floor;
    ep_current = call;
    return call->host_bailout || callback_exception;
}

/* No Zend bailout escapes this C frame into the native stack. */
int elephc_php_write(const char *bytes, size_t length) { return ep_deliver(bytes, length, false); }

/* Warnings use the host error reporting mask and host PHP error handler. */
int elephc_php_warning(const char *bytes, size_t length) { return ep_deliver(bytes, length, true); }

/* The native result is copied before its heap owner is retired. */
void elephc_php_result(const zval *value)
{
    if (ep_current && !ep_current->error) {
        const char *error = ep_infrastructure_error(ep_boundary_status);
        if (error) { ep_current->error = error; return; }
        ep_current->result = ep_copy(ep_current, value, false, 0);
    }
}

/* Native code can only flag the discard while it owns the runtime lock. */
void elephc_php_discard_exception(const uintptr_t *object)
{
    if (!object && ep_current && ep_current->host_exception && !ep_current->host_bailout) {
        ep_current->discarded_host_exception = true;
    }
}

/* Native conversion reports a type failure without allocating or entering Zend. */
void elephc_php_unsupported_result(void)
{
    if (ep_current && !ep_current->error) {
        ep_current->error = "PHP extension boundary supports only int, float, bool, string, array and null";
    }
}

/* Copy compact native Throwable fields while their owned message remains live. */
static int ep_copy_exception(ep_call *call, const uintptr_t *object, char **name,
    size_t *name_len, char **message, size_t *message_len, zend_long *code)
{
    uintptr_t id = object[0];
    if (id < ep_class_name_count) {
        *name_len = ep_class_name_entries[id * 2 + 1];
        *name = ep_alloc(call, *name_len + 1);
        if (*name) {
            memcpy(*name, (void *)ep_class_name_entries[id * 2], *name_len);
        }
    }
    *message_len = object[2];
    *message = ep_alloc(call, *message_len + 1);
    if (*message) {
        memcpy(*message, (void *)object[1], *message_len);
    }
    *code = (zend_long)object[3];
    return call->error != NULL;
}

/* Copy one native previous-chain node and append it to the active copied root. */
int elephc_php_exception_previous(const uintptr_t *object)
{
    ep_call *call = ep_current;
    if (!call || !object || call->error) { return 1; }
    ep_exception_copy *copy = ep_alloc(call, sizeof(ep_exception_copy));
    if (!copy || ep_copy_exception(call, object, &copy->name, &copy->name_len,
        &copy->message, &copy->message_len, &copy->code)) {
        return 1;
    }
    if (call->exception_chain_tail) {
        call->exception_chain_tail->previous = copy;
    } else {
        call->exception_previous = copy;
    }
    call->exception_chain_tail = copy;
    return 0;
}

/* Preserve an older native Throwable before a host callback escape consumes its owner. */
void elephc_php_previous_exception(const uintptr_t *object)
{
    ep_call *call = ep_current;
    if (!call || !object || call->error) { return; }
    ep_exception_copy *copy = ep_alloc(call, sizeof(ep_exception_copy));
    if (!copy || ep_copy_exception(call, object, &copy->name, &copy->name_len,
        &copy->message, &copy->message_len, &copy->code)) {
        return;
    }
    copy->anchor = call->previous_exception_anchor;
    if (call->previous_exceptions_tail) {
        call->previous_exceptions_tail->next = copy;
    } else {
        call->previous_exceptions = copy;
    }
    call->previous_exceptions_tail = copy;
    call->exception_chain_tail = copy;
}

/* Copy the primary compact native Throwable at the invocation boundary. */
int elephc_php_exception(const uintptr_t *object, uintptr_t status)
{
    ep_call *call = ep_current;
    if (!call) { return 1; }
    if (call->host_bailout) { return 0; }
    if (call->error) { return 1; }
    const char *error = ep_infrastructure_error(status);
    if (error) { call->error = error; return 1; }
    if (call->host_exception && !object && status == 2) { return 0; }
    if (!object) {
        call->error = "elephc native runtime failed";
        return 1;
    }
    call->exception_previous = NULL;
    call->exception_chain_tail = NULL;
    return ep_copy_exception(call, object, &call->exception_name, &call->exception_name_len,
        &call->exception_message, &call->exception_message_len, &call->exception_code);
}

/* Resolve existing host Throwable classes without invoking user autoloaders. */
static zend_class_entry *ep_exception_class(char *name, size_t name_len)
{
    zend_class_entry *ce = NULL;
    if (name) {
        for (size_t i = 0; i < name_len; i++) {
            unsigned char c = (unsigned char)name[i];
            if (c >= 'A' && c <= 'Z') { name[i] = (char)(c + ('a' - 'A')); }
        }
        ce = zend_hash_str_find_ptr(CG(class_table), name, name_len);
    }
    if (!ce || ce->type != ZEND_INTERNAL_CLASS || !instanceof_function(ce, zend_ce_throwable)) {
        ce = zend_ce_exception;
    }
    return ce;
}

/* Materialize one copied native Throwable without invoking a user autoloader. */
static void ep_materialize_exception(char *name, size_t name_len, char *message,
    size_t message_len, zend_long code, zval *value)
{
    zend_class_entry *ce = ep_exception_class(name, name_len);
    object_init_ex(value, ce);
    zend_object *exception = Z_OBJ_P(value);
    zend_update_property_stringl(ce, exception, "message", sizeof("message") - 1,
        message, message_len);
    zend_update_property_long(ce, exception, "code", sizeof("code") - 1, code);
}

/* Materialize copied previous links in order without recursing on the C stack. */
static zend_object *ep_materialize_previous_chain(zend_object *tail, ep_exception_copy *copy)
{
    while (copy) {
        zval value;
        ep_materialize_exception(copy->name, copy->name_len, copy->message,
            copy->message_len, copy->code, &value);
        zend_object *previous = Z_OBJ(value);
        zend_exception_set_previous(tail, previous);
        ZVAL_UNDEF(&value);
        tail = previous;
        copy = copy->previous;
    }
    return tail;
}

/* Materialize one copied native Throwable and every native previous link it owns. */
static zend_object *ep_materialize_exception_copy(ep_exception_copy *copy, zval *value)
{
    ep_materialize_exception(copy->name, copy->name_len, copy->message,
        copy->message_len, copy->code, value);
    return ep_materialize_previous_chain(Z_OBJ_P(value), copy->previous);
}

/* Throwable previous properties are private on either the Exception or Error base. */
static zend_object *ep_host_previous(zend_object *exception)
{
    zval rv;
    zval *previous = zend_read_property(zend_get_exception_base(exception), exception,
        "previous", sizeof("previous") - 1, true, &rv);
    ZVAL_DEREF(previous);
    return Z_TYPE_P(previous) == IS_OBJECT ? Z_OBJ_P(previous) : NULL;
}

/* Insert a transported native Throwable at the host-chain point where it arose. */
static void ep_insert_previous(ep_exception_copy *copy)
{
    zval value;
    zend_object *native_tail = ep_materialize_exception_copy(copy, &value);
    if (copy->anchor) {
        zend_object *cursor = EG(exception);
        while (cursor) {
            zend_object *previous = ep_host_previous(cursor);
            if (previous == copy->anchor) {
                GC_ADDREF(copy->anchor);
                zend_exception_set_previous(native_tail, copy->anchor);
                zend_update_property(zend_get_exception_base(cursor), cursor,
                    "previous", sizeof("previous") - 1, &value);
                zval_ptr_dtor(&value);
                return;
            }
            cursor = previous;
        }
    }
    zend_exception_set_previous(EG(exception), Z_OBJ(value));
    ZVAL_UNDEF(&value);
}

/* No Zend frame, allocation or callback lies below the native invocation. */
static void ep_invoke(INTERNAL_FUNCTION_PARAMETERS, ep_native_function function, zval *values,
    size_t count, bool integer_to_float)
{
    (void)execute_data;
    if (!EP_GLOBAL(initialized)) {
        zend_throw_error(NULL, "elephc PHP extension request is not initialized");
        return;
    }
    /* Request images contain pointers into parked native stacks. Only synchronous
     * reentry on their owning host context can safely share those handler chains. */
    if (EP_GLOBAL(active) && EP_GLOBAL(active)->fiber_context != EG(current_fiber_context)) {
        zend_throw_error(NULL, "elephc PHP extension has an active native call on another Fiber");
        return;
    }
    if (!EP_GLOBAL(snapshot)) {
        EP_GLOBAL(snapshot) = calloc(1, ep_state_size);
        if (!EP_GLOBAL(snapshot)) { zend_throw_error(NULL, "PHP extension snapshot allocation failed"); return; }
        memcpy(EP_GLOBAL(snapshot), ep_template, ep_data_size);
    }
    ep_call *call = calloc(1, sizeof(ep_call));
    if (!call) { zend_throw_error(NULL, "PHP extension activation allocation failed"); return; }
    call->parent = EP_GLOBAL(active);
    call->stack_floor = ep_stack_floor();
    call->fiber_context = EG(current_fiber_context);
    call->snapshot = EP_GLOBAL(snapshot);
    if (!call->stack_floor) { call->error = "Cannot determine the PHP thread stack boundary"; }
    const zval **inputs = ep_alloc(call, count * sizeof(zval *));
    if (inputs) {
        for (size_t i = 0; i < count; i++) {
            inputs[i] = ep_copy(call, &values[i], true, 0);
            if (!inputs[i]) { break; }
        }
    }
    if (!call->error) {
        int locked = pthread_mutex_lock(&ep_native_lock);
        if (locked) {
            call->error = "PHP extension runtime lock failed";
        } else {
            ep_state_restore(EP_GLOBAL(snapshot));
            EP_GLOBAL(active) = call;
            ep_current = call;
            function(inputs);
            ep_current = NULL;
            /* The thunk has retired every native frame, including contained unwinds. */
            EP_GLOBAL(active) = call->parent;
            /* Infrastructure failures can leave partial native imports or results. Roll back
             * the entire image; ordinary PHP exceptions keep their completed state changes. */
            if (!call->error) { ep_state_save(call->snapshot); }
            else if (call->checkpoint) { memcpy(call->snapshot, call->checkpoint, ep_state_size); }
            ep_state_reset();
            pthread_mutex_unlock(&ep_native_lock);
        }
    }
    /* The call allocation is on the heap: setjmp cannot invalidate its cleanup fields. */
    zend_try {
        if (call->host_bailout) { zend_bailout(); }
        ep_clear_discarded_host_exception(call);
        bool host_exit = ep_is_host_exit();
        if (!host_exit && call->error && EG(exception)) {
            /* A transport/runtime failure is authoritative over the incomplete host callback. */
            zend_clear_exception();
            call->host_exception = false;
        } else if (!host_exit && call->result && call->host_exception && EG(exception)) {
            /* A return from finally overrides the pending host callback exception. */
            zend_clear_exception();
            call->host_exception = false;
        }
        bool restore_host_exception = !host_exit && call->exception_message && EG(exception);
        if (!host_exit && EG(exception)) {
            for (ep_exception_copy *copy = call->previous_exceptions; copy; copy = copy->next) {
                ep_insert_previous(copy);
            }
        }
        if (restore_host_exception) {
            /* A native exception from finally replaces the host callback exception and keeps
             * that exception as its previous link, matching Zend's finally semantics. */
            zend_exception_save();
        }
        if (!EG(exception)) {
            if (call->error) {
                zend_throw_error(NULL, "%s", call->error);
            } else if (call->exception_message) {
                ep_materialize_exception(call->exception_name, call->exception_name_len,
                    call->exception_message, call->exception_message_len,
                    call->exception_code, &call->pending_exception);
                ep_materialize_previous_chain(Z_OBJ(call->pending_exception),
                    call->exception_previous);
                zval thrown = call->pending_exception;
                ZVAL_UNDEF(&call->pending_exception);
                zend_throw_exception_object(&thrown);
            } else if (call->result) {
                ep_to_zend(call->result, return_value);
                if (integer_to_float && Z_TYPE_P(return_value) == IS_LONG) {
                    ZVAL_DOUBLE(return_value, (double)Z_LVAL_P(return_value));
                }
            } else {
                zend_throw_error(NULL, "elephc native call returned no value");
            }
        }
        if (restore_host_exception) { zend_exception_restore(); }
    } zend_catch {
        ep_call_destroy(call);
        zend_bailout();
    } zend_end_try();
    ep_call_destroy(call);
}

static PHP_GINIT_FUNCTION(@IDENT@)
{
    @IDENT@_globals->snapshot = NULL;
    @IDENT@_globals->active = NULL;
    @IDENT@_globals->initialized = false;
}

/* Fiber destruction can resume parked callbacks after every module RSHUTDOWN.
 * Zend blocks switching before this post-deactivate phase, so native images and
 * activation records become unreachable only here. */
static ZEND_MODULE_POST_ZEND_DEACTIVATE_D(@IDENT@)
{
    ep_call_abandon_chain(EP_GLOBAL(active));
    free(EP_GLOBAL(snapshot));
    EP_GLOBAL(snapshot) = NULL;
    EP_GLOBAL(active) = NULL;
    EP_GLOBAL(initialized) = false;
    return SUCCESS;
}

/* Also covers partial request teardown that never reaches post-deactivate. */
static PHP_GSHUTDOWN_FUNCTION(@IDENT@)
{
    ep_call_abandon_chain(@IDENT@_globals->active);
    free(@IDENT@_globals->snapshot);
    @IDENT@_globals->snapshot = NULL;
    @IDENT@_globals->active = NULL;
    @IDENT@_globals->initialized = false;
}
