SRC = fuse_rust_wrapper.c
CFLAGS += -fPIC -Wall -Wextra

.PHONY: libs dirs

build/%.o: %.c
	$(CC) $(CFLAGS) -c $(shell pkg-config --cflags fuse) -o $@ $<

DIRS = lib build

OBJECTS=$(SRC:%.c=build/%.o)

libs: dirs lib/libfuserustwrapper.a

dirs: $(DIRS)

$(DIRS):
	mkdir -p $@

lib/libfuserustwrapper.a: $(OBJECTS)
	ar cr $@ $<
