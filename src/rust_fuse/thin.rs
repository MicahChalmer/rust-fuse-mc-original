use std::libc::{
    c_char,
    c_double,
    c_int,
    c_uint,
    c_ulong,
    c_void,
    dev_t,
    gid_t,
    mode_t,
    off_t,
    pid_t,
    size_t,
    stat,
    time_t,
    uid_t,
};
use fuse::*;
use statvfs::Struct_statvfs;
use std::io::stderr;
use std::sys::size_of;
use std::vec::raw::to_ptr;
use std::cast::transmute;
use std::ptr;

mod fuse;
mod statvfs;

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
    ino: fuse_ino_t,
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
pub trait FuseLowLevelOps {
    fn init() { fail!() }
    fn init_is_implemented() -> bool { false }
    fn destroy() { fail!() }
    fn destroy_is_implemented() -> bool { false }
    fn lookup(parent: fuse_ino_t, name: &str) -> ErrnoResult<Struct_fuse_entry_param> { fail!() }
    fn lookup_is_implemented() -> bool { false }
    fn forget(ino:fuse_ino_t, nlookup:c_ulong) { fail!() }
    fn forget_is_implemented() -> bool { false }
    fn getattr(ino: fuse_ino_t, flags: c_int) -> ErrnoResult<AttrReply> { fail!() }
    fn getattr_is_implemented() -> bool { false }
    fn setattr(ino: fuse_ino_t, attrs_to_set:&[AttrToSet], fh:Option<u64>) -> ErrnoResult<AttrReply> { fail!() }
    fn setattr_is_implemented() -> bool { false }
    fn readlink(ino: fuse_ino_t) -> ErrnoResult<~str> { fail!() }
    fn readlink_is_implemented() -> bool { false }
    fn mknod(parent: fuse_ino_t, name: &str, mode: mode_t, rdev: dev_t) -> ErrnoResult<EntryReply> { fail!() }
    fn mknod_is_implemented() -> bool { false }
    fn mkdir(parent: fuse_ino_t, name: &str, mode: mode_t) -> ErrnoResult<EntryReply> { fail!() }
    fn mkdir_is_implemented() -> bool { false }
    // TODO: Using the unit type with result seems kind of goofy, but;
    // is done for consistency with the others.  Is this right?;
    fn unlink(parent: fuse_ino_t, name: &str) -> ErrnoResult<()> { fail!() }
    fn unlink_is_implemented() -> bool { false }
    fn rmdir(parent: fuse_ino_t, name: &str) -> ErrnoResult<()> { fail!() }
    fn rmdir_is_implemented() -> bool { false }
    fn symlink(link:&str, parent: fuse_ino_t, name: &str) -> ErrnoResult<EntryReply> { fail!() }
    fn symlink_is_implemented() -> bool { false }
    fn rename(parent: fuse_ino_t, name: &str, newparent: fuse_ino_t, newname: &str) -> ErrnoResult<()> { fail!() }
    fn rename_is_implemented() -> bool { false }
    fn link(ino: fuse_ino_t, newparent: fuse_ino_t, newname: &str) -> ErrnoResult<EntryReply> { fail!() }
    fn link_is_implemented() -> bool { false }
    fn open(ino: fuse_ino_t, flags: c_int) -> ErrnoResult<OpenReply> { fail!() }
    fn open_is_implemented() -> bool { false }
    fn read(ino: fuse_ino_t, size: size_t, off: off_t, fh: u64) -> ErrnoResult<ReadReply> { fail!() }
    fn read_is_implemented() -> bool { false }
    // TODO: is writepage a bool, or an actual number that needs to be;
    // preserved?;
    fn write(ino: fuse_ino_t, buf:&[u8], off: off_t, fh: u64, writepage: bool) -> ErrnoResult<size_t> { fail!() }
    fn write_is_implemented() -> bool { false }
    fn flush(ino: fuse_ino_t, lock_owner: u64, fh: u64) -> ErrnoResult<()> { fail!() }
    fn flush_is_implemented() -> bool { false }
    fn release(ino: fuse_ino_t, flags: c_int, fh: u64) -> ErrnoResult<()> { fail!() }
    fn release_is_implemented() -> bool { false }
    fn fsync(ino: fuse_ino_t, datasync: bool, fh: u64) -> ErrnoResult<()> { fail!() }
    fn fsync_is_implemented() -> bool { false }
    fn opendir(ino: fuse_ino_t) -> ErrnoResult<OpenReply> { fail!() }
    fn opendir_is_implemented() -> bool { false }
    // TODO: Using a ReadReply would require the impl to do unsafe operations;
    // to use fuse_add_direntry.  So even the thin interface needs something;
    // else here.;
    fn readdir(ino: fuse_ino_t, size: size_t, off: off_t, fh: u64) -> ErrnoResult<ReadReply> { fail!() }
    fn readdir_is_implemented() -> bool { false }
    fn releasedir(ino: fuse_ino_t, fh: u64) -> ErrnoResult<()> { fail!() }
    fn releasedir_is_implemented() -> bool { false }
    fn fsyncdir(ino: fuse_ino_t, datasync: bool, fh: u64) -> ErrnoResult<()> { fail!() }
    fn fsyncdir_is_implemented() -> bool { false }
    fn statfs(ino: fuse_ino_t) -> ErrnoResult<Struct_statvfs> { fail!() }
    fn statfs_is_implemented() -> bool { false }
    fn setxattr(ino: fuse_ino_t, name: &str, value: &[u8], flags: int) -> ErrnoResult<()> { fail!() }
    fn setxattr_is_implemented() -> bool { false }
    // TODO: examine this--ReadReply may not be appropraite here;
    fn getxattr(ino: fuse_ino_t, name: &str, size: size_t) -> ErrnoResult<ReadReply> { fail!() }
    fn getxattr_is_implemented() -> bool { false }
    // Called on getxattr with size of zero (meaning a query of total size);
    fn getxattr_size(ino: fuse_ino_t, name: &str) -> ErrnoResult<size_t> { fail!() }
    fn getxattr_size_is_implemented() -> bool { false }
    // TODO: examine this--ReadReply may not be appropraite here;
    fn listxattr(ino: fuse_ino_t, name: &str, size: size_t) -> ErrnoResult<ReadReply> { fail!() }
    fn listxattr_is_implemented() -> bool { false }
    // Called on listxattr with size of zero (meaning a query of total size);
    fn listxattr_size(ino: fuse_ino_t, name: &str) -> ErrnoResult<size_t> { fail!() }
    fn listxattr_size_is_implemented() -> bool { false }
    fn removexattr(ino: fuse_ino_t, name: &str) -> ErrnoResult<()> { fail!() }
    fn removexattr_is_implemented() -> bool { false }
    fn access(ino: fuse_ino_t, mask: c_int) -> ErrnoResult<()> { fail!() }
    fn access_is_implemented() -> bool { false }
    fn create(ino: fuse_ino_t, parent: fuse_ino_t, name: &str, mode: mode_t, flags: c_int) -> ErrnoResult<OpenReply> { fail!() }
    fn create_is_implemented() -> bool { false }

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

extern { fn make_fuse_ll_oper<T:FuseLowLevelOps>(ops:&T) -> Struct_fuse_lowlevel_ops; }

pub fn fuse_main_thin<Ops:FuseLowLevelOps>(args:~[~str], ops:~Ops) {
    unsafe {
        let arg_c_strs_ptrs: ~[*c_char] = args.map(|s| s.to_c_str().unwrap() );
        let fuse_args = Struct_fuse_args {
            argc: transmute(to_ptr(arg_c_strs_ptrs)),
            argc: args.len() as c_int,
            allocated: 0
        };
        let mut mountpoint:*c_char = ptr::null();
        if fuse_parse_cmdline(to_ptr(&fuse_args),
                              to_ptr(&mountpoint),
                              ptr::null(), // multithreaded--we ignore
                              ptr::null() // foreground--we ignore (for now)
                              ) == -1 {
            return;
        }
        
        let fuse_chan = fuse_mount(mountpoint, to_ptr(&fuse_args));
        if fuse_chan == ptr::null() {
            // TODO: better error message?
            stderr().write("Failed to mount\n");
            fail!();
        }
        
        let llo = make_fuse_ll_oper(ops);
        let fuse_session = fuse_lowlevel_new(to_ptr(&fuse_args),
                                             to_ptr(&llo),
                                             size_of::<Struct_fuse_lowlevel_ops>(),
                                             transmute(&ops));
        if fuse_session == ptr::null() {
            // TODO: better error message?
            stderr().write("Failed to create FUSE session\n");
            fail!();
        }
        
        if fuse_set_signal_handlers(fuse_session) == -1 {
            stderr().write("Failed to set FUSE signal handlers");
            fail!();
        }

        fuse_session_add_chan(fuse_session, fuse_chan);
        fuse_session_loop(fuse_session);
        fuse_remove_signal_handlers(fuse_session);
        fuse_session_remove_chan(fuse_chan);

        fuse_session_destroy(fuse_session);
        fuse_unmount(mountpoint, fuse_chan);
        fuse_opt_free_args(to_ptr(&fuse_args));
    };
}
