/* Namespace and X socket probe inside the snap. Usage: nsprobe [display-digit] */
#include <dirent.h>
#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

static void list_dir(const char *dir)
{
    printf("%s:\n", dir);
    struct dirent *e;
    DIR *d = opendir(dir);
    if (!d) {
        printf("  (cannot open: %s)\n", strerror(errno));
        return;
    }
    while ((e = readdir(d)))
        printf("  %s\n", e->d_name);
    closedir(d);
}

int main(int argc, char **argv)
{
    char buf[256];
    ssize_t n;

    n = readlink("/proc/self/ns/mnt", buf, sizeof buf - 1);
    if (n > 0) { buf[n] = 0; printf("mnt ns: %s\n", buf); }
    n = readlink("/proc/self/ns/net", buf, sizeof buf - 1);
    if (n > 0) { buf[n] = 0; printf("net ns: %s\n", buf); }

    FILE *f = fopen("/proc/self/mountinfo", "r");
    if (f) {
        char line[1024];
        while (fgets(line, sizeof line, f))
            if (strstr(line, " /tmp ") || strstr(line, " /dev/shm "))
                fputs(line, stdout);
        fclose(f);
    }

    list_dir("/tmp/.X11-unix");

    if (argc > 1) {
        int d = argv[1][0] - '0';
        char path[64];
        snprintf(path, sizeof path, "X%d", d);

        struct sockaddr_un a;
        memset(&a, 0, sizeof a);
        a.sun_family = AF_UNIX;
        snprintf(a.sun_path, sizeof a.sun_path, "/tmp/.X11-unix/X%d", d);
        int s = socket(AF_UNIX, SOCK_STREAM, 0);
        int r = connect(s, (struct sockaddr *)&a, sizeof a);
        printf("pathname /tmp/.X11-unix/X%d: %s\n", d, r ? "FAIL" : "OK");

        struct sockaddr_un ab;
        memset(&ab, 0, sizeof ab);
        ab.sun_family = AF_UNIX;
        const char *name = "/tmp/.X11-unix/X";
        char tail[2] = { (char) '0' + d, 0 };
        size_t len = strlen(name) + 1;
        memcpy(ab.sun_path + 1, name, len);
        ab.sun_path[len] = tail[0];
        int s2 = socket(AF_UNIX, SOCK_STREAM, 0);
        r = connect(s2, (struct sockaddr *)&ab, offsetof(struct sockaddr_un, sun_path) + len + 1);
        printf("abstract  @/tmp/.X11-unix/X%d: %s\n", d, r ? "FAIL" : "OK");
    }
    return 0;
}
