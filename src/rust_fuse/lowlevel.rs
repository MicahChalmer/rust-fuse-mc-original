use std::libc::{
    c_double,
    c_int,
    c_schar,
    c_uint,
    c_ulong,
    c_void,
    dev_t,
    gid_t,
    mode_t,
    off_t,
    size_t,
    stat,
    time_t,
    uid_t,
};
use std::io::stderr;
use std::sys::size_of;
use std::cast::transmute;
use std::ptr;
use std::vec;
use std::task::{task, SingleThreaded};
use std::c_str::CString;
use std::str;
use std::num::zero;
use std::bool::to_bit;
use std::cmp;
use std::iterator::AdditiveIterator;
use ffi::*;
use std::path::stat::arch::default_stat;
use std::libc::ENOSYS;
pub use ffi::{fuse_ino_t,Struct_fuse_entry_param};

/// Information to be returned from open
#[deriving(Zero)]
pub struct OpenReply {
    direct_io:bool,
    keep_cache:bool,
    fh: u64
}

pub struct CreateReply {
    open_reply: OpenReply,
    entry_param: EntryReply
}

pub struct AttrReply {
    attr: stat,
    attr_timeout: c_double
}

pub enum AttrToSet {
    Mode(mode_t),
    Uid(uid_t),
    Gid(gid_t),
    Size(off_t),
    Atime(time_t),
    Mtime(time_t),

    // TODO: confirm assumption about what these mean
    Atime_now,
    Mtime_now,
}

pub enum ReadReply {
    // TODO: support iov and fuse_bufvec type replies for more efficient (?)
    // types of implementation
    DataBuffer(~[u8]),
    EOF
}

#[deriving(Clone)]
pub struct DirEntry {
    ino: fuse_ino_t,
    name: ~str,
    mode: mode_t,
    next_offset: off_t
}

pub enum ReaddirReply {
    DirEntries(~[DirEntry]),
}

pub type EntryReply = Struct_fuse_entry_param;

/// The error result should be one of libc's errno values
pub type ErrnoResult<T> = Result<T, c_int>;

/**
 * Structure of callbacks to pass into fuse_main.

 * It would be best if a user of this could be a trait, and a user could just
 * implement the methods as desired, leaving the rest as defaults.
 * Unfortunately that's not quite possible.  FUSE has default behavior for some
 * ops that can't just be invoked from a callback--the only way to get it is to
 * pass NULL for the callback pointers into the fuse_lowlevel_ops structure.
 * But we can't know at run time which default methods of a trait were
 * overridden, which means we don't know which entries in the
 * Struct_fuse_lowlevel_ops to null out.

 * Instead, we've got the rust direct equivalent of what the C API has: a struct
 * full of optional fns--equivalent to the struct of nullable function pointers
 * in C.  What you pass into here will look like Rust's imitation of Javascript.
 * But it's the best we can do without some sort of reflection API that rust
 * doesn't have, or a way to call the FUSE default behavior from a callback,
 * which FUSE does not have.

 */
#[deriving(Zero)]
pub struct FuseLowLevelOps<'self> {
    init: Option<&'self fn()>,
    destroy: Option<&'self fn()>,
    lookup: Option<&'self fn(_parent: fuse_ino_t, _name: &str) -> ErrnoResult<EntryReply>>,
    forget: Option<&'self fn(_ino:fuse_ino_t, _nlookup:c_ulong)>,
    getattr: Option<&'self fn(_ino: fuse_ino_t) -> ErrnoResult<AttrReply>>,
    setattr: Option<&'self fn(_ino: fuse_ino_t, _attrs_to_set:&[AttrToSet], _fh:Option<u64>) -> ErrnoResult<AttrReply>>,
    readlink: Option<&'self fn(_ino: fuse_ino_t) -> ErrnoResult<~str>>,
    mknod: Option<&'self fn(_parent: fuse_ino_t, _name: &str, _mode: mode_t, _rdev: dev_t) -> ErrnoResult<EntryReply>>,
    mkdir: Option<&'self fn(_parent: fuse_ino_t, _name: &str, _mode: mode_t) -> ErrnoResult<EntryReply>>,
    // TODO: Using the unit type with result seems kind of goofy, but;
    // is done for consistency with the others.  Is this right?;
    unlink: Option<&'self fn(_parent: fuse_ino_t, _name: &str) -> ErrnoResult<()>>,
    rmdir: Option<&'self fn(_parent: fuse_ino_t, _name: &str) -> ErrnoResult<()>>,
    symlink: Option<&'self fn(_link:&str, _parent: fuse_ino_t, _name: &str) -> ErrnoResult<EntryReply>>,
    rename: Option<&'self fn(_parent: fuse_ino_t, _name: &str, _newparent: fuse_ino_t, _newname: &str) -> ErrnoResult<()>>,
    link: Option<&'self fn(_ino: fuse_ino_t, _newparent: fuse_ino_t, _newname: &str) -> ErrnoResult<EntryReply>>,
    open: Option<&'self fn(_ino: fuse_ino_t, _flags: c_int) -> ErrnoResult<OpenReply>>,
    read: Option<&'self fn(_ino: fuse_ino_t, _size: size_t, _off: off_t, _fh: u64) -> ErrnoResult<ReadReply>>,
    // TODO: is writepage a bool, or an actual number that needs to be;
    // preserved?;
    write: Option<&'self fn(_ino: fuse_ino_t, _buf:&[u8], _off: off_t, _fh: u64, _writepage: bool) -> ErrnoResult<size_t>>,
    flush: Option<&'self fn(_ino: fuse_ino_t, _lock_owner: u64, _fh: u64) -> ErrnoResult<()>>,
    release: Option<&'self fn(_ino: fuse_ino_t, _flags: c_int, _fh: u64) -> ErrnoResult<()>>,
    fsync: Option<&'self fn(_ino: fuse_ino_t, _datasync: bool, _fh: u64) -> ErrnoResult<()>>,
    opendir: Option<&'self fn(_ino: fuse_ino_t) -> ErrnoResult<OpenReply>>,
    readdir: Option<&'self fn(_ino: fuse_ino_t, _size: size_t, _off: off_t, _fh: u64) -> ErrnoResult<ReaddirReply>>,
    releasedir: Option<&'self fn(_ino: fuse_ino_t, _fh: u64) -> ErrnoResult<()>>,
    fsyncdir: Option<&'self fn(_ino: fuse_ino_t, _datasync: bool, _fh: u64) -> ErrnoResult<()>>,
    statfs: Option<&'self fn(_ino: fuse_ino_t) -> ErrnoResult<Struct_statvfs>>,
    setxattr: Option<&'self fn(_ino: fuse_ino_t, _name: &str, _value: &[u8], _flags: c_int) -> ErrnoResult<()>>,
    // TODO: examine this--ReadReply may not be appropraite here;
    getxattr: Option<&'self fn(_ino: fuse_ino_t, _name: &str, _size: size_t) -> ErrnoResult<ReadReply>>,
    // Called on getxattr with size of zero (meaning a query of total size);
    getxattr_size: Option<&'self fn(_ino: fuse_ino_t, _name: &str) -> ErrnoResult<size_t>>,
    // TODO: examine this--ReadReply may not be appropraite here;
    listxattr: Option<&'self fn(_ino: fuse_ino_t, _size: size_t) -> ErrnoResult<ReadReply>>,
    // Called on listxattr with size of zero (meaning a query of total size);
    listxattr_size: Option<&'self fn(_ino: fuse_ino_t) -> ErrnoResult<size_t>>,
    removexattr: Option<&'self fn(_ino: fuse_ino_t, _name: &str) -> ErrnoResult<()>>,
    access: Option<&'self fn(_ino: fuse_ino_t, _mask: c_int) -> ErrnoResult<()>>,
    create: Option<&'self fn(_parent: fuse_ino_t, _name: &str, _mode: mode_t, _flags: c_int) -> ErrnoResult<CreateReply>>,

    // TODO: The following, which didn't even exist in earlier versions of FUSE,
    // can be considered nice-to-have (to an even greater extent than the whole
    // project can at this point):
    //
    // getlk
    // setlk
    // bmap
    // ioctl
    // poll
    // write_buf
    // retrieve_reply
    // forget_multi
    // flock
    // fallocate
}

/*
 * Run a function with a borrowed pointer to the FuseLowLevelOps struct pointed
 * to by the given userdata pointer.  The "arg" parameter is for passing extra
 * data into the function a la task::spawn_with (needed to push owned pointers
 * into the closure)
 */
fn userdata_to_ops<T, U>(userdata:*mut c_void, arg:U,
                         func:&fn(&FuseLowLevelOps, U) -> T) -> T {
    unsafe {
        func(*(userdata as *~FuseLowLevelOps), arg)
    }
}

#[fixed_stack_segment]
pub fn fuse_main(args:~[~str], ops:~FuseLowLevelOps) {
    unsafe {
        let arg_c_strs_ptrs: ~[*c_schar] = args.map(|s| s.to_c_str().unwrap() );
        let mut fuse_args = Struct_fuse_args {
            argv: transmute(vec::raw::to_ptr(arg_c_strs_ptrs)),
            argc: args.len() as c_int,
            allocated: 0
        };
        let mut mountpoint:*mut c_schar = ptr::mut_null();
        if fuse_parse_cmdline(ptr::to_mut_unsafe_ptr(&mut fuse_args),
                              ptr::to_mut_unsafe_ptr(&mut mountpoint),
                              ptr::mut_null(), // multithreaded--we ignore
                              ptr::mut_null() // foreground--we ignore (for now)
                              ) == -1 {
            return;
        }
        
        let fuse_chan = fuse_mount(mountpoint as *c_schar, ptr::to_mut_unsafe_ptr(&mut fuse_args));
        if fuse_chan == ptr::mut_null() {
            // TODO: better error message?
            stderr().write_line("Failed to mount\n");
            fail!();
        }
        
        let llo = make_fuse_ll_oper(ops);
        let fuse_session = fuse_lowlevel_new(ptr::to_mut_unsafe_ptr(&mut fuse_args),
                                             ptr::to_unsafe_ptr(&llo),
                                             size_of::<Struct_fuse_lowlevel_ops>() as size_t,
                                             ptr::to_unsafe_ptr(&ops) as *mut c_void);
        if fuse_session == ptr::mut_null() {
            // TODO: better error message?
            stderr().write_line("Failed to create FUSE session\n");
            fail!();
        }
        
        if fuse_set_signal_handlers(fuse_session) == -1 {
            stderr().write_line("Failed to set FUSE signal handlers");
            fail!();
        }

        fuse_session_add_chan(fuse_session, fuse_chan);
        fuse_session_loop(fuse_session);
        fuse_remove_signal_handlers(fuse_session);
        fuse_session_remove_chan(fuse_chan);

        fuse_session_destroy(fuse_session);
        fuse_unmount(mountpoint as *c_schar, fuse_chan);
        fuse_opt_free_args(ptr::to_mut_unsafe_ptr(&mut fuse_args));
    };
}

pub fn make_fuse_ll_oper(ops:&FuseLowLevelOps)
                         -> Struct_fuse_lowlevel_ops {
    return Struct_fuse_lowlevel_ops {
        init: ops.init.map(|_| init_impl),
        destroy: ops.destroy.map(|_| destroy_impl),
        lookup: ops.lookup.map(|_| lookup_impl),
        forget: ops.forget.map(|_| forget_impl),
        getattr: ops.getattr.map(|_| getattr_impl),
        setattr: ops.setattr.map(|_| setattr_impl),
        readlink: ops.readlink.map(|_| readlink_impl),
        mknod: ops.mknod.map(|_| mknod_impl),
        mkdir: ops.mkdir.map(|_| mkdir_impl),
        unlink: ops.unlink.map(|_| unlink_impl),
        rmdir: ops.rmdir.map(|_| rmdir_impl),
        symlink: ops.symlink.map(|_| symlink_impl),
        rename: ops.rename.map(|_| rename_impl),
        link: ops.link.map(|_| link_impl),
        open: ops.open.map(|_| open_impl),
        read: ops.read.map(|_| read_impl),
        write: ops.write.map(|_| write_impl),
        flush: ops.flush.map(|_| flush_impl),
        release: ops.release.map(|_| release_impl),
        fsync: ops.fsync.map(|_| fsync_impl),
        opendir: ops.opendir.map(|_| opendir_impl),
        readdir: ops.readdir.map(|_| readdir_impl),
        releasedir: ops.releasedir.map(|_| releasedir_impl),
        fsyncdir: ops.fsyncdir.map(|_| fsyncdir_impl),
        statfs: ops.statfs.map(|_| statfs_impl),
        setxattr: ops.setxattr.map(|_| setxattr_impl),
        getxattr: ops.getxattr.map(|_| getxattr_impl),
        listxattr: ops.listxattr.map(|_| listxattr_impl),
        removexattr: ops.removexattr.map(|_| removexattr_impl),
        access: ops.access.map(|_| access_impl),
        create: ops.create.map(|_| create_impl),

        // Not implemented yet:
        getlk: None,
        setlk: None,
        bmap: None,
        ioctl: None,
        poll: None,
        write_buf: None,
        retrieve_reply: None,
        forget_multi: None,
        flock: None,
        fallocate: None,
    }
}

type ReplySuccessFn<T> = ~fn(req:fuse_req_t, reply:T);

#[fixed_stack_segment]
fn send_fuse_reply<T>(result:ErrnoResult<T>, req:fuse_req_t, 
                      reply_success:ReplySuccessFn<T>) {
    match result {
        Ok(reply) => reply_success(req, reply),
        Err(errno) => unsafe { fuse_reply_err(req, errno); },
    };
}

fn run_for_reply<T>(req:fuse_req_t, reply_success:ReplySuccessFn<T>,
                    do_op:~fn(&FuseLowLevelOps) -> ErrnoResult<T>) {
    #[fixed_stack_segment] 
    unsafe fn call_fuse_req_userdata(req:fuse_req_t) -> *mut c_void {
        fuse_req_userdata(req)
    }
    let mut task = task();
    task.sched_mode(SingleThreaded);
    task.supervised();
    do task.spawn_with((reply_success, do_op)) |(reply_success, do_op)| {
        unsafe {
            do userdata_to_ops(call_fuse_req_userdata(req), reply_success)
                |ops, reply_success| {
                send_fuse_reply(do_op(ops), req, reply_success)
            }
        }
    }
}

#[inline]
fn cptr_to_str<T>(cptr:*c_schar, func:&fn(&str) -> T) -> T {
    unsafe {
        let cstr = CString::new(cptr,false);
        func(str::from_utf8_slice(cstr.as_bytes()).trim_right_chars(&(0u8 as char)))
    }
}

#[fixed_stack_segment]
fn reply_entryparam(req: fuse_req_t, reply:EntryReply) {
    unsafe {
        fuse_reply_entry(req, ptr::to_unsafe_ptr(&reply));
    }
}

#[fixed_stack_segment]
fn reply_attr(req: fuse_req_t, reply: AttrReply) {
    unsafe {
        fuse_reply_attr(req, ptr::to_unsafe_ptr(&reply.attr), reply.attr_timeout);
    }
}

#[fixed_stack_segment]
fn reply_none(req: fuse_req_t, _arg:()) {
    unsafe {
        fuse_reply_none(req);
    }
}

#[fixed_stack_segment]
fn reply_readlink(req: fuse_req_t, link:~str) {
    do link.with_c_str() |c_link| {
        unsafe {
            fuse_reply_readlink(req, c_link);
        }
    }
}

#[fixed_stack_segment]
fn reply_zero_err(req: fuse_req_t, _arg:()) {
    unsafe {
        fuse_reply_err(req, 0);
    }
}

fn openreply_to_fileinfo(reply: OpenReply) -> Struct_fuse_file_info {
    Struct_fuse_file_info{
        direct_io: to_bit(reply.direct_io) as c_uint,
        keep_cache: to_bit(reply.keep_cache) as c_uint,
        fh: reply.fh,
        ..zero()
    }
}

#[fixed_stack_segment]
fn reply_open(req: fuse_req_t, reply: OpenReply) {
    unsafe {
        let fi = openreply_to_fileinfo(reply);
        fuse_reply_open(req, ptr::to_unsafe_ptr(&fi));
    }
}

#[fixed_stack_segment]
fn reply_create(req: fuse_req_t, reply: CreateReply) {
    unsafe {
        let fi = openreply_to_fileinfo(reply.open_reply);
        fuse_reply_create(req, ptr::to_unsafe_ptr(&(reply.entry_param)),
                          ptr::to_unsafe_ptr(&fi));
    }
}

#[fixed_stack_segment]
fn reply_read(req: fuse_req_t, reply: ReadReply) {
    unsafe {
        match reply {
            DataBuffer(vec) => {
                fuse_reply_buf(req, vec::raw::to_ptr(vec) as *c_schar,
                               vec.len() as size_t);
            },
            EOF => {
                fuse_reply_buf(req, ptr::null(), 0);
            }
        }
    }
}

#[fixed_stack_segment]
fn reply_readdir(req: fuse_req_t, tuple: (size_t, ReaddirReply)) {
    let (size, reply) = tuple;
    match reply {
        DirEntries(entries) => {
            // Alignment makes the size per entry not an exact function
            // of the length of the name, but this should be enough for
            // what's needed
            static EXTRA_CAP_PER_ENTRY:size_t = 32;
            let mut lengths = entries.iter().map(|x| x.name.len() as size_t);
            let max_buf_size = lengths.sum() +
                (entries.len() as size_t*EXTRA_CAP_PER_ENTRY);
            let buf_size = cmp::min(max_buf_size, size);
            let mut buf: ~[c_schar] = vec::with_capacity(buf_size as uint);
            buf.grow(buf_size as uint, &(0 as c_schar));
            let mut returned_size = 0 as size_t;
            unsafe {
                for entry in entries.iter() {
                    let buf_ptr = ptr::mut_offset(vec::raw::to_mut_ptr(buf),
                                                  returned_size as int);
                    let remaining_size = buf_size - returned_size;
                    let added_size = do entry.name.to_c_str().with_ref |name_cstr| {
                        let stbuf = stat{
                            st_mode: entry.mode,
                            st_ino: entry.ino,
                            ..default_stat()
                        };
                        fuse_add_direntry(req,
                                          buf_ptr,
                                          remaining_size,
                                          name_cstr,
                                          ptr::to_unsafe_ptr(&stbuf),
                                          entry.next_offset)
                    };
                    if added_size <= remaining_size {
                        returned_size += added_size;
                    } else {
                        break;
                    }
                }
                fuse_reply_buf(req, vec::raw::to_ptr(buf) as *c_schar,
                               returned_size);
            }
        }
    }
}

#[fixed_stack_segment]
fn reply_write(req: fuse_req_t, count: size_t) {
    unsafe {
        fuse_reply_write(req, count);
    }
}

#[fixed_stack_segment]
fn reply_statfs(req: fuse_req_t, statfs: Struct_statvfs) {
    unsafe {
        fuse_reply_statfs(req, ptr::to_unsafe_ptr(&statfs));
    }
}

#[fixed_stack_segment]
fn reply_xattr(req: fuse_req_t, size: size_t) {
    unsafe {
        fuse_reply_xattr(req, size);
    }
}

fn handle_unimpl<F, T>(opt:&Option<F>, imp:&fn(&F) -> ErrnoResult<T>)
                       -> ErrnoResult<T> {
    match *opt {
        Some(ref f) => imp(f),
        None => {
            error!("FUSE called a callback for which there was no"+
                   "fn supplied.  This should never happen.");
            Err(ENOSYS)
        }
    }
}

extern fn init_impl(userdata:*mut c_void, _conn:*Struct_fuse_conn_info) {
    do userdata_to_ops(userdata, ()) |ops, _| {
        match ops.init {
            Some(ref f) => (*f)(),
            None=>()
        }
    }
}

extern fn destroy_impl(userdata:*mut c_void) {
    do userdata_to_ops(userdata, ()) |ops, _| {
        match ops.destroy {
            Some(ref f) => (*f)(),
            None=>()
        }
    }
}

macro_rules! multi_do {
    (|$args:pat| <- $e:expr; $(|$args_rest:pat| <- $e_rest:expr;)+ => $blk:block) =>
        (do $e |$args| { multi_do!($(|$args_rest| <- $e_rest;)+ => $blk) });
    {|$args:pat| <- $e:expr; => $blk:block} => (do $e |$args| $blk);
}

extern fn lookup_impl(req:fuse_req_t,  parent:fuse_ino_t, name:*c_schar) {
    multi_do!(
        |ops| <- run_for_reply(req, reply_entryparam);
        |f| <- handle_unimpl(&ops.lookup);
        |name| <- cptr_to_str(name);
        => {
            (*f)(parent, name)
        });
}

extern fn forget_impl(req: fuse_req_t, ino: fuse_ino_t, nlookup:c_ulong) {
    do run_for_reply(req, reply_none) |ops| {
        do handle_unimpl(&ops.forget) |f| {
            (*f)(ino, nlookup); Ok(())
        }
    }
}

extern fn getattr_impl(req:fuse_req_t, ino: fuse_ino_t,
                       _fi:*Struct_fuse_file_info) {
    do run_for_reply(req, reply_attr) |ops| {
        do handle_unimpl(&ops.getattr) |f| {
            (*f)(ino)
        }
    }
}

extern fn setattr_impl(req: fuse_req_t, ino: fuse_ino_t, attr:*stat,
                       to_set: int, fi: *Struct_fuse_file_info) {
    static FUSE_SET_ATTR_MODE:int = (1 << 0);
    static FUSE_SET_ATTR_UID:int = (1 << 1);
    static FUSE_SET_ATTR_GID:int = (1 << 2);
    static FUSE_SET_ATTR_SIZE:int = (1 << 3);
    static FUSE_SET_ATTR_ATIME:int = (1 << 4);
    static FUSE_SET_ATTR_MTIME:int = (1 << 5);
    static FUSE_SET_ATTR_ATIME_NOW:int = (1 << 7);
    static FUSE_SET_ATTR_MTIME_NOW:int = (1 << 8);
    do run_for_reply(req, reply_attr) |ops| {
        do handle_unimpl(&ops.setattr) |f| {
            unsafe {
                let mut attrs_to_set:~[AttrToSet] = vec::with_capacity(8);
                if to_set & FUSE_SET_ATTR_MODE != 0 { attrs_to_set.push(Mode((*attr).st_mode)) }
                if to_set & FUSE_SET_ATTR_UID != 0 { attrs_to_set.push(Uid((*attr).st_uid)) }
                if to_set & FUSE_SET_ATTR_GID != 0 { attrs_to_set.push(Gid((*attr).st_gid)) }
                if to_set & FUSE_SET_ATTR_SIZE != 0 { attrs_to_set.push(Size((*attr).st_size)) }
                if to_set & FUSE_SET_ATTR_ATIME != 0 { attrs_to_set.push(Atime((*attr).st_atime)) }
                if to_set & FUSE_SET_ATTR_MTIME != 0 { attrs_to_set.push(Mtime((*attr).st_mtime)) }
                if to_set & FUSE_SET_ATTR_ATIME_NOW != 0 { attrs_to_set.push(Atime_now) }
                if to_set & FUSE_SET_ATTR_MTIME_NOW != 0 { attrs_to_set.push(Mtime_now) }

                (*f)(ino, attrs_to_set, fi.to_option().map(|fi| fi.fh))
            }
        }
    }
}

extern fn readlink_impl(req: fuse_req_t, ino: fuse_ino_t) {
    do run_for_reply(req, reply_readlink) |ops| {
        do handle_unimpl(&ops.readlink) |f| {
            (*f)(ino)
        }
    }
}

extern fn mknod_impl(req:fuse_req_t, parent: fuse_ino_t, name:*c_schar,
                     mode: mode_t, rdev: dev_t) {
    do run_for_reply(req, reply_entryparam) |ops| {
        do handle_unimpl(&ops.mknod) |f| {
            do cptr_to_str(name) |name| { (*f)(parent, name, mode, rdev) }
        }
    }
}

extern fn mkdir_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar, 
                     mode:mode_t) {
    do run_for_reply(req, reply_entryparam) |ops| {
        do handle_unimpl(&ops.mkdir) |f| {
            do cptr_to_str(name) |name| { (*f)(parent, name, mode) }
        }
    }
}

extern fn unlink_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar) {
    do run_for_reply(req, reply_zero_err) |ops| {
        do handle_unimpl(&ops.unlink) |f| {
            do cptr_to_str(name) |name| { (*f)(parent, name) }
        }
    }
}

extern fn rmdir_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar) {
    do run_for_reply(req, reply_zero_err) |ops| {
        do handle_unimpl(&ops.rmdir) |f| {
            do cptr_to_str(name) |name| { (*f)(parent, name) }
        }
    }
}

extern fn symlink_impl(req: fuse_req_t, link: *c_schar, parent: fuse_ino_t,
                       name: *c_schar) {
    do run_for_reply(req, reply_entryparam) |ops| {
        do handle_unimpl(&ops.symlink) |f| {
            do cptr_to_str(link) |link| {
                do cptr_to_str(name) |name| {
                    (*f)(link, parent, name)
                }
            }
        }
    }
}

extern fn rename_impl(req: fuse_req_t, parent: fuse_ino_t, name: *c_schar, newparent: fuse_ino_t,
                      newname: *c_schar) {
    do run_for_reply(req, reply_zero_err) |ops| {
        do handle_unimpl(&ops.rename) |f| {
            do cptr_to_str(name) |name| {
                do cptr_to_str(newname) |newname| {
                    (*f)(parent, name, newparent,newname)
                }
            }
        }
    }
}

extern fn link_impl(req: fuse_req_t, ino: fuse_ino_t, newparent: fuse_ino_t,
                    newname: *c_schar) {
    do run_for_reply(req, reply_entryparam) |ops| {
        do handle_unimpl(&ops.link) |f| {
            do cptr_to_str(newname) |newname| {
                (*f)(ino, newparent, newname)
            }
        }
    }
}

extern fn open_impl(req: fuse_req_t, ino: fuse_ino_t,
                    fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_open) |ops| {
        do handle_unimpl(&ops.open) |f| {
            unsafe {
                (*f)(ino, (*fi).flags)
            }
        }
    }
}

extern fn read_impl(req: fuse_req_t, ino: fuse_ino_t, size: size_t, off: off_t,
                    fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_read) |ops| {
        do handle_unimpl(&ops.read) |f| {
            unsafe {
                (*f)(ino, size, off, (*fi).fh)
            }
        }
    }
}

extern fn write_impl(req: fuse_req_t, ino: fuse_ino_t, buf: *u8,
                     size: size_t, off: off_t, fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_write) |ops| {
        do handle_unimpl(&ops.write) |f| {
            unsafe {
                do vec::raw::buf_as_slice(buf, size as uint) |vec| {
                    (*f)(ino, vec, off, (*fi).fh, ((*fi).writepage != 0))
                }
            }
        }
    }
}

extern fn flush_impl(req: fuse_req_t, ino: fuse_ino_t,
                     fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_zero_err) |ops| {
        do handle_unimpl(&ops.flush) |f| {
            unsafe {
                (*f)(ino, (*fi).lock_owner, (*fi).fh)
            }
        }
    }
}

extern fn release_impl(req: fuse_req_t, ino: fuse_ino_t,
                       fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_zero_err) |ops| {
        do handle_unimpl(&ops.release) |f| {
            unsafe {
                (*f)(ino, (*fi).flags, (*fi).fh)
            }
        }
    }
}

extern fn fsync_impl(req: fuse_req_t, ino: fuse_ino_t, datasync: c_int,
                     fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_zero_err) |ops| {
        do handle_unimpl(&ops.fsync) |f| {
            unsafe {
                (*f)(ino, (datasync != 0), (*fi).fh)
            }
        }
    }
}

extern fn opendir_impl(req: fuse_req_t, ino: fuse_ino_t,
                       _fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_open) |ops| {
        do handle_unimpl(&ops.opendir) |f| {
            (*f)(ino)
        }
    }
}

extern fn readdir_impl(req: fuse_req_t, ino: fuse_ino_t, size: size_t, off: off_t,
                       fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_readdir) |ops| {
        do handle_unimpl(&ops.readdir) |f| {
            unsafe {
                (*f)(ino, size, off, (*fi).fh)
            }.chain(|rr| Ok((size, rr)))
        }
    }
}

extern fn releasedir_impl(req: fuse_req_t, ino: fuse_ino_t,
                          fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_zero_err) |ops| {
        do handle_unimpl(&ops.releasedir) |f| {
            unsafe {
                (*f)(ino, (*fi).fh)
            }
        }
    }
}

extern fn fsyncdir_impl(req: fuse_req_t, ino: fuse_ino_t, datasync: c_int,
                        fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_zero_err) |ops| {
        do handle_unimpl(&ops.fsyncdir) |f| {
            unsafe {
                (*f)(ino, (datasync != 0), (*fi).fh)
            }
        }
    }
}

extern fn statfs_impl(req: fuse_req_t, ino: fuse_ino_t) {
    do run_for_reply(req, reply_statfs) |ops| {
        do handle_unimpl(&ops.statfs) |f| {
            (*f)(ino)
        }
    }
}

extern fn setxattr_impl(req: fuse_req_t, ino: fuse_ino_t, name: *c_schar,
                        value: *u8, size: size_t, flags: c_int) {
    do run_for_reply(req, reply_zero_err) |ops| {
        do handle_unimpl(&ops.setxattr) |f| {
            unsafe {
                do vec::raw::buf_as_slice(value, size as uint) |vec| {
                    do cptr_to_str(name) |name| {
                        (*f)(ino, name, vec, flags)
                    }
                }
            }
        }
    }
}

extern fn getxattr_impl(req: fuse_req_t, ino: fuse_ino_t, name: *c_schar,
                        size: size_t) {
    if size == 0 {
        do run_for_reply(req, reply_xattr) |ops| {
            do handle_unimpl(&ops.getxattr_size) |f| {
                do cptr_to_str(name) |name| { (*f)(ino, name) }
            }
        }
    } else {
        do run_for_reply(req, reply_read) |ops| {
            do handle_unimpl(&ops.getxattr) |f| {
                do cptr_to_str(name) |name| { (*f)(ino, name, size) }
            }
        }
    }
}

extern fn listxattr_impl(req: fuse_req_t, ino: fuse_ino_t, size: size_t) {
    if size == 0 {
        do run_for_reply(req, reply_xattr) |ops| {
            do handle_unimpl(&ops.listxattr_size) |f| {
                (*f)(ino)
            }
        }
    } else {
        do run_for_reply(req, reply_read) |ops| {
            do handle_unimpl(&ops.listxattr) |f| {
                (*f)(ino, size)
            }
        }
    }
}

extern fn removexattr_impl(req: fuse_req_t, ino: fuse_ino_t, name: *c_schar) {
    do run_for_reply(req, reply_zero_err) |ops| {
        do handle_unimpl(&ops.removexattr) |f| {
            do cptr_to_str(name) |name| { (*f)(ino, name) }
        }
    }
}

extern fn access_impl(req: fuse_req_t, 
                      ino: fuse_ino_t, mask: c_int) {
    do run_for_reply(req, reply_zero_err) |ops| {
        do handle_unimpl(&ops.access) |f| {
            (*f)(ino, mask)
        }
    }
}

extern fn create_impl(req: fuse_req_t, parent: fuse_ino_t, name: *c_schar,
                      mode: mode_t, fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_create) |ops| {
        do handle_unimpl(&ops.create) |f| {
            unsafe {
                do cptr_to_str(name) |name| {
                    (*f)(parent, name, mode, (*fi).flags)
                }
            }
        }
    }
}
