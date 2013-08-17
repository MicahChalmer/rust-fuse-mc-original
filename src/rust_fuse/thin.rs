use std::libc::{
    c_double,
    c_int,
    c_schar,
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
use std::task::task;
use std::c_str::CString;
use std::cell;

mod fuse;

/// Information to be returned from open
#[deriving(Zero)]
pub struct OpenReply {
    direct_io:bool,
    keep_cache:bool,
    fh: u64
}

pub struct AttrReply {
    attr: stat,
    attr_timeout: c_double
}

pub struct EntryReply {
    ino: fuse::fuse_ino_t,
    generation: c_ulong,
    attr: stat,
    attr_timeout: c_double,
    entry_timeout: c_double
}

pub enum AttrToSet {
    Mode(mode_t),
    Uid(uid_t),
    Gid(gid_t),
    Size(size_t),
    Atime(time_t),
    Mtime(time_t),

    // TODO: confirm assumption about what these mean
    Atime_now,
    Mtime_now,
}

pub enum ReadReply {
    // TODO: support iov and fuse_bufvec type replies for more efficient (?)
    // types of implementation
    DataBuffer(~[u8])
}

/// The error result should be one of libc's errno values
type ErrnoResult<T> = Result<T, c_int>;

/**
 * Trait for "thin" interface to low-level FUSE ops.

 * It would be best if a user of this could just implement the methods as
 * desired, and leave the rest as defaults, and have this interface take care of
 * the rest.  Unfortunately that's not quite possible.  FUSE has default
 * behavior for some ops that can't just be invoked from a callback--the only
 * way to get it is to pass NULL for the callback pointers into the
 * fuse_lowlevel_ops structure.  But we can't know at run time which default
 * methods of a trait were overridden, which means we don't know which entries
 * in the Struct_fuse_lowlevel_ops to null out.

 * Instead, we've got a corresponding "is_implemented" method for each one.
 * Define it to return true for each real method you implement.  _BLEAH!  YUCK!
 * UGH!_ If you return true from an "is_implemented" method, but don't implement
 * the corresponding real method, it's not a compile error--you just fail at
 * run-time!  But it's the best we can do without some sort of reflection API
 * that rust doesn't have, or a way to call the FUSE default behavior from a
 * callback, which FUSE does not have.

 */
pub trait FuseLowLevelOps:Clone+Send {
    fn init(&self) { fail!() }
    fn init_is_implemented(&self) -> bool { false }
    fn destroy(&self) { fail!() }
    fn destroy_is_implemented(&self) -> bool { false }
    fn lookup(&self, _parent: fuse::fuse_ino_t, _name: &str) -> ErrnoResult<fuse::Struct_fuse_entry_param> { fail!() }
    fn lookup_is_implemented(&self) -> bool { false }
    fn forget(&self, _ino:fuse::fuse_ino_t, _nlookup:c_ulong) { fail!() }
    fn forget_is_implemented(&self) -> bool { false }
    fn getattr(&self, _ino: fuse::fuse_ino_t, _flags: c_int) -> ErrnoResult<AttrReply> { fail!() }
    fn getattr_is_implemented(&self) -> bool { false }
    fn setattr(&self, _ino: fuse::fuse_ino_t, _attrs_to_set:&[AttrToSet], _fh:Option<u64>) -> ErrnoResult<AttrReply> { fail!() }
    fn setattr_is_implemented(&self) -> bool { false }
    fn readlink(&self, _ino: fuse::fuse_ino_t) -> ErrnoResult<~str> { fail!() }
    fn readlink_is_implemented(&self) -> bool { false }
    fn mknod(&self, _parent: fuse::fuse_ino_t, _name: &str, _mode: mode_t, _rdev: dev_t) -> ErrnoResult<EntryReply> { fail!() }
    fn mknod_is_implemented(&self) -> bool { false }
    fn mkdir(&self, _parent: fuse::fuse_ino_t, _name: &str, _mode: mode_t) -> ErrnoResult<EntryReply> { fail!() }
    fn mkdir_is_implemented(&self) -> bool { false }
    // TODO: Using the unit type with result seems kind of goofy, but;
    // is done for consistency with the others.  Is this right?;
    fn unlink(&self, _parent: fuse::fuse_ino_t, _name: &str) -> ErrnoResult<()> { fail!() }
    fn unlink_is_implemented(&self) -> bool { false }
    fn rmdir(&self, _parent: fuse::fuse_ino_t, _name: &str) -> ErrnoResult<()> { fail!() }
    fn rmdir_is_implemented(&self) -> bool { false }
    fn symlink(&self, _link:&str, _parent: fuse::fuse_ino_t, _name: &str) -> ErrnoResult<EntryReply> { fail!() }
    fn symlink_is_implemented(&self) -> bool { false }
    fn rename(&self, _parent: fuse::fuse_ino_t, _name: &str, _newparent: fuse::fuse_ino_t, _newname: &str) -> ErrnoResult<()> { fail!() }
    fn rename_is_implemented(&self) -> bool { false }
    fn link(&self, _ino: fuse::fuse_ino_t, _newparent: fuse::fuse_ino_t, _newname: &str) -> ErrnoResult<EntryReply> { fail!() }
    fn link_is_implemented(&self) -> bool { false }
    fn open(&self, _ino: fuse::fuse_ino_t, _flags: c_int) -> ErrnoResult<OpenReply> { fail!() }
    fn open_is_implemented(&self) -> bool { false }
    fn read(&self, _ino: fuse::fuse_ino_t, _size: size_t, _off: off_t, _fh: u64) -> ErrnoResult<ReadReply> { fail!() }
    fn read_is_implemented(&self) -> bool { false }
    // TODO: is writepage a bool, or an actual number that needs to be;
    // preserved?;
    fn write(&self, _ino: fuse::fuse_ino_t, _buf:&[u8], _off: off_t, _fh: u64, _writepage: bool) -> ErrnoResult<size_t> { fail!() }
    fn write_is_implemented(&self) -> bool { false }
    fn flush(&self, _ino: fuse::fuse_ino_t, _lock_owner: u64, _fh: u64) -> ErrnoResult<()> { fail!() }
    fn flush_is_implemented(&self) -> bool { false }
    fn release(&self, _ino: fuse::fuse_ino_t, _flags: c_int, _fh: u64) -> ErrnoResult<()> { fail!() }
    fn release_is_implemented(&self) -> bool { false }
    fn fsync(&self, _ino: fuse::fuse_ino_t, _datasync: bool, _fh: u64) -> ErrnoResult<()> { fail!() }
    fn fsync_is_implemented(&self) -> bool { false }
    fn opendir(&self, _ino: fuse::fuse_ino_t) -> ErrnoResult<OpenReply> { fail!() }
    fn opendir_is_implemented(&self) -> bool { false }
    // TODO: Using a ReadReply would require the impl to do unsafe operations;
    // to use fuse_add_direntry.  So even the thin interface needs something;
    // else here.;
    fn readdir(&self, _ino: fuse::fuse_ino_t, _size: size_t, _off: off_t, _fh: u64) -> ErrnoResult<ReadReply> { fail!() }
    fn readdir_is_implemented(&self) -> bool { false }
    fn releasedir(&self, _ino: fuse::fuse_ino_t, _fh: u64) -> ErrnoResult<()> { fail!() }
    fn releasedir_is_implemented(&self) -> bool { false }
    fn fsyncdir(&self, _ino: fuse::fuse_ino_t, _datasync: bool, _fh: u64) -> ErrnoResult<()> { fail!() }
    fn fsyncdir_is_implemented(&self) -> bool { false }
    fn statfs(&self, _ino: fuse::fuse_ino_t) -> ErrnoResult<fuse::Struct_statvfs> { fail!() }
    fn statfs_is_implemented(&self) -> bool { false }
    fn setxattr(&self, _ino: fuse::fuse_ino_t, _name: &str, _value: &[u8], _flags: int) -> ErrnoResult<()> { fail!() }
    fn setxattr_is_implemented(&self) -> bool { false }
    // TODO: examine this--ReadReply may not be appropraite here;
    fn getxattr(&self, _ino: fuse::fuse_ino_t, _name: &str, _size: size_t) -> ErrnoResult<ReadReply> { fail!() }
    fn getxattr_is_implemented(&self) -> bool { false }
    // Called on getxattr with size of zero (meaning a query of total size);
    fn getxattr_size(&self, _ino: fuse::fuse_ino_t, _name: &str) -> ErrnoResult<size_t> { fail!() }
    // TODO: examine this--ReadReply may not be appropraite here;
    fn listxattr(&self, _ino: fuse::fuse_ino_t, _name: &str, _size: size_t) -> ErrnoResult<ReadReply> { fail!() }
    fn listxattr_is_implemented(&self) -> bool { false }
    // Called on listxattr with size of zero (meaning a query of total size);
    fn listxattr_size(&self, _ino: fuse::fuse_ino_t, _name: &str) -> ErrnoResult<size_t> { fail!() }
    fn removexattr(&self, _ino: fuse::fuse_ino_t, _name: &str) -> ErrnoResult<()> { fail!() }
    fn removexattr_is_implemented(&self) -> bool { false }
    fn access(&self, _ino: fuse::fuse_ino_t, _mask: c_int) -> ErrnoResult<()> { fail!() }
    fn access_is_implemented(&self) -> bool { false }
    fn create(&self, _ino: fuse::fuse_ino_t, _parent: fuse::fuse_ino_t, _name: &str, _mode: mode_t, _flags: c_int) -> ErrnoResult<OpenReply> { fail!() }
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

/**
* Run a function with a borrowed pointer to the FuseLowLevelOps object pointed to by the given userdata pointer.
*/
fn userdata_to_ops<Ops:FuseLowLevelOps,Result>(userdata:*mut c_void,
                      func:&fn(&Ops) -> Result) -> Result {
    unsafe {
        func(&*(userdata as *Ops))
    }
}

pub fn fuse_main_thin<Ops:FuseLowLevelOps+Send>(args:~[~str], ops:~Ops) {
    unsafe {
        let arg_c_strs_ptrs: ~[*c_schar] = args.map(|s| s.to_c_str().unwrap() );
        let mut fuse_args = fuse::Struct_fuse_args {
            argv: transmute(vec::raw::to_ptr(arg_c_strs_ptrs)),
            argc: args.len() as c_int,
            allocated: 0
        };
        let mut mountpoint:*mut c_schar = ptr::mut_null();
        if fuse::fuse_parse_cmdline(ptr::to_mut_unsafe_ptr(&mut fuse_args),
                              ptr::to_mut_unsafe_ptr(&mut mountpoint),
                              ptr::mut_null(), // multithreaded--we ignore
                              ptr::mut_null() // foreground--we ignore (for now)
                              ) == -1 {
            return;
        }
        
        let fuse_chan = fuse::fuse_mount(mountpoint as *c_schar, ptr::to_mut_unsafe_ptr(&mut fuse_args));
        if fuse_chan == ptr::null() {
            // TODO: better error message?
            stderr().write_line("Failed to mount\n");
            fail!();
        }
        
        let llo = make_fuse_ll_oper(ops);
        let fuse_session = fuse::fuse_lowlevel_new(ptr::to_mut_unsafe_ptr(&mut fuse_args),
                                             ptr::to_unsafe_ptr(&llo),
                                             size_of::<fuse::Struct_fuse_lowlevel_ops>() as size_t,
                                             ptr::to_unsafe_ptr(&ops) as *mut c_void);
        if fuse_session == ptr::null() {
            // TODO: better error message?
            stderr().write_line("Failed to create FUSE session\n");
            fail!();
        }
        
        if fuse::fuse_set_signal_handlers(fuse_session) == -1 {
            stderr().write_line("Failed to set FUSE signal handlers");
            fail!();
        }

        fuse::fuse_session_add_chan(fuse_session, fuse_chan);
        fuse::fuse_session_loop(fuse_session);
        fuse::fuse_remove_signal_handlers(fuse_session);
        fuse::fuse_session_remove_chan(fuse_chan);

        fuse::fuse_session_destroy(fuse_session);
        fuse::fuse_unmount(mountpoint as *c_schar, fuse_chan);
        fuse::fuse_opt_free_args(ptr::to_mut_unsafe_ptr(&mut fuse_args));
    };
}


pub fn make_fuse_ll_oper<Ops:FuseLowLevelOps+Send>(ops:&Ops)
    -> fuse::Struct_fuse_lowlevel_ops {
    return fuse::Struct_fuse_lowlevel_ops {
        init: if ops.init_is_implemented() { init_impl } else { ptr::null() },
        destroy: if ops.destroy_is_implemented() { destroy_impl } else { ptr::null() },
        lookup: if ops.lookup_is_implemented() { lookup_impl } else { ptr::null() },
        forget: if ops.forget_is_implemented() { forget_impl } else { ptr::null() },
        getattr: if ops.getattr_is_implemented() { getattr_impl } else { ptr::null() },
        setattr: if ops.setattr_is_implemented() { setattr_impl } else { ptr::null() },
        readlink: if ops.readlink_is_implemented() { readlink_impl } else { ptr::null() },
        mknod: if ops.mknod_is_implemented() { mknod_impl } else { ptr::null() },
        mkdir: if ops.mkdir_is_implemented() { mkdir_impl } else { ptr::null() },
        unlink: if ops.unlink_is_implemented() { unlink_impl } else { ptr::null() },
        rmdir: if ops.rmdir_is_implemented() { rmdir_impl } else { ptr::null() },
        symlink: if ops.symlink_is_implemented() { symlink_impl } else { ptr::null() },
        rename: if ops.rename_is_implemented() { rename_impl } else { ptr::null() },
        link: if ops.link_is_implemented() { link_impl } else { ptr::null() },
        open: if ops.open_is_implemented() { open_impl } else { ptr::null() },
        read: if ops.read_is_implemented() { read_impl } else { ptr::null() },
        write: if ops.write_is_implemented() { write_impl } else { ptr::null() },
        flush: if ops.flush_is_implemented() { flush_impl } else { ptr::null() },
        release: if ops.release_is_implemented() { release_impl } else { ptr::null() },
        fsync: if ops.fsync_is_implemented() { fsync_impl } else { ptr::null() },
        opendir: if ops.opendir_is_implemented() { opendir_impl } else { ptr::null() },
        readdir: if ops.readdir_is_implemented() { readdir_impl } else { ptr::null() },
        releasedir: if ops.releasedir_is_implemented() { releasedir_impl } else { ptr::null() },
        fsyncdir: if ops.fsyncdir_is_implemented() { fsyncdir_impl } else { ptr::null() },
        statfs: if ops.statfs_is_implemented() { statfs_impl } else { ptr::null() },
        setxattr: if ops.setxattr_is_implemented() { setxattr_impl } else { ptr::null() },
        getxattr: if ops.getxattr_is_implemented() { getxattr_impl } else { ptr::null() },
        listxattr: if ops.listxattr_is_implemented() { listxattr_impl } else { ptr::null() },
        removexattr: if ops.removexattr_is_implemented() { removexattr_impl } else { ptr::null() },
        access: if ops.access_is_implemented() { access_impl } else { ptr::null() },
        create: if ops.create_is_implemented() { create_impl } else { ptr::null() },

        // Not implemented yet:
        getlk: ptr::null(),
        setlk: ptr::null(),
        bmap: ptr::null(),
        ioctl: ptr::null(),
        poll: ptr::null(),
        write_buf: ptr::null(),
        retrieve_reply: ptr::null(),
        forget_multi: ptr::null(),
        flock: ptr::null(),
        fallocate: ptr::null(),
    }
}

fn run_for_reply<Ops:FuseLowLevelOps+Send, Reply>(req:fuse::fuse_req_t, 
                                                  reply_success:~fn(req:fuse::fuse_req_t, reply:Reply),
                                                  do_op:~fn(~Ops) -> ErrnoResult<Reply>) {
    unsafe {
        // Need to use a cell to pass ownership of do_op through a closure.
        // This is how spawn_with does it...
        let fns_cell = cell::Cell::new((reply_success, do_op));
        do userdata_to_ops(fuse::fuse_req_userdata(req)) |ops:&Ops| {
            let dupe:~Ops = ~ops.clone();
            let (reply_success, do_op) = fns_cell.take();
            do task().spawn_with((dupe, reply_success, do_op)) |tuple: 
                (~Ops, 
                 ~fn(fuse::fuse_req_t, Reply), 
                 ~fn(~Ops) -> ErrnoResult<Reply>)| {
                let (ops, reply_success, do_op) = tuple;
                match do_op(ops) {
                    Ok(reply) => reply_success(req, reply),
                    Err(errno) => { fuse::fuse_reply_err(req, errno); },
                }
                
            }
        }
    }
}

fn reply_entryparam(req: fuse::fuse_req_t, reply:fuse::Struct_fuse_entry_param) {
    unsafe {
        fuse::fuse_reply_entry(req, ptr::to_unsafe_ptr(&reply));
    }
}

extern fn init_impl<Ops:FuseLowLevelOps+Send>(userdata:*mut c_void, _conn:*fuse::Struct_fuse_conn_info) {
    do userdata_to_ops(userdata) |ops:&Ops| { ops.init() }
}

extern fn destroy_impl<Ops:FuseLowLevelOps+Send>(userdata:*mut c_void) {
    do userdata_to_ops(userdata) |ops:&Ops| { ops.destroy() }
}

extern fn lookup_impl<Ops:FuseLowLevelOps+Send>(req:fuse::fuse_req_t, 
                                           parent:fuse::fuse_ino_t, 
                                           name:*c_schar) {
    do run_for_reply(req, reply_entryparam) |ops:~Ops| {
        unsafe {
            let cstr = CString::new(name,false);
            ops.lookup(parent, cstr.as_bytes().to_str())
        }
    }
}

extern fn forget_impl() { fail!() }

extern fn getattr_impl() { fail!() }

extern fn setattr_impl() { fail!() }

extern fn readlink_impl() { fail!() }

extern fn mknod_impl() { fail!() }

extern fn mkdir_impl() { fail!() }

extern fn unlink_impl() { fail!() }

extern fn rmdir_impl() { fail!() }

extern fn symlink_impl() { fail!() }

extern fn rename_impl() { fail!() }

extern fn link_impl() { fail!() }

extern fn open_impl() { fail!() }

extern fn read_impl() { fail!() }

extern fn write_impl() { fail!() }

extern fn flush_impl() { fail!() }

extern fn release_impl() { fail!() }

extern fn fsync_impl() { fail!() }

extern fn opendir_impl() { fail!() }

extern fn readdir_impl() { fail!() }

extern fn releasedir_impl() { fail!() }

extern fn fsyncdir_impl() { fail!() }

extern fn statfs_impl() { fail!() }

extern fn setxattr_impl() { fail!() }

extern fn getxattr_impl() { fail!() }

extern fn listxattr_impl() { fail!() }

extern fn removexattr_impl() { fail!() }

extern fn access_impl() { fail!() }

extern fn create_impl() { fail!() }
