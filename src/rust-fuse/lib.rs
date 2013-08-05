#[link(name = "fuse",
uuid = "d37c5c30-fdcd-459d-bfca-ebb8da04b2a0",
url = "https://github.com/MicahChalmer/rust-fuse")];

#[comment = "FUSE bindings"];
#[license = "MIT"];
#[crate_type = "lib"];

use std::libc::{
    c_char,
    c_int,
    c_uint,
    c_ulong,
    c_void,
    mode_t,
    off_t,
    pid_t,
    size_t,
    stat,
    uid_t,
    gid_t
};

use std::ptr;
use std::str;
use std::vec;
use std::sys::size_of;

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

// TODO: this should not be public
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

struct c_fuse_context {
    fuse: *c_void,
    uid: uid_t,
    gid: gid_t,
    pid: pid_t,
    private_data: *rust_fuse_data,  // we use this to know what object to call
    umask: mode_t 
}

struct rust_fuse_data {
    ops: ~FuseOperations
}

extern {
    fn fuse_main_real(argc:c_int, argv:**c_char, 
                      op:*c_fuse_operations, op_size: size_t,
                      user_data: *c_void) -> c_int;

    fn fuse_get_context() -> *c_fuse_context;

    // Workaround for the fact that we can't call into c via a function ptr right
    // from rust
    fn call_filler_function(filler: cfuncptr, buf: *c_void, name: *c_char, stbuf: *stat,
                           off: off_t) -> c_int;
}

// Used for return values from FS operations
pub type errno = c_int;

pub type fuse_fill_dir_func<'self> = &'self fn (&str, Option<stat>, off_t) -> c_int;

pub type filehandle = u64;

pub struct dir_entry {
    name: ~str,
    stat: Option<stat>,
    off: off_t
}

#[deriving(Clone, Eq, ToStr)]
pub enum ErrorOrResult<E, T> {
    Error(E),
    Result(T)
}

pub trait FuseOperations {
    fn getattr(&self, path:&str) -> ErrorOrResult<errno, stat>;
    fn readdir(&self, path:&str, filler: fuse_fill_dir_func,
               offset: off_t, info: &fuse_file_info) -> ErrorOrResult<errno, ()>;
    fn open(&self, path:&str, info: &fuse_file_info) -> ErrorOrResult<errno, filehandle>;
    fn read(&self, path:&str, buf:&mut [u8], size: size_t, offset: off_t,
            info: &fuse_file_info) -> ErrorOrResult<errno, c_int>;
}

unsafe fn get_context_ops() -> &~FuseOperations {
    &((*(*fuse_get_context()).private_data).ops)
}

extern fn c_getattr(path: *c_char, stbuf: *mut stat) -> errno {
    unsafe {
        let ops = get_context_ops();
        ptr::zero_memory(stbuf, 1);
        match ops.getattr(str::raw::from_c_str(path)) {
            Error(e) => -e,
            Result(st) => { ptr::copy_memory(stbuf, &st, 1); 0 }
        }
    }
}

unsafe fn option_to_ptr<T>(opt: Option<T>) -> *T {
    match opt {
        Some(ref t) => ptr::to_unsafe_ptr(t),
        None => ptr::null()
    }
}

extern fn c_readdir(path: *c_char, buf: *c_void, filler: cfuncptr,
                    offset: off_t, fi: *fuse_file_info) -> c_int {
    unsafe {
        let ops = get_context_ops();
        let fill_func: fuse_fill_dir_func = |name, st, ofs| -> c_int {
            do name.as_c_str |c_name| {
                call_filler_function(filler, buf, c_name, option_to_ptr(st), ofs)
            }
        };
        match ops.readdir(str::raw::from_c_str(path), fill_func, offset, &*fi) {
            Error(e) => -e,
            Result(_) => 0
        }
    }
}

extern fn c_open(path: *c_char, info: *mut fuse_file_info) -> c_int {
    unsafe {
        let ops = get_context_ops();
        match ops.open(str::raw::from_c_str(path), &*info) {
            Error(e) => -e,
            Result(fh) => { (*info).fh = fh; 0 }
        }   
    }
}

extern fn c_read(path: *c_char, buf: *mut u8, size: size_t, offset: off_t,
                 fi: *fuse_file_info) -> c_int {
    unsafe {
        let ops = get_context_ops();
        do vec::raw::mut_buf_as_slice(buf, size as uint) |slice| {
            match ops.read(str::raw::from_c_str(path), slice, size, offset, &*fi) {
                Error(e) => -e,
                Result(sz) => sz
            }
        }
    }
}

pub fn fuse_main<T: FuseOperations>(args: ~[~str], ops: ~T) -> c_int {
    let cfo = c_fuse_operations {
        getattr: c_getattr,
        readdir: c_readdir,
        open: c_open,
        read: c_read,

        readlink: ptr::null(),
        getdir: ptr::null(),
        mknod: ptr::null(),
        mkdir: ptr::null(),
        unlink: ptr::null(),
        rmdir: ptr::null(),
        symlink: ptr::null(),
        rename: ptr::null(),
        link: ptr::null(),
        chmod: ptr::null(),
        chown: ptr::null(),
        truncate: ptr::null(),
        utime: ptr::null(),
        write: ptr::null(),
        statfs: ptr::null(),
        flush: ptr::null(),
        release: ptr::null(),
        fsync: ptr::null(),
        setxattr: ptr::null(),
        getxattr: ptr::null(),
        listxattr: ptr::null(),
        removexattr: ptr::null(),
        opendir: ptr::null(),
        releasedir: ptr::null(),
        fsyncdir: ptr::null(),
        init: ptr::null(),
        destroy: ptr::null(),
        access: ptr::null(),
        create: ptr::null(),
        ftruncate: ptr::null(),
        fgetattr: ptr::null(),
        lock: ptr::null(),
        utimens: ptr::null(),
        bmap: ptr::null(),

        flag_nullpath_ok: 0,
        flag_nopath: 0,
        flag_utime_omit_ok: 0,
        flag_reserved: 29,

        ioctl: ptr::null(),
        poll: ptr::null(),
        write_buf: ptr::null(),
        read_buf: ptr::null(),
        flock: ptr::null()
    };
    unsafe {
        let arg_c_strs = vec::raw::to_ptr(args.map(|s| vec::raw::to_ptr(s.as_bytes_with_null())));
        fuse_main_real(args.len() as c_int, std::cast::transmute(arg_c_strs), 
                       &cfo, size_of::<c_fuse_operations>() as size_t,
                       std::cast::transmute(&ops))
    }
}
