/* Probe: X11 + EGL + dlopen inside the snap. */
#include <dlfcn.h>
#include <EGL/egl.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <X11/Xlib.h>

int main(void)
{
    printf("SNAP=%s\n", getenv("SNAP"));
    printf("DISPLAY=%s XAUTHORITY=%s\n", getenv("DISPLAY"), getenv("XAUTHORITY"));

    Display *d = XOpenDisplay(NULL);
    printf("XOpenDisplay: %p\n", (void *)d);
    if (d)
        XCloseDisplay(d);

    void *egl = dlopen("libEGL.so.1", RTLD_NOW | RTLD_GLOBAL);
    printf("dlopen libEGL.so.1: %p (%s)\n", egl, egl ? "ok" : dlerror());
    void *mesa = dlopen("libEGL_mesa.so.0", RTLD_NOW | RTLD_GLOBAL);
    printf("dlopen libEGL_mesa.so.0: %p (%s)\n", mesa, mesa ? "ok" : dlerror());

    typedef EGLAPI EGLDisplay (EGLAPI *getdisp_fn)(EGLNativeDisplayType);
    getdisp_fn mesa_get = (getdisp_fn)dlsym(mesa, "__eglGetDisplay");
    if (mesa_get) {
        EGLDisplay me = mesa_get(EGL_DEFAULT_DISPLAY);
        printf("mesa eglGetDisplay: %p (err 0x%x)\n", (void *)me, eglGetError());
    }

    EGLDisplay e = eglGetDisplay(EGL_DEFAULT_DISPLAY);
    printf("eglGetDisplay: %p (err 0x%x)\n", (void *)e, eglGetError());
    return 0;
}
