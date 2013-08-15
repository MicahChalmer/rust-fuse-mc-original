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

/** Struct full of optional functions to implement fuse operations
 *
 * This really should be a trait.  But we can't know at run time which default
 * methods of a trait were overridden, which means we don't know which entries
 * in the Struct_fuse_lowlevel_ops to null out.  FUSE has default behavior for
 * some ops that can't just be invoked from a callback--the only way to get it
 * is to pass NULL for the callback pointers.  So here it is--a struct full of
 * optional closures instead. Argh.
 */
#[deriving(Zero)]
pub struct FuseLowLevelOps {
    init: Option<~fn()>,
    destroy: Option<~fn()>,
    lookup: Option<~fn(parent: fuse::fuse_ino_t, name: &str)
                       -> ErrnoResult<fuse::Struct_fuse_entry_param>>,
    forget: Option<~fn(ino:fuse::fuse_ino_t, nlookup:c_ulong)>,
    getattr: Option<~fn(ino: fuse::fuse_ino_t, flags: c_int)
                        -> ErrnoResult<AttrReply>>,
    setattr: Option<~fn(ino: fuse_ino_t, attrs_to_set:&[AttrToSet], 
                        fh:Option<u64>) -> ErrnoResult<AttrReply>>,
    readlink:Option<~fn(fuse_ino_t) -> ErrnoResult<~str>>,
    mknod: Option<~fn(parent: fuse_ino_t, name: &str, mode: mode_t, 
                      rdev: dev_t) -> ErrnoResult<EntryReply>>,
    mkdir: Option<~fn(parent: fuse_ino_t, name: &str, mode: mode_t) 
        -> ErrnoResult<EntryReply>>,
    // TODO: Using the unit type with result seems kind of goofy, but
    // is done for consistency with the others.  Is this right?
    unlink: Option<~fn(parent: fuse_ino_t, name: &str)
        -> ErrnoResult<()>>,
    rmdir: Option<~fn(parent: fuse_ino_t, name: &str)
        -> ErrnoResult<()>>,
    symlink: Option<~fn(link:&str, parent: fuse_ino_t, name: &str)
        -> ErrnoResult<EntryReply>>,
    rename: Option<~fn(parent: fuse_ino_t, name: &str,
                       newparent: fuse_ino_t, newname: &str)
        -> ErrnoResult<()>>,
    link: Option<~fn(ino: fuse_ino_t, newparent: fuse_ino_t, newname: &str)
        -> ErrnoResult<EntryReply>>,
    open: Option<~fn(ino: fuse_ino_t, flags: c_int) 
        -> ErrnoResult<OpenReply>>,
    read: Option<~fn(ino: fuse::fuse_ino_t, size: size_t, off: off_t, fh: u64)
                     -> ErrnoResult<ReadReply>>,
    // TODO: is writepage a bool, or an actual number that needs to be
    // preserved?
    write: Option<~fn(ino: fuse_ino_t, buf:&[u8], off: off_t, fh: u64,
                      writepage: bool) -> ErrnoResult<size_t>>,
    flush: Option<~fn(ino: fuse_ino_t, lock_owner: u64, fh: u64)
        -> ErrnoResult<()>>,
    release: Option<~fn(ino: fuse_ino_t, flags: c_int, fh: u64)
        -> ErrnoResult<()>>,
    fsync: Option<~fn(ino: fuse_ino_t, datasync: bool, fh: u64)
        -> ErrnoResult<()>>,
    opendir: Option<~fn(ino: fuse_ino_t) -> ErrnoResult<OpenReply>>,
    // TODO: Using a ReadReply would require the impl to do unsafe operations
    // to use fuse_add_direntry.  So even the thin interface needs something
    // else here.
    readdir: Option<~fn(ino: fuse_ino_t, size: size_t, off: off_t, fh: u64)
        -> ErrnoResult<ReadReply>>,
    releasedir: Option<~fn(ino: fuse_ino_t, fh: u64) -> ErrnoResult<()>>,
    fsyncdir: Option<~fn(ino: fuse_ino_t, datasync: bool, fh: u64)
        -> ErrnoResult<()>>,
    statfs: Option<~fn(ino: fuse_ino_t) -> ErrnoResult<Struct_statvfs>>,
    setxattr: Option<~fn(ino: fuse_ino_t, name: &str, value: &[u8], flags: int)
        -> ErrnoResult<()>>,
    // TODO: examine this--ReadReply may not be appropraite here
    getxattr: Option<~fn(ino: fuse_ino_t, name: &str, size: size_t)
        -> ErrnoResult<ReadReply>>,
    // Called on getxattr with size of zero (meaning a query of total size)
    getxattr_size: Option<~fn(ino: fuse_ino_t, name: &str)
        -> ErrnoResult<size_t>>,
    // TODO: examine this--ReadReply may not be appropraite here
    listxattr: Option<~fn(ino: fuse_ino_t, name: &str, size: size_t)
        -> ErrnoResult<ReadReply>>,
    // Called on listxattr with size of zero (meaning a query of total size)
    listxattr_size: Option<~fn(ino: fuse_ino_t, name: &str)
        -> ErrnoResult<size_t>>,
    removexattr: Option<~fn(ino: fuse_ino_t, name: &str) -> ErrnoResult<()>>,
    access: Option<~fn(ino: fuse_ino_t, mask: c_int) -> ErrnoResult<()>>,
    create: Option<~fn(ino: fuse_ino_t, parent: fuse_ino_t, name: &str,
                       mode: mode_t, flags: c_int) -> ErrnoResult<OpenReply>>,

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
