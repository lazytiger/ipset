#include <stdlib.h>
#include <stdio.h>
#include <stdarg.h>
#include <libipset/ipset.h>

extern void ipset_out(void *p, const char *output);

int print_out(struct ipset_session *session, void *p, const char *fmt, ...) {
    (void) session;
    va_list args;
    int length = 1024;
    int running = 1;
    do {
        char *data = malloc(length);
        if (data == NULL) {
            return 0;
        }
        va_start(args, fmt);
        int n = vsnprintf(data, length - 1, fmt, args);
        va_end(args);
        if (n <= length - 1) {
            data[n] = 0;
            ipset_out(p, data);
            running = 0;
            length = n;
        } else {
            length = n + 1;
        }
        free(data);
    } while (running);
    return length;
}