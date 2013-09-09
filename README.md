# FUSE (Filesystem In Userspace) Bindings for Rust

This is an interface to write a [FUSE](http://fuse.sourceforge.net/) filesystem in [rust](http://www.rust-lang.org/).

# WARNINGS

## WORK IN PROGRESS

Like Rust itself, this is a work in progress.  As of now it has bindings for the FUSE low-level API, and you can run a "hello FS" based directly on the `example/hello_ll.c` that exists in the FUSE source.  At least, you can on MY machine.  I don't know if it works anywhere else.

## QUESTIONABLE PROJECT LIFE

This is a curiosity project for me.  No actual need to use it is driving me to develop it.  My only motivation was curiosity about both Rust and FUSE--by developing an interface between them I figured I could learn about both.  As such, I may or may not stay interested enough to keep updating it until Rust has a stable release.  Consider yourself warned.

# GUIDE

The modules:

  * `rust_fuse` - This is the overall package--nothing exists here directly at this point, other than the submodules
  * `rust_fuse::ffi` - The actual C headers, translated to rust extern fns.  Not meant for direct use.
  * `rust_fuse::lowlevel` - This is a "thin" rust wrapper over the FUSE low level C API.  The goals:
    * Eliminate the need for a user of this library to use unsafe code.  That means converting all raw pointers to vectors, borrowed pointers, etc as appropriate.
    * Use rust's task system to:
      * Run each filesystem request in its own task to allow them to run in parallel.
      * Guarantee that each "request" call receives an appropriate reply, without having to track it yourself.

# PROBLEMS

There are some problems with it as it exists now:

  * That unfortunate struct of `Option<fn...>`s.  It really should be a trait...but I can't see how, at least not without resorting to even worse hacks than what I ended up with.
  * If the filesystem ops tasks fail, no reply is generated.  It should be able to come back with an error.
  * Something is screwy in `fuse_main` with how it handles signals.  Unlike the fuse hello_ll example, here if you SIGINT the process it doesn't unmount and die right away.  It waits until something tries to access the filesystem again, then produces an error (`Software caused connection abort`) and then dies.
  * As of now it spawns a thread for every new task it creates.  It shouldn't do that...it should spawn one for the blocking C calls, then let the default scheduler do the rest

# MISSING PIECES

If I were going to publish this for actual use, it would need:

  1. Documentation
  2. A test suite
  3. A higher-level abstraction over the lowlevel API, similar to FUSE's high-level API but taking full advantage of Rust's features.
    * Unfortunately, FUSE's high-level C API is no help with this.  Rust's task system doesn't play nicely with having a C API spawn its own threads and then try to call back into rust code from them, which is what the FUSE high-level API tries to do.  You can tell it to run single threaded, but that forces all filesystem operations to run serially, since the high-level FUSE API makes a synchronous call to your callback and replies when you return.

The first two are more important for real world use.  The third is more fun.  Guess which one I'm going to do next...;-)

# BUILDING

Build the rust code with [rustpkg](https://github.com/mozilla/rust/blob/master/doc/rustpkg.md).  `rust_fuse` is the interface library and `hello_fs` is the aforementioned "hello world" filesystem that uses it.

To build and run the "hello FS" example and see the result:
  1. Install FUSE and rust on your system.  Other sources can say how to do this better than I can.
  2. `rustpkg install hello_fs` to build the hello_fs binary in this source tree
  3. Run `./bin/hello_fs` and pass the directory you want to mount.  For example:

````
$ mkdir /tmp/hello_fs
$ ./bin/hello_fs /tmp/hello_fs &
[1] 5835
fuse: warning: library too old, some operations may not work
$ ls -laF /tmp/hello_fs
total 4
drwxr-xr-x 2 root root    0 Dec 31  1969 ./
drwxrwxrwt 8 root root 4096 Aug 27 01:07 ../
-r--r--r-- 1 root root   19 Dec 31  1969 hello_from_rust
$ cat /tmp/hello_fs/hello_from_rust 
Hello rusty world!
$ fusermount -u /tmp/hello_fs 
[1]+  Done                    ./bin/hello_fs /tmp/hello_fs
````

I'm using the nightly builds of Rust as pulled from the [Ubuntu PPA](https://launchpad.net/%7Ehansjorg/+archive/rust), which tracks the `master` branch of rust.
