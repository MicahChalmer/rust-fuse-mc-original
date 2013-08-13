# FUSE (Filesystem In Userspace) Bindings for Rust

This is an interface to write a [FUSE](http://fuse.sourceforge.net/) filesystem in [rust](http://www.rust-lang.org/).

# WORK IN PROGRESS

At this point all you can do with this is compile and run a rust version of the "hello world" filesystem that comes with FUSE as a tutorial.  At least, you can do that on MY machine.  I don't know if it works anywhere else.

Only the few functions needed for hello_fs are even implemented at this point.

As per some helpful discussion on the Rust mailing list, the high level FUSE API is pretty much incompatible with rust as it currently stands.  Rust's std I/O does not work when called from a thread that wasn't started as a rust task.  So it'll have to be the low-level API from the get-go.  New plan:

  1. Switch to low-level API and write another "hello world" based on hello_ll.c from FUSE
  2. Try to cover the rest of the API.  Make a very thin wrapper, just enough to be useful without unsafe blocks.  Create a test FS that just puts files into its own in-memory data structures using that, and get it to pass some filesystem tests.
  3. Change/wrap the thin wrapper with something that is more rust-y.  Try to get rust's type system to enforce the constraints that in the FUSE documentation are just comments.


This is a curiosity project for me.  No actual need to use it is motivating me to develop it.  My only motivation was curiosity about both Rust and FUSE--by developing an interface between them I figured I could learn about both.  Consider yourself warned.

Calling through C function pointers still doesn't work (see https://github.com/mozilla/rust/issues/6194 and https://github.com/mozilla/rust/issues/3678).  This makes it necessary to use my own C shim to be able to call a function pointer that fuse passes us.  I'm not particularly concerned with this, because it is a temporary stopgap--once Rust fixes up its FFI to be able to call C functions through C function pointers it will no longer be necessary.

# BUILDING

To build the C shim, run `make` inside the `wrapper` directory.  I did not bother trying to make rustpkg do this, so you have to do it yourself before building the rust code.

Build the rust code with [rustpkg](https://github.com/mozilla/rust/blob/master/doc/rustpkg.md).  `rust_fuse` is the interface library and `hello_fs` is the aforementioned "hello world" filesystem that uses it.

I'm using the nightly builds of Rust as pulled from the [Ubuntu PPA](https://launchpad.net/%7Ehansjorg/+archive/rust), which tracks the `master` branch of rust.
