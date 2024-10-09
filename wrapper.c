#include <stdlib.h>
#include <stdio.h>
#include <stdarg.h>
#include <stdint.h>
#include <libipset/ipset.h>

extern void ipset_out(void *p, const char *output, uint32_t len, uint32_t cap);

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
        int n = vsnprintf(data, length, fmt, args);
        va_end(args);
        if(n<0) {
            free(data);
            return n;
        }
        else if (n <= length) {
            ipset_out(p, data, n, length);
            running = 0;
            length = n;
        } else {
            length = n;
            free(data);
        }
    } while (running);
    return length;
}