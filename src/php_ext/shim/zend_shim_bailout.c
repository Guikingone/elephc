/* zend_shim_bailout.c — the Zend fatal-error contract, for a host that is not
 * the Zend VM.
 *
 * PHP's model: a fatal error inside an extension calls zend_bailout(), which
 * longjmps to a JMP_BUF the engine installed at the request boundary. Nothing
 * unwinds; the request-scoped memory manager simply discards everything after.
 * Extensions rely on this — they call zend_bailout() and never return.
 *
 * This is the one contract static linking cannot police: once the symbol
 * exists, whether it unwinds *correctly* is a runtime property, invisible at
 * build time and only reachable on error paths. So it is implemented properly
 * here, and its two known hazards are measured rather than assumed:
 *   1. no bailout handler installed at all;
 *   2. longjmp across a C++ frame — destructors do NOT run.
 */

#include "php.h"
#include "zend_globals.h"
#include <stdio.h>
#include <stdlib.h>

/* Counters the driver reads to check what actually happened. */
int shim_bailouts_taken = 0;
int shim_bailout_handlers_entered = 0;

ZEND_API ZEND_COLD ZEND_NORETURN void _zend_bailout(const char *filename, uint32_t lineno) {
    shim_bailouts_taken++;
    if (!EG(bailout)) {
        /* No protected frame: this is a genuine fatal with nowhere to go.
         * Report it honestly rather than longjmping into an uninitialised
         * buffer — the exact failure the design review flagged as the most
         * dangerous, because it is silent until an error path is hit. */
        fprintf(stderr, "shim: FATAL from extension at %s:%u with no bailout handler installed\n",
                filename, lineno);
        exit(255);
    }
    LONGJMP(*EG(bailout), FAILURE);
}

/* Host-side protected call. Elephc would install one of these at whatever it
 * considers a request boundary (per-connection in --web, per-program in CLI).
 * Returns 1 if the callee completed, 0 if it bailed out. */
int shim_protected_call(void (*fn)(void *), void *arg) {
    JMP_BUF *saved = EG(bailout);
    JMP_BUF here;
    int completed;

    EG(bailout) = &here;
    if (SETJMP(here) == 0) {
        fn(arg);
        completed = 1;
    } else {
        shim_bailout_handlers_entered++;
        completed = 0;
    }
    EG(bailout) = saved;
    return completed;
}
