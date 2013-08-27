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
use fuse_c::*;
use std::path::stat::arch::default_stat;
pub use fuse_c::{fuse_ino_t,Struct_fuse_entry_param};

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
 * Trait for "thin" interface to low-level FUSE ops.
 *
 * It would be best if a user of this could just implement the methods as
 * desired, and leave the rest as defaults, and have this interface take care of
 * the rest.  Unfortunately that's not quite possible.  FUSE has default
 * behavior for some ops that can't just be invoked from a callback--the only
 * way to get it is to pass NULL for the callback pointers into the
 * fuse_lowlevel_ops structure.  But we can't know at run time which default
 * methods of a trait were overridden, which means we don't know which entries
 * in the Struct_fuse_lowlevel_ops to null out.
 *
 * Instead, we've got a corresponding "is_implemented" method for each one.
 * Define it to return true for each real method you implement.  _BLEAH!  YUCK!
 * UGH!_ If you return true from an "is_implemented" method, but don't implement
 * the corresponding real method, it's not a compile error--you just fail at
 * run-time!  But it's the best we can do without some sort of reflection API
 * that rust doesn't have, or a way to call the FUSE default behavior from a
 * callback, which FUSE does not have.

 */
pub trait FuseLowLevelOps {
    fn init(&self) { fail!() }
    fn init_is_implemented(&self) -> bool { false }
    fn destroy(&self) { fail!() }
    fn destroy_is_implemented(&self) -> bool { false }
    fn lookup(&self, _parent: fuse_ino_t, _name: &str) -> ErrnoResult<EntryReply> { fail!() }
    fn lookup_is_implemented(&self) -> bool { false }
    fn forget(&self, _ino:fuse_ino_t, _nlookup:c_ulong) { fail!() }
    fn forget_is_implemented(&self) -> bool { false }
    fn getattr(&self, _ino: fuse_ino_t) -> ErrnoResult<AttrReply> { fail!() }
    fn getattr_is_implemented(&self) -> bool { false }
    fn setattr(&self, _ino: fuse_ino_t, _attrs_to_set:&[AttrToSet], _fh:Option<u64>) -> ErrnoResult<AttrReply> { fail!() }
    fn setattr_is_implemented(&self) -> bool { false }
    fn readlink(&self, _ino: fuse_ino_t) -> ErrnoResult<~str> { fail!() }
    fn readlink_is_implemented(&self) -> bool { false }
    fn mknod(&self, _parent: fuse_ino_t, _name: &str, _mode: mode_t, _rdev: dev_t) -> ErrnoResult<EntryReply> { fail!() }
    fn mknod_is_implemented(&self) -> bool { false }
    fn mkdir(&self, _parent: fuse_ino_t, _name: &str, _mode: mode_t) -> ErrnoResult<EntryReply> { fail!() }
    fn mkdir_is_implemented(&self) -> bool { false }
    // TODO: Using the unit type with result seems kind of goofy, but;
    // is done for consistency with the others.  Is this right?;
    fn unlink(&self, _parent: fuse_ino_t, _name: &str) -> ErrnoResult<()> { fail!() }
    fn unlink_is_implemented(&self) -> bool { false }
    fn rmdir(&self, _parent: fuse_ino_t, _name: &str) -> ErrnoResult<()> { fail!() }
    fn rmdir_is_implemented(&self) -> bool { false }
    fn symlink(&self, _link:&str, _parent: fuse_ino_t, _name: &str) -> ErrnoResult<EntryReply> { fail!() }
    fn symlink_is_implemented(&self) -> bool { false }
    fn rename(&self, _parent: fuse_ino_t, _name: &str, _newparent: fuse_ino_t, _newname: &str) -> ErrnoResult<()> { fail!() }
    fn rename_is_implemented(&self) -> bool { false }
    fn link(&self, _ino: fuse_ino_t, _newparent: fuse_ino_t, _newname: &str) -> ErrnoResult<EntryReply> { fail!() }
    fn link_is_implemented(&self) -> bool { false }
    fn open(&self, _ino: fuse_ino_t, _flags: c_int) -> ErrnoResult<OpenReply> { fail!() }
    fn open_is_implemented(&self) -> bool { false }
    fn read(&self, _ino: fuse_ino_t, _size: size_t, _off: off_t, _fh: u64) -> ErrnoResult<ReadReply> { fail!() }
    fn read_is_implemented(&self) -> bool { false }
    // TODO: is writepage a bool, or an actual number that needs to be;
    // preserved?;
    fn write(&self, _ino: fuse_ino_t, _buf:&[u8], _off: off_t, _fh: u64, _writepage: bool) -> ErrnoResult<size_t> { fail!() }
    fn write_is_implemented(&self) -> bool { false }
    fn flush(&self, _ino: fuse_ino_t, _lock_owner: u64, _fh: u64) -> ErrnoResult<()> { fail!() }
    fn flush_is_implemented(&self) -> bool { false }
    fn release(&self, _ino: fuse_ino_t, _flags: c_int, _fh: u64) -> ErrnoResult<()> { fail!() }
    fn release_is_implemented(&self) -> bool { false }
    fn fsync(&self, _ino: fuse_ino_t, _datasync: bool, _fh: u64) -> ErrnoResult<()> { fail!() }
    fn fsync_is_implemented(&self) -> bool { false }
    fn opendir(&self, _ino: fuse_ino_t) -> ErrnoResult<OpenReply> { fail!() }
    fn opendir_is_implemented(&self) -> bool { false }
    fn readdir(&self, _ino: fuse_ino_t, _size: size_t, _off: off_t, _fh: u64) -> ErrnoResult<ReaddirReply> { fail!() }
    fn readdir_is_implemented(&self) -> bool { false }
    fn releasedir(&self, _ino: fuse_ino_t, _fh: u64) -> ErrnoResult<()> { fail!() }
    fn releasedir_is_implemented(&self) -> bool { false }
    fn fsyncdir(&self, _ino: fuse_ino_t, _datasync: bool, _fh: u64) -> ErrnoResult<()> { fail!() }
    fn fsyncdir_is_implemented(&self) -> bool { false }
    fn statfs(&self, _ino: fuse_ino_t) -> ErrnoResult<Struct_statvfs> { fail!() }
    fn statfs_is_implemented(&self) -> bool { false }
    fn setxattr(&self, _ino: fuse_ino_t, _name: &str, _value: &[u8], _flags: c_int) -> ErrnoResult<()> { fail!() }
    fn setxattr_is_implemented(&self) -> bool { false }
    // TODO: examine this--ReadReply may not be appropraite here;
    fn getxattr(&self, _ino: fuse_ino_t, _name: &str, _size: size_t) -> ErrnoResult<ReadReply> { fail!() }
    fn getxattr_is_implemented(&self) -> bool { false }
    // Called on getxattr with size of zero (meaning a query of total size);
    fn getxattr_size(&self, _ino: fuse_ino_t, _name: &str) -> ErrnoResult<size_t> { fail!() }
    // TODO: examine this--ReadReply may not be appropraite here;
    fn listxattr(&self, _ino: fuse_ino_t, _size: size_t) -> ErrnoResult<ReadReply> { fail!() }
    fn listxattr_is_implemented(&self) -> bool { false }
    // Called on listxattr with size of zero (meaning a query of total size);
    fn listxattr_size(&self, _ino: fuse_ino_t) -> ErrnoResult<size_t> { fail!() }
    fn removexattr(&self, _ino: fuse_ino_t, _name: &str) -> ErrnoResult<()> { fail!() }
    fn removexattr_is_implemented(&self) -> bool { false }
    fn access(&self, _ino: fuse_ino_t, _mask: c_int) -> ErrnoResult<()> { fail!() }
    fn access_is_implemented(&self) -> bool { false }
    fn create(&self, _parent: fuse_ino_t, _name: &str, _mode: mode_t, _flags: c_int) -> ErrnoResult<CreateReply> { fail!() }
    fn create_is_implemented(&self) -> bool { false }

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
* Run a function with a borrowed pointer to the FuseLowLevelOps object pointed
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
        if fuse_chan == ptr::null() {
            // TODO: better error message?
            stderr().write_line("Failed to mount\n");
            fail!();
        }
        
        let llo = make_fuse_ll_oper(ops);
        let fuse_session = fuse_lowlevel_new(ptr::to_mut_unsafe_ptr(&mut fuse_args),
                                             ptr::to_unsafe_ptr(&llo),
                                             size_of::<Struct_fuse_lowlevel_ops>() as size_t,
                                             ptr::to_unsafe_ptr(&ops) as *mut c_void);
        if fuse_session == ptr::null() {
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
        init: if ops.init_is_implemented() { Some(init_impl) } else { None },
        destroy: if ops.destroy_is_implemented() { Some(destroy_impl) } else { None },
        lookup: if ops.lookup_is_implemented() { Some(lookup_impl) } else { None },
        forget: if ops.forget_is_implemented() { Some(forget_impl) } else { None },
        getattr: if ops.getattr_is_implemented() { Some(getattr_impl) } else { None },
        setattr: if ops.setattr_is_implemented() { Some(setattr_impl) } else { None },
        readlink: if ops.readlink_is_implemented() { Some(readlink_impl) } else { None },
        mknod: if ops.mknod_is_implemented() { Some(mknod_impl) } else { None },
        mkdir: if ops.mkdir_is_implemented() { Some(mkdir_impl) } else { None },
        unlink: if ops.unlink_is_implemented() { Some(unlink_impl) } else { None },
        rmdir: if ops.rmdir_is_implemented() { Some(rmdir_impl) } else { None },
        symlink: if ops.symlink_is_implemented() { Some(symlink_impl) } else { None },
        rename: if ops.rename_is_implemented() { Some(rename_impl) } else { None },
        link: if ops.link_is_implemented() { Some(link_impl) } else { None },
        open: if ops.open_is_implemented() { Some(open_impl) } else { None },
        read: if ops.read_is_implemented() { Some(read_impl) } else { None },
        write: if ops.write_is_implemented() { Some(write_impl) } else { None },
        flush: if ops.flush_is_implemented() { Some(flush_impl) } else { None },
        release: if ops.release_is_implemented() { Some(release_impl) } else { None },
        fsync: if ops.fsync_is_implemented() { Some(fsync_impl) } else { None },
        opendir: if ops.opendir_is_implemented() { Some(opendir_impl) } else { None },
        readdir: if ops.readdir_is_implemented() { Some(readdir_impl) } else { None },
        releasedir: if ops.releasedir_is_implemented() { Some(releasedir_impl) } else { None },
        fsyncdir: if ops.fsyncdir_is_implemented() { Some(fsyncdir_impl) } else { None },
        statfs: if ops.statfs_is_implemented() { Some(statfs_impl) } else { None },
        setxattr: if ops.setxattr_is_implemented() { Some(setxattr_impl) } else { None },
        getxattr: if ops.getxattr_is_implemented() { Some(getxattr_impl) } else { None },
        listxattr: if ops.listxattr_is_implemented() { Some(listxattr_impl) } else { None },
        removexattr: if ops.removexattr_is_implemented() { Some(removexattr_impl) } else { None },
        access: if ops.access_is_implemented() { Some(access_impl) } else { None },
        create: if ops.create_is_implemented() { Some(create_impl) } else { None },

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
unsafe fn cptr_to_str<T>(cptr:*c_schar, func:&fn(&str) -> T) -> T {
    let cstr = CString::new(cptr,false);
    func(str::from_bytes_slice(cstr.as_bytes()).trim_right_chars(&(0 as char)))
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

extern fn init_impl(userdata:*mut c_void, _conn:*Struct_fuse_conn_info) {
    do userdata_to_ops(userdata, ()) |ops, _| { ops.init() }
}

extern fn destroy_impl(userdata:*mut c_void) {
    do userdata_to_ops(userdata, ()) |ops, _| { ops.destroy() }
}

extern fn lookup_impl(req:fuse_req_t,  parent:fuse_ino_t, name:*c_schar) {
    do run_for_reply(req, reply_entryparam) |ops| {
        unsafe {
            do cptr_to_str(name) |name| { ops.lookup(parent, name)}
        }
    }
}

extern fn forget_impl(req: fuse_req_t, ino: fuse_ino_t, nlookup:c_ulong) {
    do run_for_reply(req, reply_none) |ops| {
        ops.forget(ino, nlookup); Ok(())
    }
}

extern fn getattr_impl(req:fuse_req_t, ino: fuse_ino_t,
                       _fi:*Struct_fuse_file_info) {
    do run_for_reply(req, reply_attr) |ops| {
        ops.getattr(ino)
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

            ops.setattr(ino, attrs_to_set, fi.to_option().map(|fi| fi.fh))
        }
    }
}

extern fn readlink_impl(req: fuse_req_t, ino: fuse_ino_t) {
    do run_for_reply(req, reply_readlink) |ops| {
        ops.readlink(ino)
    }
}

extern fn mknod_impl(req:fuse_req_t, parent: fuse_ino_t, name:*c_schar,
                     mode: mode_t, rdev: dev_t) {
    do run_for_reply(req, reply_entryparam) |ops| {
        unsafe {
            do cptr_to_str(name) |name| { ops.mknod(parent, name, mode, rdev) }
        }
    }
}

extern fn mkdir_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar, 
                     mode:mode_t) {
    do run_for_reply(req, reply_entryparam) |ops| {
        unsafe {
            do cptr_to_str(name) |name| { ops.mkdir(parent, name, mode) }
        }
    }
}

extern fn unlink_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe {
            do cptr_to_str(name) |name| { ops.unlink(parent, name) }
        }
    }
}

extern fn rmdir_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe {
            do cptr_to_str(name) |name| { ops.rmdir(parent, name) }
        }
    }
}

extern fn symlink_impl(req: fuse_req_t, link: *c_schar, parent: fuse_ino_t,
                       name: *c_schar) {
    do run_for_reply(req, reply_entryparam) |ops| {
        unsafe {
            do cptr_to_str(link) |link| {
                do cptr_to_str(name) |name| {
                    ops.symlink(link, parent, name)
                }
            }
        }
    }
}

extern fn rename_impl(req: fuse_req_t, parent: fuse_ino_t, name: *c_schar, newparent: fuse_ino_t,
                       newname: *c_schar) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe {
            do cptr_to_str(name) |name| {
                do cptr_to_str(newname) |newname| {
                    ops.rename(parent, name, newparent,newname)
                }
            }
        }
    }
}

extern fn link_impl(req: fuse_req_t, ino: fuse_ino_t, newparent: fuse_ino_t,
                    newname: *c_schar) {
    do run_for_reply(req, reply_entryparam) |ops| {
        unsafe {
            do cptr_to_str(newname) |newname| {
                ops.link(ino, newparent, newname)
            }
        }
    }
}

extern fn open_impl(req: fuse_req_t, ino: fuse_ino_t,
                    fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_open) |ops| {
        unsafe {
            ops.open(ino, (*fi).flags)
        }
    }
}

extern fn read_impl(req: fuse_req_t, ino: fuse_ino_t, size: size_t, off: off_t,
                    fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_read) |ops| {
        unsafe {
            ops.read(ino, size, off, (*fi).fh)
        }
    }
}

extern fn write_impl(req: fuse_req_t, ino: fuse_ino_t, buf: *u8,
                     size: size_t, off: off_t, fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_write) |ops| {
        unsafe {
            do vec::raw::buf_as_slice(buf, size as uint) |vec| {
                ops.write(ino, vec, off, (*fi).fh, ((*fi).writepage != 0))
            }
        }
    }
}

extern fn flush_impl(req: fuse_req_t, ino: fuse_ino_t,
                     fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe {
            ops.flush(ino, (*fi).lock_owner, (*fi).fh)
        }
    }
}

extern fn release_impl(req: fuse_req_t, ino: fuse_ino_t,
                       fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe {
            ops.release(ino, (*fi).flags, (*fi).fh)
        }
    }
}

extern fn fsync_impl(req: fuse_req_t, ino: fuse_ino_t, datasync: c_int,
                     fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe {
            ops.fsync(ino, (datasync != 0), (*fi).fh)
        }
    }
}

extern fn opendir_impl(req: fuse_req_t, ino: fuse_ino_t,
                     _fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_open) |ops| {
        ops.opendir(ino)
    }
}

extern fn readdir_impl(req: fuse_req_t, ino: fuse_ino_t, size: size_t, off: off_t,
                    fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_readdir) |ops| {
        unsafe {
            ops.readdir(ino, size, off, (*fi).fh)
        }.chain(|rr| Ok((size, rr)))
    }
}

extern fn releasedir_impl(req: fuse_req_t, ino: fuse_ino_t,
                       fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe {
            ops.releasedir(ino, (*fi).fh)
        }
    }
}

extern fn fsyncdir_impl(req: fuse_req_t, ino: fuse_ino_t, datasync: c_int,
                     fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe {
            ops.fsyncdir(ino, (datasync != 0), (*fi).fh)
        }
    }
}

extern fn statfs_impl(req: fuse_req_t, ino: fuse_ino_t) {
    do run_for_reply(req, reply_statfs) |ops| {
        ops.statfs(ino)
    }
}

extern fn setxattr_impl(req: fuse_req_t, ino: fuse_ino_t, name: *c_schar,
                        value: *u8, size: size_t, flags: c_int) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe {
            do vec::raw::buf_as_slice(value, size as uint) |vec| {
                do cptr_to_str(name) |name| {
                    ops.setxattr(ino, name, vec, flags)
                }
            }
        }
    }
}

extern fn getxattr_impl(req: fuse_req_t, ino: fuse_ino_t, name: *c_schar,
                        size: size_t) {
    if size == 0 {
        do run_for_reply(req, reply_xattr) |ops| {
            unsafe {
                do cptr_to_str(name) |name| { ops.getxattr_size(ino, name) }
            }
        }
    } else {
        do run_for_reply(req, reply_read) |ops| {
            unsafe {
                do cptr_to_str(name) |name| { ops.getxattr(ino, name, size) }
            }
        }
    }
}

extern fn listxattr_impl(req: fuse_req_t, ino: fuse_ino_t, size: size_t) {
    if size == 0 {
        do run_for_reply(req, reply_xattr) |ops| {
            ops.listxattr_size(ino)
        }
    } else {
        do run_for_reply(req, reply_read) |ops| {
            ops.listxattr(ino, size)
        }
    }
}

extern fn removexattr_impl(req: fuse_req_t, ino: fuse_ino_t, name: *c_schar) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe {
            do cptr_to_str(name) |name| { ops.removexattr(ino, name) }
        }
    }
}

extern fn access_impl(req: fuse_req_t, ino: fuse_ino_t, mask: c_int) {
    do run_for_reply(req, reply_zero_err) |ops| {
        ops.access(ino, mask)
    }
}

extern fn create_impl(req: fuse_req_t, parent: fuse_ino_t, name: *c_schar,
                      mode: mode_t, fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_create) |ops| {
        unsafe {
            do cptr_to_str(name) |name| {
                ops.create(parent, name, mode, (*fi).flags)
            }
        }
    }
}
