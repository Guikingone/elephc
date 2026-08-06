/* zend_shim_ds.c — the surface a *class-based* extension needs, on top of
 * zend_shim.c. Target: ext/ds (Ds\Vector etc.) — objects, methods, callbacks.
 *
 * Kept separate from zend_shim.c so the already-green simdjson binary is not
 * disturbed while this is in flux. If ds works, the two merge.
 *
 * Note on executor_globals: it is simply *defined* here with its real type from
 * PHP's headers. That is the whole argument for compiling extensions against
 * headers we control — the "exact binary layout" problem only exists when you
 * must match a layout you did not choose.
 */

#include "php.h"
#include "zend_API.h"
#include "zend_exceptions.h"
#include "zend_interfaces.h"
#include "zend_objects.h"
#include "zend_objects_API.h"
#include "zend_operators.h"
#include "zend_smart_str.h"
#include "ext/standard/php_var.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#undef _emalloc
#undef _efree

extern void *ZEND_FASTCALL _emalloc(size_t);
extern void ZEND_FASTCALL _efree(void *);
extern void *ZEND_FASTCALL _ecalloc(size_t, size_t);

#define DS_UNIMPL(name)                                                        \
    do { fprintf(stderr, "shim(ds): '%s' reached — not implemented\n", (name));\
         abort(); } while (0)

/* ------------------------------------------------------------- globals ----- */

ZEND_API zend_executor_globals executor_globals;   /* layout from the headers */

ZEND_API zend_class_entry *zend_ce_traversable  = NULL;
ZEND_API zend_class_entry *zend_ce_aggregate    = NULL;
ZEND_API zend_class_entry *zend_ce_arrayaccess  = NULL;
ZEND_API zend_class_entry *zend_ce_countable    = NULL;
ZEND_API zend_class_entry *zend_ce_error        = NULL;
ZEND_API zend_class_entry *zend_ce_type_error   = NULL;
PHPAPI zend_class_entry *php_json_serializable_ce = NULL;
PHPAPI zend_class_entry *spl_ce_InvalidArgumentException = NULL;
PHPAPI zend_class_entry *spl_ce_OutOfBoundsException     = NULL;
PHPAPI zend_class_entry *spl_ce_OutOfRangeException      = NULL;
PHPAPI zend_class_entry *spl_ce_UnderflowException       = NULL;
PHPAPI zend_class_entry *spl_ce_UnexpectedValueException = NULL;

/* --------------------------------------------------------- allocator bins -- */
/* zend_alloc.h routes small constant-size allocations to size-binned symbols. */
/* Size-binned allocators now live in zend_shim_arena.c (the whole ladder). */
ZEND_API char *ZEND_FASTCALL _estrndup(const char *s, size_t n) {
    char *p = (char *)_emalloc(n + 1);
    memcpy(p, s, n); p[n] = '\0';
    return p;
}
ZEND_API void ZEND_FASTCALL smart_str_erealloc(smart_str *str, size_t len) {
    str->s = (zend_string *)realloc(str->s, _ZSTR_STRUCT_SIZE(len));
    ZSTR_LEN(str->s) = len;
}

/* ------------------------------------------------------------ hashtables --- */

extern zval *ZEND_FASTCALL zend_hash_update(HashTable *, zend_string *, zval *);
extern zval *ZEND_FASTCALL zend_hash_index_update(HashTable *, zend_ulong, zval *);
extern zval *ZEND_FASTCALL zend_hash_find(const HashTable *, zend_string *);

ZEND_API void ZEND_FASTCALL _zend_hash_init(HashTable *ht, uint32_t size, dtor_func_t d, bool persistent) {
    (void)persistent;
    memset(ht, 0, sizeof(*ht));
    if (size < 8) size = 8;
    HT_FLAGS(ht) = HASH_FLAG_STATIC_KEYS;
    ht->nTableSize = size;
    ht->nTableMask = (uint32_t)-(int32_t)size;
    ht->arData = (Bucket *)_ecalloc(size, sizeof(Bucket));
    ht->pDestructor = d;
}
ZEND_API zend_ulong ZEND_FASTCALL zend_string_hash_func(zend_string *s) {
    zend_ulong h = 5381;
    for (size_t i = 0; i < ZSTR_LEN(s); i++) h = h * 33 + (unsigned char)ZSTR_VAL(s)[i];
    h |= (1ULL << 63);
    ZSTR_H(s) = h;
    return h;
}

/* ------------------------------------------------- classes and interfaces -- */

static zend_class_entry *shim_new_ce(const char *name) {
    zend_class_entry *ce = (zend_class_entry *)_ecalloc(1, sizeof(zend_class_entry));
    ce->type = ZEND_INTERNAL_CLASS;
    ce->name = zend_string_init_interned(name, strlen(name), 1);
    _zend_hash_init(&ce->function_table, 8, NULL, 1);
    _zend_hash_init(&ce->constants_table, 8, NULL, 1);
    return ce;
}

ZEND_API zend_class_entry *zend_register_internal_class(zend_class_entry *orig) {
    zend_class_entry *ce = shim_new_ce(ZSTR_VAL(orig->name));
    ce->create_object = orig->create_object;
    ce->get_iterator = orig->get_iterator;
    ce->default_object_handlers = orig->default_object_handlers;
    /* Copy the declared method table: the extension filled it via its
     * zend_function_entry list before calling us. */
    if (orig->function_table.nNumUsed) {
        for (uint32_t i = 0; i < orig->function_table.nNumUsed; i++) {
            Bucket *b = orig->function_table.arData + i;
            if (b->key) zend_hash_update(&ce->function_table, b->key, &b->val);
        }
    }
    ce->constructor = orig->constructor;
    return ce;
}
ZEND_API zend_class_entry *zend_register_internal_interface(zend_class_entry *orig) {
    zend_class_entry *ce = shim_new_ce(ZSTR_VAL(orig->name));
    ce->ce_flags |= ZEND_ACC_INTERFACE;
    return ce;
}
ZEND_API void zend_class_implements(zend_class_entry *ce, int num, ...) {
    (void)ce; (void)num;   /* interface membership is not consulted in this probe */
}
ZEND_API void zend_declare_class_constant_long(zend_class_entry *ce, const char *n, size_t l, zend_long v) {
    zval z; ZVAL_LONG(&z, v);
    zend_string *k = zend_string_init_interned(n, l, 1);
    zend_hash_update(&ce->constants_table, k, &z);
}
ZEND_API void zend_declare_property_null(zend_class_entry *ce, const char *n, size_t l, int access) {
    (void)ce; (void)n; (void)l; (void)access;
}

/* ------------------------------------------------------------- objects ----- */

extern void ZEND_FASTCALL zval_ptr_dtor(zval *);

ZEND_API void zend_object_std_init(zend_object *obj, zend_class_entry *ce) {
    GC_SET_REFCOUNT(obj, 1);
    GC_TYPE_INFO(obj) = GC_OBJECT;
    obj->ce = ce;
    obj->handlers = ce->default_object_handlers ? ce->default_object_handlers : &std_object_handlers;
    obj->properties = NULL;
    static uint32_t handle = 0;
    obj->handle = ++handle;
}
ZEND_API void zend_object_std_dtor(zend_object *obj) {
    if (obj->properties) {
        zval tmp; ZVAL_ARR(&tmp, obj->properties);
        zval_ptr_dtor(&tmp);
        obj->properties = NULL;
    }
}
ZEND_API void object_properties_init(zend_object *obj, zend_class_entry *ce) {
    (void)ce; obj->properties = NULL;
}
ZEND_API void zend_objects_destroy_object(zend_object *obj) { (void)obj; }
ZEND_API void ZEND_FASTCALL zend_objects_store_del(zend_object *obj) {
    if (GC_DELREF(obj) == 0) {
        if (obj->handlers && obj->handlers->free_obj) obj->handlers->free_obj(obj);
        else { zend_object_std_dtor(obj); _efree(obj); }
    }
}
ZEND_API void ZEND_FASTCALL gc_possible_root(zend_refcounted *ref) { (void)ref; }

ZEND_API zval *zend_read_property(zend_class_entry *scope, zend_object *obj, const char *name, size_t len, bool silent, zval *rv) {
    (void)scope; (void)silent; (void)rv;
    if (!obj->properties) return NULL;
    zend_string *k = zend_string_init_interned(name, len, 0);
    return zend_hash_find(obj->properties, k);
}
ZEND_API void zend_update_property(const zend_class_entry *scope, zend_object *obj, const char *name, size_t len, zval *value) {
    (void)scope;
    extern zval *zend_std_write_property(zend_object *, zend_string *, zval *, void **);
    zend_string *k = zend_string_init_interned(name, len, 0);
    zend_std_write_property(obj, k, value, NULL);
}
ZEND_API void zend_update_property_null(const zend_class_entry *scope, zend_object *obj, const char *name, size_t len) {
    zval z; ZVAL_NULL(&z);
    zend_update_property(scope, obj, name, len, &z);
}
ZEND_API bool ZEND_FASTCALL instanceof_function_slow(const zend_class_entry *i, const zend_class_entry *c) {
    return i == c;
}

/* ------------------------------------------------- conversions / operators - */

ZEND_API zend_long ZEND_FASTCALL zval_get_long_func(const zval *op, bool silent) {
    (void)silent;
    switch (Z_TYPE_P(op)) {
        case IS_LONG:   return Z_LVAL_P(op);
        case IS_DOUBLE: return (zend_long)Z_DVAL_P(op);
        case IS_TRUE:   return 1;
        case IS_FALSE: case IS_NULL: return 0;
        case IS_STRING: return strtoll(Z_STRVAL_P(op), NULL, 10);
        default: return 0;
    }
}
ZEND_API zend_string *ZEND_FASTCALL zval_get_string_func(zval *op) {
    char buf[64];
    switch (Z_TYPE_P(op)) {
        case IS_STRING: GC_ADDREF(Z_STR_P(op)); return Z_STR_P(op);
        case IS_LONG:   { int n = snprintf(buf, sizeof buf, ZEND_LONG_FMT, Z_LVAL_P(op));
                          return zend_string_init_interned(buf, n, 0); }
        case IS_DOUBLE: { int n = snprintf(buf, sizeof buf, "%.*G", 17, Z_DVAL_P(op));
                          return zend_string_init_interned(buf, n, 0); }
        case IS_TRUE:   return zend_string_init_interned("1", 1, 0);
        default:        return zend_string_init_interned("", 0, 0);
    }
}
ZEND_API bool ZEND_FASTCALL zend_is_true(const zval *op) {
    switch (Z_TYPE_P(op)) {
        case IS_TRUE: return 1;
        case IS_FALSE: case IS_NULL: return 0;
        case IS_LONG: return Z_LVAL_P(op) != 0;
        case IS_DOUBLE: return Z_DVAL_P(op) != 0;
        case IS_STRING: return ZSTR_LEN(Z_STR_P(op)) > 0 && strcmp(Z_STRVAL_P(op), "0") != 0;
        default: return 1;
    }
}
ZEND_API bool ZEND_FASTCALL zend_is_identical(const zval *a, const zval *b) {
    if (Z_TYPE_P(a) != Z_TYPE_P(b)) return 0;
    switch (Z_TYPE_P(a)) {
        case IS_NULL: case IS_TRUE: case IS_FALSE: return 1;
        case IS_LONG:   return Z_LVAL_P(a) == Z_LVAL_P(b);
        case IS_DOUBLE: return Z_DVAL_P(a) == Z_DVAL_P(b);
        case IS_STRING: return ZSTR_LEN(Z_STR_P(a)) == ZSTR_LEN(Z_STR_P(b))
                            && memcmp(Z_STRVAL_P(a), Z_STRVAL_P(b), ZSTR_LEN(Z_STR_P(a))) == 0;
        default: return Z_COUNTED_P(a) == Z_COUNTED_P(b);
    }
}
ZEND_API zend_result ZEND_FASTCALL compare_function(zval *r, zval *a, zval *b) {
    if (Z_TYPE_P(a) == IS_LONG && Z_TYPE_P(b) == IS_LONG) {
        ZVAL_LONG(r, Z_LVAL_P(a) < Z_LVAL_P(b) ? -1 : (Z_LVAL_P(a) > Z_LVAL_P(b) ? 1 : 0));
        return SUCCESS;
    }
    double x = (Z_TYPE_P(a) == IS_DOUBLE) ? Z_DVAL_P(a) : (double)zval_get_long_func(a, 1);
    double y = (Z_TYPE_P(b) == IS_DOUBLE) ? Z_DVAL_P(b) : (double)zval_get_long_func(b, 1);
    ZVAL_LONG(r, x < y ? -1 : (x > y ? 1 : 0));
    return SUCCESS;
}
ZEND_API zend_result ZEND_FASTCALL add_function(zval *r, zval *a, zval *b) {
    if (Z_TYPE_P(a) == IS_LONG && Z_TYPE_P(b) == IS_LONG) {
        ZVAL_LONG(r, Z_LVAL_P(a) + Z_LVAL_P(b)); return SUCCESS;
    }
    double x = (Z_TYPE_P(a) == IS_DOUBLE) ? Z_DVAL_P(a) : (double)zval_get_long_func(a, 1);
    double y = (Z_TYPE_P(b) == IS_DOUBLE) ? Z_DVAL_P(b) : (double)zval_get_long_func(b, 1);
    ZVAL_DOUBLE(r, x + y);
    return SUCCESS;
}
ZEND_API void ZEND_FASTCALL convert_scalar_to_number(zval *op) {
    if (Z_TYPE_P(op) != IS_LONG && Z_TYPE_P(op) != IS_DOUBLE) {
        ZVAL_LONG(op, zval_get_long_func(op, 1));
    }
}
ZEND_API void ZEND_FASTCALL convert_to_object(zval *op) { (void)op; DS_UNIMPL("convert_to_object"); }
ZEND_API const char *zend_get_type_by_const(int type) {
    switch (type) {
        case IS_NULL: return "null";  case IS_FALSE: case IS_TRUE: return "bool";
        case IS_LONG: return "int";   case IS_DOUBLE: return "float";
        case IS_STRING: return "string"; case IS_ARRAY: return "array";
        case IS_OBJECT: return "object"; default: return "mixed";
    }
}

/* --------------------------------------------------- calls / misc surface -- */

extern zend_result zend_call_function(zend_fcall_info *, zend_fcall_info_cache *);

ZEND_API zval *zend_call_method(zend_object *obj, zend_class_entry *ce, zend_function **fn_proxy,
                                const char *name, size_t len, zval *retval,
                                uint32_t argc, zval *a1, zval *a2) {
    (void)obj; (void)ce; (void)fn_proxy; (void)name; (void)len;
    (void)argc; (void)a1; (void)a2;
    if (retval) ZVAL_NULL(retval);
    DS_UNIMPL("zend_call_method");
}
ZEND_API zend_result zend_parse_parameter(int flags, uint32_t arg_num, zval *arg, const char *spec, ...) {
    (void)flags; (void)arg_num; (void)arg; (void)spec;
    DS_UNIMPL("zend_parse_parameter");
}
ZEND_API ZEND_COLD void ZEND_FASTCALL zend_wrong_parameters_none_error(void) {
    fprintf(stderr, "shim(ds): wrong parameter count (none expected)\n");
}
ZEND_API zend_string *zend_vstrpprintf(size_t max_len, const char *format, va_list ap) {
    char buf[1024];
    int n = vsnprintf(buf, sizeof buf, format, ap);
    (void)max_len;
    return zend_string_init_interned(buf, n < 0 ? 0 : (size_t)n, 0);
}
ZEND_API void ZEND_FASTCALL add_assoc_zval_ex(zval *arr, const char *k, size_t l, zval *v) {
    zend_string *key = zend_string_init_interned(k, l, 0);
    zend_hash_update(Z_ARRVAL_P(arr), key, v);
}
ZEND_API zend_result array_set_zval_key(HashTable *ht, zval *key, zval *value) {
    if (Z_TYPE_P(key) == IS_LONG) zend_hash_index_update(ht, Z_LVAL_P(key), value);
    else zend_hash_update(ht, zval_get_string_func(key), value);
    return SUCCESS;
}
ZEND_API void zend_iterator_init(zend_object_iterator *iter) { (void)iter; }
typedef int (*shim_spl_iter_fn)(zend_object_iterator *iter, void *puser);
PHPAPI int spl_iterator_apply(zval *obj, shim_spl_iter_fn f, void *p) {
    (void)obj; (void)f; (void)p; DS_UNIMPL("spl_iterator_apply");
}

/* Serialization is a whole subsystem; ds only needs it for its own
 * serialize()/unserialize() handlers, which this probe does not exercise. */
PHPAPI void php_var_serialize(smart_str *b, zval *s, php_serialize_data_t *d) {
    (void)b; (void)s; (void)d; DS_UNIMPL("php_var_serialize");
}
PHPAPI php_serialize_data_t php_var_serialize_init(void) { DS_UNIMPL("php_var_serialize_init"); }
PHPAPI void php_var_serialize_destroy(php_serialize_data_t d) { (void)d; }
PHPAPI int php_var_unserialize(zval *r, const unsigned char **p, const unsigned char *m, php_unserialize_data_t *d) {
    (void)r; (void)p; (void)m; (void)d; DS_UNIMPL("php_var_unserialize");
}
PHPAPI php_unserialize_data_t php_var_unserialize_init(void) { DS_UNIMPL("php_var_unserialize_init"); }
PHPAPI void php_var_unserialize_destroy(php_unserialize_data_t d) { (void)d; }
PHPAPI void var_push_dtor(php_unserialize_data_t *d, zval *z) { (void)d; (void)z; }
PHPAPI zval *var_tmp_var(php_unserialize_data_t *d) { (void)d; DS_UNIMPL("var_tmp_var"); }

/* ds's own module globals, normally defined by ZEND_DECLARE_MODULE_GLOBALS in
 * php_ds.c. We supply them so the module's own object files need not be linked
 * — the host owns module state, exactly as it would under Elephc. */
#include "php_ds.h"
ZEND_DECLARE_MODULE_GLOBALS(ds)

void ds_shim_startup(void) {
    memset(&executor_globals, 0, sizeof(executor_globals));
    memset(&ds_globals, 0, sizeof(ds_globals));
    zend_ce_traversable = shim_new_ce("Traversable");
    zend_ce_aggregate   = shim_new_ce("IteratorAggregate");
    zend_ce_arrayaccess = shim_new_ce("ArrayAccess");
    zend_ce_countable   = shim_new_ce("Countable");
    zend_ce_error       = shim_new_ce("Error");
    zend_ce_type_error  = shim_new_ce("TypeError");
    php_json_serializable_ce = shim_new_ce("JsonSerializable");
    spl_ce_InvalidArgumentException = shim_new_ce("InvalidArgumentException");
    spl_ce_OutOfBoundsException     = shim_new_ce("OutOfBoundsException");
    spl_ce_OutOfRangeException      = shim_new_ce("OutOfRangeException");
    spl_ce_UnderflowException       = shim_new_ce("UnderflowException");
    spl_ce_UnexpectedValueException = shim_new_ce("UnexpectedValueException");
}
