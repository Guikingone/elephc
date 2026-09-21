#include <stdint.h>
#include <stddef.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>
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
    pcre2_code *code;
    pcre2_match_data *match_data;
    PCRE2_SIZE *ovector;
    PCRE2_SIZE subject_length;
    PCRE2_SIZE match_offset;
    uint32_t match_options;
    size_t slots;
    size_t pair_count;
    size_t index;
    int use_startend;
    int64_t start_offset = -1;
    int64_t end_offset = -1;
    int rc;

    if (handle == NULL || subject_z == NULL || (requested_slots != 0 && offset_pairs == NULL) || eflags > INT_MAX) {
        return (int32_t)REG_BADPAT;
    }
    if (requested_slots > SIZE_MAX || requested_slots > SIZE_MAX / (2 * sizeof(int64_t))) {
        return (int32_t)REG_ESPACE;
    }
    slots = (size_t)requested_slots;
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

    /* Match through native pcre2_match() rather than pcre2_regexec(). The POSIX wrapper
       implements REG_STARTEND by ADVANCING the subject pointer -- `pcre2_match(re, string + so,
       eo - so, 0, ...)` -- so every continuation of a global match saw its own start offset as
       the start of the subject, and `^` under the `m` modifier matched THERE. php's own preg
       passes the whole subject with a start offset instead, which is the only way `^` keeps
       meaning "line start". Measured: `preg_replace('/^./m', '    $0', "a\nb\n")` indented every
       character rather than the first of each line, which is exactly how Symfony's
       `CompiledUrlMatcherDumper` writes its routing cache -- the dumped file was unparsable PHP.
       pcre2_match() also reports offsets relative to the whole subject already, which is what
       this ABI's callers expect and what the POSIX wrapper had to re-add by hand. */
    code = (pcre2_code *)handle->regex.re_pcre2_code;
    match_data = (pcre2_match_data *)handle->regex.re_match_data;
    if (code == NULL || match_data == NULL) {
        return (int32_t)REG_BADPAT;
    }
    if (use_startend) {
        subject_length = (PCRE2_SIZE)end_offset;
        match_offset = (PCRE2_SIZE)start_offset;
    } else {
        subject_length = (PCRE2_SIZE)strlen(subject_z);
        match_offset = 0;
    }
    match_options = 0;
    if ((eflags & REG_NOTBOL) != 0) match_options |= PCRE2_NOTBOL;
    if ((eflags & REG_NOTEOL) != 0) match_options |= PCRE2_NOTEOL;
    if ((eflags & REG_NOTEMPTY) != 0) match_options |= PCRE2_NOTEMPTY;
    rc = pcre2_match(code, (PCRE2_SPTR)subject_z, subject_length, match_offset, match_options,
                     match_data, NULL);
    if (rc == PCRE2_ERROR_NOMATCH) {
        return (int32_t)REG_NOMATCH;
    }
    if (rc < 0) {
        return (int32_t)REG_BADPAT;
    }
    ovector = pcre2_get_ovector_pointer(match_data);
    if (ovector == NULL) {
        return (int32_t)REG_BADPAT;
    }
    /* rc == 0 means the ovector was too small to report every capture; the pairs it did fill
       are still valid, and the pattern's own slot count bounds them. */
    pair_count = (rc == 0) ? handle->slot_count : (size_t)rc;
    if (handle->anchored && ovector[0] != match_offset) {
        return (int32_t)REG_NOMATCH;
    }
    for (index = 0; index < slots && index < pair_count; ++index) {
        if (ovector[index * 2] == PCRE2_UNSET) {
            continue;
        }
        offset_pairs[index * 2] = (int64_t)ovector[index * 2];
        offset_pairs[index * 2 + 1] = (int64_t)ovector[index * 2 + 1];
    }
    return 0;
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


/* Returns the name set by the last MARK verb (`(*:name)`) the most recent match passed.
 *
 * PHP exposes this as `$matches['MARK']`, and Symfony's dumped `CompiledUrlMatcher` is built on
 * it: each alternative of the dynamic-route regexp ends in `(*:<offset>)` and the matcher reads
 * the mark to choose `$this->dynamicRoutes[(int) $matches['MARK']]`. Without it that index is 0
 * and every route with a placeholder fails.
 *
 * The caller only invokes this after a SUCCESSFUL match, which is the only case PHP reports a
 * mark for -- pcre2_get_mark() can also report a mark passed before a FAILING match, and PHP
 * does not surface that one.
 */
int32_t elephc_pcre2_v1_last_mark(
    void *opaque_handle,
    const char **mark_out,
    uint64_t *mark_len_out
) {
    elephc_pcre2_v1_handle *handle = (elephc_pcre2_v1_handle *)opaque_handle;
    pcre2_match_data *match_data;
    PCRE2_SPTR mark;
    size_t length;

    if (mark_out != NULL) {
        *mark_out = NULL;
    }
    if (mark_len_out != NULL) {
        *mark_len_out = 0;
    }
    if (handle == NULL || mark_out == NULL || mark_len_out == NULL) {
        return 1;
    }
    match_data = (pcre2_match_data *)handle->regex.re_match_data;
    if (match_data == NULL) {
        return 1;
    }
    mark = pcre2_get_mark(match_data);
    if (mark == NULL) {
        return 1;
    }
    /* pcre2 stores the name NUL-terminated in its own pattern storage, which outlives this call
       for as long as the compiled pattern does -- the same lifetime the name table relies on. */
    for (length = 0; mark[length] != '\0'; ++length) {
    }
    *mark_out = (const char *)mark;
    *mark_len_out = (uint64_t)length;
    return 0;
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
