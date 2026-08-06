/* zend_shim_arena.c — request-scoped arena + persistent allocations.
 *
 * Zend's memory manager has two lifetimes, and extensions depend on both:
 *   - request memory (emalloc/efree): reclaimed wholesale at request end. This
 *     is why PHP can longjmp out of a fatal and leak nothing — the arena is
 *     discarded, so skipped frees and skipped C++ destructors cost nothing.
 *   - persistent memory (pemalloc(size, 1)): survives requests. APCu allocates
 *     its cache header this way at MINIT.
 *
 * The bailout probe showed these two are not separate concerns: arena teardown
 * IS the cleanup mechanism that makes a fatal survivable. Refcount semantics
 * stay faithful (addref/delref, destruction at rc=0) — only the *backing
 * memory* is recycled lazily, which is Zend's own request-mode policy.
 */

#include "php.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define ARENA_ALIGN     16
#define ARENA_BLOCK_MIN (64 * 1024)

typedef struct arena_block {
    struct arena_block *next;
    size_t used, cap;
    char data[];
} arena_block;

static arena_block *arena_head = NULL;

/* Observable counters, so tests can assert rather than trust. */
size_t shim_arena_requested = 0;   /* bytes handed to the extension */
size_t shim_arena_blocks    = 0;
size_t shim_arena_frees     = 0;   /* efree calls (memory returns at teardown) */
size_t shim_persistent_live = 0;   /* persistent bytes currently outstanding */
size_t shim_requests_ended  = 0;

static size_t align_up(size_t n) { return (n + ARENA_ALIGN - 1) & ~(size_t)(ARENA_ALIGN - 1); }

/* Every allocation carries its size, so erealloc can copy and so persistent
 * blocks can be distinguished from arena blocks on free. */
typedef struct { size_t size; size_t persistent; } alloc_hdr;

static void *arena_alloc(size_t size) {
    size_t need = align_up(sizeof(alloc_hdr)) + align_up(size ? size : 1);
    if (!arena_head || arena_head->used + need > arena_head->cap) {
        size_t cap = need > ARENA_BLOCK_MIN ? need : ARENA_BLOCK_MIN;
        arena_block *b = (arena_block *)malloc(sizeof(arena_block) + cap);
        if (!b) { fprintf(stderr, "shim: arena out of memory\n"); abort(); }
        b->next = arena_head; b->used = 0; b->cap = cap;
        arena_head = b;
        shim_arena_blocks++;
    }
    char *p = arena_head->data + arena_head->used;
    arena_head->used += need;
    alloc_hdr *h = (alloc_hdr *)p;
    h->size = size; h->persistent = 0;
    shim_arena_requested += size;
    return p + align_up(sizeof(alloc_hdr));
}

static void *persistent_alloc(size_t size) {
    char *p = (char *)malloc(align_up(sizeof(alloc_hdr)) + size);
    if (!p) { fprintf(stderr, "shim: out of memory (persistent)\n"); abort(); }
    alloc_hdr *h = (alloc_hdr *)p;
    h->size = size; h->persistent = 1;
    shim_persistent_live += size;
    return p + align_up(sizeof(alloc_hdr));
}

static alloc_hdr *hdr_of(void *ptr) {
    return (alloc_hdr *)((char *)ptr - align_up(sizeof(alloc_hdr)));
}

void *shim_alloc(size_t size, int persistent) {
    return persistent ? persistent_alloc(size) : arena_alloc(size);
}

void shim_free(void *ptr) {
    if (!ptr) return;
    alloc_hdr *h = hdr_of(ptr);
    if (h->persistent) {
        shim_persistent_live -= h->size;
        free(h);
        return;
    }
    /* Request memory: reclaimed wholesale at teardown, exactly as Zend does. */
    shim_arena_frees++;
}

void *shim_realloc(void *ptr, size_t size) {
    if (!ptr) return shim_alloc(size, 0);
    alloc_hdr *h = hdr_of(ptr);
    if (h->persistent) {
        char *p = (char *)realloc(h, align_up(sizeof(alloc_hdr)) + size);
        shim_persistent_live += size - h->size;
        ((alloc_hdr *)p)->size = size;
        return p + align_up(sizeof(alloc_hdr));
    }
    void *n = arena_alloc(size);
    memcpy(n, ptr, h->size < size ? h->size : size);
    return n;
}

/* ---- request boundary: this is what makes a fatal survivable -------------- */

void shim_request_end(void) {
    arena_block *b = arena_head;
    while (b) { arena_block *n = b->next; free(b); b = n; }
    arena_head = NULL;
    shim_requests_ended++;
}

size_t shim_arena_live_blocks(void) {
    size_t n = 0;
    for (arena_block *b = arena_head; b; b = b->next) n++;
    return n;
}

/* Zend routes constant-size allocations to size-binned symbols (_emalloc_1024
 * and friends). Extensions hit these constantly, so define the whole ladder
 * rather than discovering them one link error at a time. */
#define SHIM_BINS(X) \
    X(8) X(16) X(24) X(32) X(40) X(48) X(56) X(64) X(80) X(96) X(112) X(128) \
    X(160) X(192) X(224) X(256) X(320) X(384) X(448) X(512) X(640) X(768) \
    X(896) X(1024) X(1280) X(1536) X(1792) X(2048) X(2560) X(3072)

#define SHIM_DEFINE_BIN(n)                                                     \
    ZEND_API void *ZEND_FASTCALL _emalloc_##n(void) { return shim_alloc(n, 0); } \
    ZEND_API void ZEND_FASTCALL _efree_##n(void *p) { shim_free(p); }
SHIM_BINS(SHIM_DEFINE_BIN)
#undef SHIM_DEFINE_BIN

ZEND_API void *ZEND_FASTCALL _emalloc_large(size_t size) { return shim_alloc(size, 0); }
ZEND_API void ZEND_FASTCALL _efree_large(void *p, size_t size) { (void)size; shim_free(p); }
ZEND_API void *ZEND_FASTCALL _emalloc_huge(size_t size) { return shim_alloc(size, 0); }
ZEND_API void ZEND_FASTCALL _efree_huge(void *p, size_t size) { (void)size; shim_free(p); }

/* PHP's persistent-allocation entry points, which extensions call directly. */
ZEND_API void *ZEND_FASTCALL __zend_malloc(size_t len) { return shim_alloc(len, 1); }
ZEND_API void *ZEND_FASTCALL __zend_calloc(size_t n, size_t l) {
    void *p = shim_alloc(n * l, 1); memset(p, 0, n * l); return p;
}
ZEND_API void *ZEND_FASTCALL __zend_realloc(void *p, size_t len) { return shim_realloc(p, len); }
