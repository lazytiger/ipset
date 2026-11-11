#include <stdlib.h>
#include <stdio.h>
#include <stdarg.h>
#include <stdint.h>
#include <libipset/ipset.h>
#include "wrapper.h"

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
        else if (n < length) {
            ipset_out(p, data, n, length);
            running = 0;
            length = n;
        } else {
            length = n + 1;
            free(data);
        }
    } while (running);
    return length;
}

static void restore_capture_message(struct ipset_restore_context *context,
                                    const char *message) {
    if (context->failed) {
        return;
    }

    context->failed = 1;
    snprintf(context->message, sizeof(context->message), "%s",
             message != NULL ? message : "Failed to restore ipset");
}

static int restore_capture_custom_error(struct ipset *ipset, void *p,
                                        int status, const char *fmt, ...) {
    (void) ipset;
    (void) status;

    struct ipset_restore_context *context = p;
    if (!context->failed) {
        if (fmt != NULL) {
            va_list args;
            va_start(args, fmt);
            vsnprintf(context->message, sizeof(context->message), fmt, args);
            va_end(args);
        } else {
            snprintf(context->message, sizeof(context->message),
                     "Failed to restore ipset");
        }
        context->failed = 1;
    }
    return -1;
}

static int restore_capture_standard_error(struct ipset *ipset, void *p) {
    struct ipset_session *session = ipset_session(ipset);
    enum ipset_err_type type = ipset_session_report_type(session);

    if (type == IPSET_ERROR) {
        restore_capture_message(p, ipset_session_report_msg(session));
    }
    ipset_session_report_reset(session);
    return type == IPSET_ERROR ? -1 : 0;
}

int ipset_restore_with_error_capture(struct ipset *ipset, FILE *file,
                                     struct ipset_restore_context *context) {
    context->failed = 0;
    context->message[0] = '\0';

    if (ipset_custom_printf(ipset, restore_capture_custom_error,
                            restore_capture_standard_error, NULL,
                            context) < 0) {
        return -1;
    }

    int ret = ipset_parse_stream(ipset, file);
    ipset_custom_printf(ipset, NULL, NULL, NULL, NULL);

    if (context->failed || ret < 0) {
        if (context->message[0] == '\0') {
            restore_capture_message(context, "Failed to restore ipset");
        }
        return -1;
    }
    return 0;
}
