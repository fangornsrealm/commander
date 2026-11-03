/* Debug probe: check xkb data visibility inside the snap. */
#include <stdio.h>
#include <unistd.h>
#include <string.h>

static void probe_line(const char *path, int *res)
{
    FILE *f = fopen("/tmp/xkb-probe.txt", "a");
    int r = access(path, 0);
    *res = r;
    fprintf(stderr, "[probe] access(%s) = %d\n", path, r);
    if (f) {
        fprintf(f, "access %s = %d\n", path, r);
        fclose(f);
    }
}

__attribute__((constructor))
static void probe(void)
{
    /* also record whether the binary is the snap one */
    fprintf(stderr, "[probe] running, argv0 env SNAP=%s\n", getenv("SNAP"));
    int r;
    probe_line("/usr/share/X11/xkb", &r);
    probe_line("/usr/share/X11/xkb/symbols", &r);
    probe_line("/usr/share/X11/xkb/rules", &r);
    probe_line("/usr/share/x11/xkb", &r);
    fflush(stderr);
}
