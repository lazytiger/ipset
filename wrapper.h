#include <libipset/ipset.h>

#define IPSET_RESTORE_ERROR_CAPACITY 1024

struct ipset_restore_context {
    int failed;
    char message[IPSET_RESTORE_ERROR_CAPACITY];
};

extern int print_out(struct ipset_session *session, void *p, const char *fmt, ...);
extern int ipset_restore_with_error_capture(struct ipset *ipset, FILE *file,
                                            struct ipset_restore_context *context);
