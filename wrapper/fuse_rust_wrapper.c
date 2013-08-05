#include <fuse.h>
#include <stdio.h>

int call_filler_function(fuse_fill_dir_t filler, void *buf, const char *name,
                         const struct stat *stbuf, off_t off) {
  return filler(buf, name, stbuf, off);
}

void test_argc_argv(int argc, const char** argv) {
  int i;
  printf("%d arguments\n", argc);
  for(i=0; i<argc; ++i) {
    printf("%s\n", argv[i]);
  }
}
