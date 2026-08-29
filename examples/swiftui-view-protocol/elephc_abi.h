// Swift bridging header for the generated ABI-v3 library contract.
//
// `libview.h` is emitted from `view.php` immediately before swiftc runs. This
// wrapper intentionally redeclares no Elephc ABI. The only adapter handles the
// source export named `dispatch`, which collides with Swift's Dispatch module;
// its C body is type-checked against the generated declaration.

#ifndef ELEPHC_VIEW_SWIFT_ABI_H
#define ELEPHC_VIEW_SWIFT_ABI_H

#include "libview.h"

static inline int32_t elephc_dispatch(
    const char *action_ptr,
    size_t action_len,
    char **output_ptr,
    size_t *output_len
) {
    return dispatch(action_ptr, action_len, output_ptr, output_len);
}

#endif /* ELEPHC_VIEW_SWIFT_ABI_H */
