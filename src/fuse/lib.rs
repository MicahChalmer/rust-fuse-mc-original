#[link(name = "fuse",
vers = "0.0.0",
uuid = "d37c5c30-fdcd-459d-bfca-ebb8da04b2a0",
url = "https://github.com/MicahChalmer/rust-fuse")];

#[comment = "FUSE bindings"];
#[license = "MIT"];
#[crate_type = "lib"];

#[link_args = "-lfuse"] 

use std::libc::{
    c_char,
    c_int,
    c_uint,
    c_ulong,
    c_void,
    off_t,
    size_t,
    stat
};

// A cfuncptr is used here to stand in for a C function pointer.
// For this struct, see fuse.h
type cfuncptr = *u8;
struct c_fuse_operations {
    getattr: cfuncptr,
    readlink: cfuncptr,
    getdir: cfuncptr,
    mknod: cfuncptr,
    mkdir: cfuncptr,
    unlink: cfuncptr,
    rmdir: cfuncptr,
    symlink: cfuncptr,
    rename: cfuncptr,
    link: cfuncptr,
    chmod: cfuncptr,
    chown: cfuncptr,
    truncate: cfuncptr,
    utime: cfuncptr,
    open: cfuncptr,
    read: cfuncptr,
    write: cfuncptr,
    statfs: cfuncptr,
    flush: cfuncptr,
    release: cfuncptr,
    fsync: cfuncptr,
    setxattr: cfuncptr,
    getxattr: cfuncptr,
    listxattr: cfuncptr,
    removexattr: cfuncptr,
    opendir: cfuncptr,
    readdir: cfuncptr,
    releasedir: cfuncptr,
    fsyncdir: cfuncptr,
    init: cfuncptr,
    destroy: cfuncptr,
    access: cfuncptr,
    create: cfuncptr,
    ftruncate: cfuncptr,
    fgetattr: cfuncptr,
    lock: cfuncptr,
    utimens: cfuncptr,
    bmap: cfuncptr,

    flag_nullpath_ok: uint,
    flag_nopath: uint,
    flag_utime_omit_ok: uint,
    flag_reserved: uint,

    ioctl: cfuncptr,
    poll: cfuncptr,
    write_buf: cfuncptr,
    read_buf: cfuncptr,
    flock: cfuncptr
}

pub struct fuse_file_info {
    flags: c_int,
    fh_old: c_ulong, // Old file handle, don't use
    writepage: c_int,
    direct_io: c_uint,
    keep_cache: c_uint,
    flush: c_uint,
    nonseekable: c_uint,
    flock_release: c_uint,
    padding: c_uint, // Padding.  Do not use
    fh: u64,
    lock_owner: u64
}

extern {
    fn fuse_main_real(argc:c_int, argv:**c_char, 
                      op:*c_fuse_operations, op_size: size_t,
                      user_data: *c_void);
}

// Used for return values from FS operations
type errno = int;

pub struct dir_entry {
    name: ~str,
    stat: Option<stat>,
    off: off_t
}

pub trait FuseOperations {
    fn getattr(path:&str) -> Either<stat, errno>;
    fn readdir(path:&str, info: &fuse_file_info) -> (errno, ~[~dir_entry]);
    fn open(path:&str, info: &mut fuse_file_info) -> errno;  // TODO: don't allow mutation of the whole fuse_file_info
    fn read(path:&str, buf:&mut [u8], size: size_t, offset: off_t, info: &fuse_file_info) -> (errno, size_t);
}

pub fn fuse_main<T: FuseOperations>(args: ~[~str], ops: ~T) {

}
