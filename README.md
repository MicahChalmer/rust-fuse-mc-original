# FUSE (Filesystem In Userspace) Bindings for Rust

This is an interface to write a [FUSE](http://fuse.sourceforge.net/) filesystem in [rust](http://www.rust-lang.org/).

# WORK IN PROGRESS

At this point all you can do with this is compile and run a rust version of the "hello world" filesystem that comes with FUSE as a tutorial.  At least, you can do that on MY machine.  I don't know if it works anywhere else.

Only the few functions needed for hello_fs are even implemented at this point.  A rough sketch of what would be needed to make this actually useful:

  1. Finish covering the high-level API (i.e. what's available from `fuse.h`)
  2. Make the API more in the spirit of Rust--don't pass around mutable references to whole structures just to look for changes to one field, etc.
  3. A test suite would be needed.  Maybe something based on Vagrant or Docker so it doesn't need a lot of setup on the machine just to run...
  4. Cover the low-level API with all of the above

This is a curiosity project for me.  No actual need to use it is motivating me to develop it.  My only motivation was curiosity about both Rust and FUSE--by developing an interface between them I figured I could learn about both.  Consider yourself warned.

Calling through C function pointers still doesn't work (see mozilla/rust#6194 and mozilla/rust#3678).  This makes it necessary to use my own C shim to be able to call a function pointer that fuse passes us.  I'm not particularly concerned with this as it is a temporary stopgap--once Rust fixes up its FFI to be able to call C functions through C function pointers it will no longer be necessary.

# BUILDING

To build the C shim, run `make` inside the `wrapper` directory.  I did not bother trying to make rustpkg do this, so you have to do it yourself before building the rust code.

Build the rust code with [rustpkg](https://github.com/mozilla/rust/blob/master/doc/rustpkg.md).  `rust_fuse` is the interface library and `hello_fs` is the aforementioned "hello world" filesystem that uses it.

I'm using the nightly builds of Rust as pulled from the [Ubuntu PPA](https://launchpad.net/%7Ehansjorg/+archive/rust), which tracks the `master` branch of rust.
