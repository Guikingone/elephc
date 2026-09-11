#include <stdint.h>
#include <stddef.h>
#include <limits.h>
#include <stdlib.h>
#define PCRE2_CODE_UNIT_WIDTH 8
#include <pcre2.h>
#include <pcre2posix.h>

#define ELEPHC_PCRE2_CFLAG_ANCHORED         0x2000U
#define ELEPHC_PCRE2_CFLAG_EXTENDED         0x4000U
#define ELEPHC_PCRE2_CFLAG_DOLLAR_ENDONLY   0x8000U
#define ELEPHC_PCRE2_CFLAG_DUPNAMES         0x10000U
#define ELEPHC_PCRE2_CFLAG_NO_AUTO_CAPTURE  0x20000U

typedef struct elephc_pcre2_v1_handle {
    regex_t regex;
    size_t slot_count;
    int anchored;
} elephc_pcre2_v1_handle;

int32_t elephc_pcre2_v1_compile(
    void **handle_out,
    const char *pattern_z,
    uint32_t cflags,
    uint64_t *match_slot_count_out
) {
    elephc_pcre2_v1_handle *handle;
    uint32_t native_options;
    pcre2_code *code;
    pcre2_match_data *match_data;
    uint32_t capture_count;
    int errorcode;
    PCRE2_SIZE erroffset;

    if (handle_out != NULL) {
        *handle_out = NULL;
    }
    if (match_slot_count_out != NULL) {
        *match_slot_count_out = 0;
    }
    if (handle_out == NULL || match_slot_count_out == NULL || pattern_z == NULL) {
        return (int32_t)REG_BADPAT;
    }

    handle = (elephc_pcre2_v1_handle *)calloc(1, sizeof(*handle));
    if (handle == NULL) {
        return (int32_t)REG_ESPACE;
    }
    handle->anchored = (cflags & ELEPHC_PCRE2_CFLAG_ANCHORED) != 0;

    /* Compile through native pcre2_compile() instead of delegating to
       pcre2_regcomp(). The POSIX wrapper's fixed REG_* cflag set has no room
       for PHP's x/D/J/n modifiers: none of PCRE2_EXTENDED, PCRE2_DOLLAR_ENDONLY,
       PCRE2_DUPNAMES, or PCRE2_NO_AUTO_CAPTURE map through any bit
       pcre2posix.h defines (confirmed against the installed header: only
       CASELESS/MULTILINE/DOTALL/UTF/UNGREEDY/UCP/LITERAL have POSIX bits, and
       REG_NOSUB does NOT map to PCRE2_NO_AUTO_CAPTURE — empirically it forces
       regexec() to report zero match positions instead, which would have
       silently broken $matches[0]). Translating every existing bit ourselves
       and wrapping the result in the same regex_t shape pcre2_regexec() and
       pcre2_regfree() already expect keeps every previously-supported
       modifier's target PCRE2_* option identical to what pcre2_regcomp() set,
       while adding the four PHP modifiers Symfony's compiled routes need. */
    native_options = 0;
    if ((cflags & REG_ICASE) != 0) native_options |= PCRE2_CASELESS;
    if ((cflags & REG_NEWLINE) != 0) native_options |= PCRE2_MULTILINE;
    if ((cflags & REG_DOTALL) != 0) native_options |= PCRE2_DOTALL;
    if ((cflags & REG_UTF) != 0) native_options |= PCRE2_UTF;
    if ((cflags & REG_UNGREEDY) != 0) native_options |= PCRE2_UNGREEDY;
    if ((cflags & REG_UCP) != 0) native_options |= PCRE2_UCP;
    if ((cflags & ELEPHC_PCRE2_CFLAG_EXTENDED) != 0) native_options |= PCRE2_EXTENDED;
    if ((cflags & ELEPHC_PCRE2_CFLAG_DOLLAR_ENDONLY) != 0) native_options |= PCRE2_DOLLAR_ENDONLY;
    if ((cflags & ELEPHC_PCRE2_CFLAG_DUPNAMES) != 0) native_options |= PCRE2_DUPNAMES;
    if ((cflags & ELEPHC_PCRE2_CFLAG_NO_AUTO_CAPTURE) != 0) native_options |= PCRE2_NO_AUTO_CAPTURE;

    code = pcre2_compile((PCRE2_SPTR)pattern_z, PCRE2_ZERO_TERMINATED, native_options, &errorcode, &erroffset, NULL);
    if (code == NULL) {
        free(handle);
        return (int32_t)REG_BADPAT;
    }
    match_data = pcre2_match_data_create_from_pattern(code, NULL);
    if (match_data == NULL) {
        pcre2_code_free(code);
        free(handle);
        return (int32_t)REG_ESPACE;
    }
    capture_count = 0;
    if (pcre2_pattern_info(code, PCRE2_INFO_CAPTURECOUNT, &capture_count) != 0) {
        pcre2_match_data_free(match_data);
        pcre2_code_free(code);
        free(handle);
        return (int32_t)REG_ESPACE;
    }

    handle->regex.re_pcre2_code = (void *)code;
    handle->regex.re_match_data = (void *)match_data;
    handle->regex.re_endp = NULL;
    handle->regex.re_nsub = (size_t)capture_count;
    handle->regex.re_erroffset = 0;
    handle->regex.re_cflags = 0;

    if (handle->regex.re_nsub == SIZE_MAX) {
        pcre2_regfree(&handle->regex);
        free(handle);
        return (int32_t)REG_ESPACE;
    }
    handle->slot_count = handle->regex.re_nsub + 1;
#if SIZE_MAX > UINT64_MAX
    if (handle->slot_count > UINT64_MAX) {
        pcre2_regfree(&handle->regex);
        free(handle);
        return (int32_t)REG_ESPACE;
    }
#endif
    *match_slot_count_out = (uint64_t)handle->slot_count;
    *handle_out = handle;
    return 0;
}

int32_t elephc_pcre2_v1_exec(
    void *opaque_handle,
    const char *subject_z,
    uint64_t requested_slots,
    int64_t *offset_pairs,
    uint32_t eflags
) {
    elephc_pcre2_v1_handle *handle = (elephc_pcre2_v1_handle *)opaque_handle;
    regmatch_t *matches = NULL;
    size_t slots;
    size_t effective_slots;
    size_t index;
    int use_startend;
    int64_t start_offset = -1;
    int64_t end_offset = -1;
    int result;

    if (handle == NULL || subject_z == NULL || (requested_slots != 0 && offset_pairs == NULL) || eflags > INT_MAX) {
        return (int32_t)REG_BADPAT;
    }
    if (requested_slots > SIZE_MAX || requested_slots > SIZE_MAX / sizeof(regmatch_t)
        || requested_slots > SIZE_MAX / (2 * sizeof(int64_t))) {
        return (int32_t)REG_ESPACE;
    }
    slots = (size_t)requested_slots;
    effective_slots = slots < handle->slot_count ? slots : handle->slot_count;
    if (handle->anchored && effective_slots == 0) {
        effective_slots = 1;
    }
    use_startend = (eflags & REG_STARTEND) != 0 && slots != 0;
    if (use_startend) {
        start_offset = offset_pairs[0];
        end_offset = offset_pairs[1];
        if (start_offset < 0 || end_offset < start_offset
            || start_offset > INT_MAX || end_offset > INT_MAX) {
            return (int32_t)REG_BADPAT;
        }
    }
    for (index = 0; index < slots; ++index) {
        offset_pairs[index * 2] = -1;
        offset_pairs[index * 2 + 1] = -1;
    }
    if (effective_slots != 0) {
        matches = (regmatch_t *)malloc(effective_slots * sizeof(*matches));
        if (matches == NULL) {
            return (int32_t)REG_ESPACE;
        }
        for (index = 0; index < effective_slots; ++index) {
            matches[index].rm_so = -1;
            matches[index].rm_eo = -1;
        }
        if (use_startend) {
            matches[0].rm_so = (regoff_t)start_offset;
            matches[0].rm_eo = (regoff_t)end_offset;
        }
    }
    result = pcre2_regexec(&handle->regex, subject_z, effective_slots, matches, (int)eflags);
    if (result == 0 && handle->anchored
        && matches[0].rm_so != (regoff_t)(use_startend ? start_offset : 0)) {
        result = REG_NOMATCH;
    }
    if (result == 0) {
        for (index = 0; index < effective_slots && index < slots; ++index) {
            offset_pairs[index * 2] = (int64_t)matches[index].rm_so;
            offset_pairs[index * 2 + 1] = (int64_t)matches[index].rm_eo;
        }
    }
    free(matches);
    return (int32_t)result;
}

void elephc_pcre2_v1_free(void *opaque_handle) {
    elephc_pcre2_v1_handle *handle = (elephc_pcre2_v1_handle *)opaque_handle;
    if (handle == NULL) {
        return;
    }
    pcre2_regfree(&handle->regex);
    free(handle);
}

/* Reads the compiled pattern out of the POSIX wrapper without exposing PCRE2 layouts to callers. */
static const pcre2_code *elephc_pcre2_v1_code(void *opaque_handle) {
    const elephc_pcre2_v1_handle *handle = (const elephc_pcre2_v1_handle *)opaque_handle;

    if (handle == NULL) {
        return NULL;
    }
    return (const pcre2_code *)handle->regex.re_pcre2_code;
}

uint64_t elephc_pcre2_v1_name_count(void *opaque_handle) {
    const pcre2_code *code = elephc_pcre2_v1_code(opaque_handle);
    uint32_t count = 0;

    if (code == NULL) {
        return 0;
    }
    if (pcre2_pattern_info(code, PCRE2_INFO_NAMECOUNT, &count) != 0) {
        return 0;
    }
    return (uint64_t)count;
}

int32_t elephc_pcre2_v1_group_name(
    void *opaque_handle,
    uint64_t group,
    const char **name_out,
    uint64_t *name_len_out
) {
    const pcre2_code *code = elephc_pcre2_v1_code(opaque_handle);
    PCRE2_SPTR table = NULL;
    uint32_t count = 0;
    uint32_t entry_size = 0;
    uint32_t index;

    if (name_out != NULL) {
        *name_out = NULL;
    }
    if (name_len_out != NULL) {
        *name_len_out = 0;
    }
    if (code == NULL || name_out == NULL || name_len_out == NULL || group > 0xFFFFU) {
        return 1;
    }
    if (pcre2_pattern_info(code, PCRE2_INFO_NAMECOUNT, &count) != 0
        || pcre2_pattern_info(code, PCRE2_INFO_NAMEENTRYSIZE, &entry_size) != 0
        || pcre2_pattern_info(code, PCRE2_INFO_NAMETABLE, &table) != 0) {
        return 1;
    }
    if (count == 0 || entry_size < 3 || table == NULL) {
        return 1;
    }
    for (index = 0; index < count; ++index) {
        const unsigned char *entry = (const unsigned char *)table + (size_t)index * (size_t)entry_size;
        uint32_t entry_group = ((uint32_t)entry[0] << 8) | (uint32_t)entry[1];
        const char *name;
        size_t limit;
        size_t length;

        if (entry_group != (uint32_t)group) {
            continue;
        }
        /* Each entry is a 2-byte group number then a NUL-terminated name, so the name can never
           run past the entry. The bound is measured rather than assumed: strnlen() is POSIX, and
           this shim is compiled as strict C11. */
        name = (const char *)(entry + 2);
        limit = (size_t)entry_size - 2;
        for (length = 0; length < limit && name[length] != '\0'; ++length) {
        }
        *name_out = name;
        *name_len_out = (uint64_t)length;
        return 0;
    }
    return 1;
}
