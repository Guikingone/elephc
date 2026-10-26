/* Native observations for borrowed_value.rs. Runtime ownership and allocation are real. */
#include <assert.h>
#include <stdint.h>
#include <string.h>

extern void *snapshot_value(void *) __asm__("__rt_borrowed_value_snapshot");
extern uint64_t *read_value(void *) __asm__("__rt_borrowed_value_read");
extern void free_snapshot(void *) __asm__("__rt_borrowed_value_free");
extern void *fetch_object(void) __asm__("_fn_fetch_u_object");
extern int64_t object_id(void *) __asm__("_fn_object_u_id");
extern uint64_t *fetch_value(void) __asm__("_fn_fetch_u_value");
extern void fixture_release_object(void *);
extern void fixture_release_cell(void *);
extern int64_t fixture_cell_kind(void *);
extern uint64_t *fixture_deref(void *);

static void *object_snapshot;
static uint64_t *saved;

/* Acquire no extra object owner while recording a scoped borrowed value. */
void fixture_capture_object(void) {
    void *object = fetch_object();
    uint64_t cell[3] = {6, (uint64_t)object, 0};
    uint32_t before = ((uint32_t *)object)[-3];
    object_snapshot = snapshot_value(cell);
    assert(((uint32_t *)object)[-3] == before);
    fixture_release_object(object);
}

/* A read during destruction owns the referent independently of the scratch allocation. */
int64_t fixture_resurrect(void) {
    saved = read_value(object_snapshot);
    assert(saved != object_snapshot && saved[0] == 6);
    return object_id((void *)saved[1]);
}

/* A later allocation must not replace the resurrected object's identity or payload. */
int64_t fixture_saved_id(void) {
    assert(saved && fixture_cell_kind((void *)saved[1]) == 4);
    return object_id((void *)saved[1]);
}

/* Destroy scratch separately from the owned result; scratch owns no referent. */
void fixture_clear_object(void) {
    free_snapshot(object_snapshot);
    object_snapshot = 0;
    fixture_release_cell(saved);
    saved = 0;
}

/* Check exact scalar bits, byte strings, shared container payloads and canonical resources. */
int64_t fixture_roundtrip(void) {
    uint64_t *owner = fetch_value();
    uint64_t *source = fixture_deref(owner);
    uint32_t before = ((uint32_t *)source)[-3];
    int container = source[0] == 4 || source[0] == 5 || source[0] == 6 || source[0] == 10;
    uint32_t child_before = container ? ((uint32_t *)source[1])[-3] : 0;
    void *snapshot = snapshot_value(owner);
    if (container) assert(((uint32_t *)source[1])[-3] == child_before);
    assert(((uint32_t *)source)[-3] == before);
    uint64_t *result = read_value(snapshot);
    assert(result != snapshot && result[0] == source[0]);
    if (source[0] == 1) {
        assert(result[2] == source[2]);
        assert(memcmp((void *)result[1], (void *)source[1], source[2]) == 0);
    } else {
        assert(result[1] == source[1] && result[2] == source[2]);
    }
    if (source[0] == 9) {
        assert(result == source);
        assert(((uint32_t *)source)[-3] == before + 1);
    } else {
        assert(result != source);
        assert(((uint32_t *)source)[-3] == before);
    }
    free_snapshot(snapshot);
    fixture_release_cell(result);
    assert(((uint32_t *)source)[-3] == before);
    fixture_release_cell(owner);
    return 1;
}
