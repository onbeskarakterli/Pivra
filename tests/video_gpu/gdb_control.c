/* Debugger control only. EGL-named stubs do not render pixels or emulate a GPU. */
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/resource.h>
__attribute__((noinline)) unsigned int eglInitialize(void *display, int *major, int *minor) {
  (void)display; (void)major; (void)minor; return 1;
}
__attribute__((noinline)) void *eglCreateContext(void *display, void *config, void *share_context, const int *attributes) {
  (void)display; (void)config; (void)share_context; (void)attributes;
  return (void *)(uintptr_t)0x2222;
}
__attribute__((noinline)) unsigned int eglDestroyContext(void *display, void *context) {
  (void)display; (void)context; return 1;
}
__attribute__((noinline)) unsigned int eglTerminate(void *display) { (void)display; return 1; }
int main(int argc, char **argv) {
  struct rlimit core = {0,0}; setrlimit(RLIMIT_CORE,&core);
  void *display=(void *)(uintptr_t)0x1111;
  eglInitialize(display,NULL,NULL);
  void *context=eglCreateContext(display,NULL,NULL,NULL);
  eglDestroyContext(display,context); eglTerminate(display);
  puts("CONTROL_AFTER_INTENTIONAL_BREAKPOINTS"); fflush(stdout);
  if (argc>1 && !strcmp(argv[1],"abort")) raise(SIGABRT);
  if (argc>1 && !strcmp(argv[1],"trap")) raise(SIGTRAP);
  if (argc>1 && !strcmp(argv[1],"exit255")) return 255;
  return argc>1 && !strcmp(argv[1],"exit7") ? 7 : 0;
}
