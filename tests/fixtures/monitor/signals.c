/* A CPU-bound tracee whose signal handlers report delivery through stdout. */
#include <signal.h>
#include <unistd.h>
#include <sys/prctl.h>

static void received(int signal) {
    const char marker = signal == SIGTRAP ? 'T' : 'U';
    (void)write(STDOUT_FILENO, &marker, 1);
}

int main(void) {
    struct sigaction action = {0};
    action.sa_handler = received;
    sigemptyset(&action.sa_mask);
    sigaction(SIGUSR1, &action, 0);
    sigaction(SIGTRAP, &action, 0);
    prctl(PR_SET_PTRACER, PR_SET_PTRACER_ANY, 0, 0, 0);
    (void)write(STDOUT_FILENO, "R", 1);
    for (;;) {
        __asm__ volatile("" ::: "memory");
    }
}
