SRC = fuse_rust_wrapper.c

.PHONY: libs dirs

build/%.o: %.c
	$(CC) -c $(shell pkg-config --cflags fuse) -o $@ $<

DIRS = lib build

OBJECTS=$(SRC:%.c=build/%.o)

libs: dirs lib/libfuserustwrapper.a

dirs: $(DIRS)

$(DIRS):
	mkdir -p $@

lib/libfuserustwrapper.a: $(OBJECTS)
	ar cr $@ $<
