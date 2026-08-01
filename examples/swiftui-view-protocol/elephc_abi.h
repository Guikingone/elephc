// The C view of elephc's cdylib string ABI.
//
// Swift refuses a Swift-declared struct in a `@convention(c)` function type --
// it is not C-representable, so the compiler cannot know it rides the platform's
// aggregate-return registers. Declaring it here and importing the header makes
// `ElephcStr` a genuine C type, after which the function pointers type-check and
// the calls follow the same ABI a C host would use.

#ifndef ELEPHC_ABI_H
#define ELEPHC_ABI_H

#include <stddef.h>

/// What a string-returning `#[Export]` hands back: a pointer and a length.
///
/// The buffer is owned by the caller and must be released with `elephc_free`.
/// It is a PHP byte string -- not NUL-terminated, and free to contain interior
/// zero bytes -- so `len` is authoritative and `strlen` is wrong.
typedef struct {
    const char *ptr;
    size_t len;
} ElephcStr;

#endif /* ELEPHC_ABI_H */
