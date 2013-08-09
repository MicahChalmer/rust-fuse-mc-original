#include <fuse.h>
#include <stdio.h>

int call_filler_function(fuse_fill_dir_t filler, void *buf, const char *name,
                         const struct stat *stbuf, off_t off) {
  return filler(buf, name, stbuf, off);
}
