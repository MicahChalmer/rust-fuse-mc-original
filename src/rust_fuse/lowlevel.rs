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
    time_t,
    uid_t,
    EIO
};
use std::sys::size_of;
use std::cast::transmute;
use std::ptr;
use std::vec;
use std::task::{task, DefaultScheduler, SingleThreaded, TaskResult};
use std::task;
use std::c_str::{CString,ToCStr};
use std::default::Default;
use std::bool::to_bit;
use std::cmp;
use std::iter::AdditiveIterator;
use ffi::*;
use super::stat::stat::arch::default_stat;
use std::libc;
use std::util::NonCopyable;
use std::cell::Cell;
use std::str;
use std::rt::io::process::{Process, ProcessConfig, Ignored};
use extra::arc::Arc;

pub use ffi::{fuse_ino_t,Struct_fuse_entry_param};

/// Information to be returned from open
#[deriving(Default)]
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
    attr: libc::stat,
    attr_timeout: c_double
}

pub enum AttrToSet {
    Mode(mode_t),
    Uid(uid_t),
    Gid(gid_t),
    Size(off_t),
    Atime(time_t),
    Mtime(time_t),

    /// This is an instruction to set atime to the current time
    Atime_now,

    /// This is an instruction to set mtime to the current time
    Mtime_now,
}

pub enum ReadReply {
    // TODO: support iov and fuse_bufvec type replies
    // types of implementation
    DataBuffer(~[u8]),
    EOF
}

pub struct DirEntry {
    ino: fuse_ino_t,
    name: CString,
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
 * Trait that defines the filesystem.  Override each method to implement the
 * corresponding filesystem operation.  See the FUSE docs for a description of
 * each.  The rust_fuse wrapper will run each operation in its own task, so
 * even though the API is blocking they will run in parallel (subject to rust's
 * default scheduling.)
 *
 * For each operation implemented, it is necessary to also implement the
 * corresponding _is_implemented method and return true--i.e. if you implement
 * `lookup` you need to implement `lookup_is_implemented` and so on.  Each
 * "is_implemented" function is called only once, before the filesystem is
 * mounted, and if it returns false, the corresponding FS operation method will
 * not be called, and the FUSE default behavior will be used in its place.
 * It's a shame that all this is necessary, but rust has no reflection to allow
 * the wrapper to know which fields of the `fuse_lowlevel_ops` struct to pass
 * NULL into in the C API.  Passing NULL into the struct is the only way to get
 * the FUSE default behavior--there is no way to invoke it from a callback.
 * 
 * The callbacks get an immutable reference to self, and can be called in
 * parallel on the same object.
 */
pub trait FuseLowLevelOps {
    /// Called when the file system is mounted and ready.
    fn init(&self) { }
    // Called when the file system has been unmounted.
    fn destroy(&self) { }

    fn lookup(&self, _parent: fuse_ino_t, _name: &CString)
              -> ErrnoResult<EntryReply> { fail!() }
    fn lookup_is_implemented(&self) -> bool { false }
    fn forget(&self, _ino:fuse_ino_t, _nlookup:c_ulong) { fail!() }
    fn forget_is_implemented(&self) -> bool { false }
    fn getattr(&self, _ino: fuse_ino_t) -> ErrnoResult<AttrReply> { fail!() }
    fn getattr_is_implemented(&self) -> bool { false }
    fn setattr(&self, _ino: fuse_ino_t, __attrs_toset:&[AttrToSet], _fh:Option<u64>)
               -> ErrnoResult<AttrReply> { fail!() }
    fn setattr_is_implemented(&self) -> bool { false }
    fn readlink(&self, _ino: fuse_ino_t) -> ErrnoResult<~str> { fail!() }
    fn readlink_is_implemented(&self) -> bool { false }
    fn mknod(&self, _parent: fuse_ino_t, _name: &CString, _mode: mode_t, _rdev: dev_t) 
             -> ErrnoResult<EntryReply> { fail!() }
    fn mknod_is_implemented(&self) -> bool { false }
    fn mkdir(&self, _parent: fuse_ino_t, _name: &CString, _mode: mode_t)
             -> ErrnoResult<EntryReply> { fail!() }
    fn mkdir_is_implemented(&self) -> bool { false }
    fn unlink(&self, _parent: fuse_ino_t, _name: &CString)
              -> ErrnoResult<()> { fail!() }
    fn unlink_is_implemented(&self) -> bool { false }
    fn rmdir(&self, _parent: fuse_ino_t, _name: &CString) -> ErrnoResult<()> { fail!() }
    fn rmdir_is_implemented(&self) -> bool { false }
    fn symlink(&self, _link:&CString, _parent: fuse_ino_t, _name: &CString)
               -> ErrnoResult<EntryReply> { fail!() }
    fn symlink_is_implemented(&self) -> bool { false }
    fn rename(&self, _parent: fuse_ino_t, _name: &CString, _newparent: fuse_ino_t, 
              _newname: &CString) -> ErrnoResult<()> { fail!() }
    fn rename_is_implemented(&self) -> bool { false }
    fn link(&self, _ino: fuse_ino_t, _newparent: fuse_ino_t, _newname: &CString)
            -> ErrnoResult<EntryReply> { fail!() }
    fn link_is_implemented(&self) -> bool { false }
    fn open(&self, _ino: fuse_ino_t, _flags: c_int)
            -> ErrnoResult<OpenReply> { fail!() }
    fn open_is_implemented(&self) -> bool { false }
    fn read(&self, _ino: fuse_ino_t, _size: size_t, _off: off_t, _fh: u64)
            -> ErrnoResult<ReadReply> { fail!() }
    fn read_is_implemented(&self) -> bool { false }
    fn write(&self, _ino: fuse_ino_t, _buf:&[u8], _off: off_t, _fh: u64, _writepage: bool)
             -> ErrnoResult<size_t> { fail!() }
    fn write_is_implemented(&self) -> bool { false }
    fn flush(&self, _ino: fuse_ino_t, __lockowner: u64, _fh: u64)
             -> ErrnoResult<()> { fail!() }
    fn flush_is_implemented(&self) -> bool { false }
    fn release(&self, _ino: fuse_ino_t, _flags: c_int, _fh: u64)
               -> ErrnoResult<()> { fail!() }
    fn release_is_implemented(&self) -> bool { false }
    fn fsync(&self, _ino: fuse_ino_t, _datasync: bool, _fh: u64)
             -> ErrnoResult<()> { fail!() }
    fn fsync_is_implemented(&self) -> bool { false }
    fn opendir(&self, _ino: fuse_ino_t)
               -> ErrnoResult<OpenReply> { fail!() }
    fn opendir_is_implemented(&self) -> bool { false }
    fn readdir(&self, _ino: fuse_ino_t, _size: size_t, _off: off_t, _fh: u64)
               -> ErrnoResult<ReaddirReply> { fail!() }
    fn readdir_is_implemented(&self) -> bool { false }
    fn releasedir(&self, _ino: fuse_ino_t, _fh: u64)
                  -> ErrnoResult<()> { fail!() }
    fn releasedir_is_implemented(&self) -> bool { false }
    fn fsyncdir(&self, _ino: fuse_ino_t, _datasync: bool, _fh: u64)
                -> ErrnoResult<()> { fail!() }
    fn fsyncdir_is_implemented(&self) -> bool { false }
    fn statfs(&self, _ino: fuse_ino_t) -> ErrnoResult<Struct_statvfs> { fail!() }
    fn statfs_is_implemented(&self) -> bool { false }
    fn setxattr(&self, _ino: fuse_ino_t, _name: &CString, _value: &[u8], _flags: c_int)
                -> ErrnoResult<()> { fail!() }
    fn setxattr_is_implemented(&self) -> bool { false }
    // _TODO: examine this--ReadReply may not be appropraite here
    fn getxattr(&self, _ino: fuse_ino_t, _name: &CString, _size: size_t)
                -> ErrnoResult<ReadReply> { fail!() }
    // Called on getxattr with size of zero (meaning a query of total size)
    fn getxattr_size(&self, _ino: fuse_ino_t, _name: &CString)
                     -> ErrnoResult<size_t>{ fail!() }
    fn getxattr_is_implemented(&self) -> bool { false }
    // _TODO: examine this--ReadReply may not be appropraite here
    fn listxattr(&self, _ino: fuse_ino_t, _size: size_t)
                 -> ErrnoResult<ReadReply> { fail!() }
    // Called on listxattr with size of zero (meaning a query of total size)
    fn listxattr_size(&self, _ino: fuse_ino_t) -> ErrnoResult<size_t> { fail!() }
    fn listxattr_is_implemented(&self) -> bool { false }
    fn removexattr(&self, _ino: fuse_ino_t, _name: &CString) 
                   -> ErrnoResult<()> { fail!() }
    fn removexattr_is_implemented(&self) -> bool { false }
    fn access(&self, _ino: fuse_ino_t, _mask: c_int) -> ErrnoResult<()> { fail!() }
    fn access_is_implemented(&self) -> bool { false }
    fn create(&self, _parent: fuse_ino_t, _name: &CString, _mode: mode_t, _flags: c_int)
              -> ErrnoResult<CreateReply> { fail!() }
    fn create_is_implemented(&self) -> bool { false }
    // _TODO: The following still need _implementing:
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
}


/// Options for mounting the file system
pub struct FuseMountOptions {
    /// Command line arguments to pass through to the FUSE API.  See the
    /// `fuse_ll_help` function in the FUSE source for what can go here.
    args:~[~[u8]]
}
impl Default for FuseMountOptions {
    fn default() -> FuseMountOptions {
        FuseMountOptions{
            args: ~[]
        }
    }
}

/**
 * Object representing the mounting of a path via FUSE
 *
 * Creating a `FuseMount` mounts the path at `mount_point` via this process
 * with the functions specified in `ops`.  The path will be mounted for as long
 * as this object is alive, or until the path is unmounted externally via
 * `fusermount -u` or `umount`.
 */
pub struct FuseMount {
    // A message appearing here means we're done
    priv finish_port:Port<TaskResult>,
    priv mounted:bool,
    priv session:~FuseSession,
    priv nocopies: NonCopyable
}
impl FuseMount {

    /// Mount the FUSE file system using the functions in `ops`, with options
    /// (including the mount point) taken from `options.args`.  This function
    /// will fail if the options are not valid as per FUSE, or if FUSE fails to
    /// mount.
    pub fn new(options:~FuseMountOptions,ops:~FuseLowLevelOps:Send+Freeze)
               -> ~FuseMount {
        // The C API needs its own OS thread because it will block.  We want to
        // run all of the filesystem commands we get in parallel on their own
        // rust tasks, but we don't want to spawn a new OS thread for each of
        // them.  So instead, we have this: the C API gets its own task on its
        // own OS thread, and does nothing but send commands through a chan to
        // the dispatch task.  The dispatch task is running on the same
        // scheduler as the task calling `FuseMount::new`.  Its job is to
        // receive commands and start a task (again on the default scheduler)
        // in which to run each one.

        let FuseMountOptions{args:args} = *options;

        let (dispatch_port, dispatch_chan) = stream::<~FSOperation>();
        let (finish_port, finish_chan) = stream::<TaskResult>();
        // This is how we get the session pointer out of the C API thread,
        // so that we can use it to end the session later.
        let (session_port, session_chan) = 
            stream::<~FuseSession>();

        // Spawn the C API task
        let mut c_api_task = task();
        c_api_task.sched_mode(SingleThreaded);
        c_api_task.linked();
        c_api_task.name(format!("FUSE C API - {:?}",
                                args.map(|v| str::from_utf8(*v))));
        c_api_task.opts.notify_chan = Some(finish_chan);
        let userdata = ~FuseUserData{
            args: args, 
            ops:Arc::new(ops),
            dispatch_chan:dispatch_chan,
            session_chan: session_chan,
            session: Cell::new_empty()
        };
        c_api_task.spawn_with(userdata,c_api_loop);
        
        // Receive the session.  The C API task will either send it or fail
        // (and if it fails, we fail with it, thanks to linked failure.)
        let session = session_port.recv();

        // Spawn the dispatch task
        let mut dispatch_task = task();
        c_api_task.sched_mode(DefaultScheduler);
        dispatch_task.linked();
        dispatch_task.name(format!("FUSE dispatch - {:s}",
                                   session.mount_point.to_str()));
        do dispatch_task.spawn_with(dispatch_port) |dispatch_port| {
            'dispatch: loop {
                // try_recv won't deschedule if the port is closed, so
                // we need to explicitly do this
                task::deschedule();
                match dispatch_port.try_recv() {
                    Some(fsop) => {
                        do task().spawn_with(fsop) |fsop| {
                            let req = fsop.req;
                            let result = do task::try {
                                (fsop.operation_fn)(req)
                            };
                            if result.is_err() {
                                reply_failure_err(req);
                            }
                        };
                    },
                    None => break 'dispatch
                };
            }
            debug!("done with dispatch");
        }

        ~FuseMount{
            finish_port: finish_port,
            mounted: true,
            session:session,
            nocopies: NonCopyable::new()
        }
    }

    /// Return true if the filesystem is still mounted, false if not. It could
    /// be unmounted while the object is still alive if something external
    /// unmounted it, or from a call to `unmount`.
    pub fn is_mounted(&self) -> bool {
        self.mounted && !self.finish_port.peek()
    }

    /// Block until the filesystem is unmounted
    pub fn finish(&mut self) {
        if self.mounted {
            debug!("Waiting to finish mount of %s",
                   self.mount_point().to_str());
            self.finish_port.recv();
            debug!("Mount finished: %s",
                   self.mount_point().to_str());
            self.mounted = false;
        }
    }

    /// Unmount the file system
    #[fixed_stack_segment]
    pub fn unmount(&mut self) {
        if self.mounted {
            debug!("Unmounting %s", self.mount_point().to_str());
            // TODO: once signal handling exists, signal the C API thread
            // instead of using an external process.
            unmount_via_external_process(self.mount_point());
            self.finish();
        }
    }

    pub fn mount_point<'a>(&'a self) -> &'a PosixPath {
        &self.session.mount_point
    }
}
impl Drop for FuseMount {
    fn drop(&mut self) {
        debug!("Destroying mounter for %s", self.mount_point().to_str());
        self.unmount();
    }
}

#[cfg(target_os = "linux")]
mod ext_unmount {
    use std::path::PosixPath;
    pub static EXT_UNMOUNT_PROG:&'static str = "fusermount";
    pub fn ext_unmount_args(mount_point:&PosixPath) -> ~[~str] {
        ~[~"-u", mount_point.to_str()]
    }
}
#[cfg(target_os = "macos")]
mod ext_unmount {
    use std::path::PosixPath;
    pub static EXT_UNMOUNT_PROG:&'static str = "umount";
    pub fn ext_unmount_args(mount_point:&PosixPath) -> ~[~str] {
        ~[mount_point.to_str()]
    }
}

fn unmount_via_external_process(mount_point:&PosixPath) {
    let args = self::ext_unmount::ext_unmount_args(mount_point);
    let io = ~[Ignored, Ignored, Ignored];
    let cwd = Some("/");
    let _proc = Process::new(ProcessConfig{
        program: self::ext_unmount::EXT_UNMOUNT_PROG,
        args: args,
        env: None,
        cwd: cwd,
        io: io
    });
}

// The FUSE userdata pointer will point to one of these.  The c extern fns
// use it to get back into the correct corresponding rust tasks.
struct FuseUserData {
    ops: Arc<~FuseLowLevelOps:Send+Freeze>,
    args: ~[~[u8]],
    // Send FS command functions through here to be dispatched to new tasks on
    // the right scheduler
    dispatch_chan:Chan<~FSOperation>,
    // During initialization, we need to send the session through the session
    // chan
    session:Cell<~FuseSession>,
    session_chan:Chan<~FuseSession>
}

struct FSOperation {
    operation_fn: ~fn(fuse_req_t),
    req: fuse_req_t
}

struct FuseSession {
    session: *mut Struct_fuse_session,
    mount_point: PosixPath
}

fn cstr_as_bytes_no_term<'a>(cs:&'a CString) -> &'a[u8] {
    let ab = cs.as_bytes();
    ab.slice_to(cmp::max(ab.len()-1,0))
}

#[fixed_stack_segment]
pub fn c_api_loop(userdata:~FuseUserData) {
    unsafe {
        let args = &(userdata.args);
        let args_c_strs = args.map(|vec| vec.to_c_str());
        let args_ptrs = args_c_strs.map(|cstr| cstr.with_ref(|ptr| ptr));
        let mut fuse_args = Struct_fuse_args {
            argv: transmute(vec::raw::to_ptr(args_ptrs)),
            argc: args.len() as c_int,
            allocated: 0
        };

        let mut mount_point:*mut c_schar = ptr::mut_null();
        if fuse_parse_cmdline(ptr::to_mut_unsafe_ptr(&mut fuse_args),
                              ptr::to_mut_unsafe_ptr(&mut mount_point),
                              ptr::mut_null(), // multithreaded--we ignore
                              ptr::mut_null() // foreground--ignore (for now)
                              ) == -1 {
            fail!("Invalid command line options");
        }

        // The fuse_chan here is a FUSE C API object, not to be confused
        // with a rust stream's "chan"
        let fuse_chan = fuse_mount(mount_point as *c_schar,
                                   ptr::to_mut_unsafe_ptr(&mut fuse_args));
        if fuse_chan == ptr::mut_null() {
            fail!("Failed to mount");
        }

        let llo = make_fuse_ll_oper(*userdata.ops.get());
        let fuse_session = fuse_lowlevel_new(
            ptr::to_mut_unsafe_ptr(&mut fuse_args),
            ptr::to_unsafe_ptr(&llo),
            size_of::<Struct_fuse_lowlevel_ops>() as size_t,
            ptr::to_unsafe_ptr(&userdata) as *mut c_void);
        if fuse_session == ptr::mut_null() {
            fail!("Failed to create FUSE session");
        }
        let mountpoint_cstr = CString::new(mount_point as *c_schar,false);
        let mountpoint_str = str::from_utf8(
            cstr_as_bytes_no_term(&mountpoint_cstr));
        userdata.session.put_back(~FuseSession{
                session:fuse_session,
                mount_point:PosixPath(mountpoint_str)
            });

        fuse_session_add_chan(fuse_session, fuse_chan);
        fuse_session_loop(fuse_session);
        debug!("Done with C API fuse session");
        fuse_session_remove_chan(fuse_chan);

        fuse_session_destroy(fuse_session);
        fuse_unmount(mount_point as *c_schar, fuse_chan);
        debug!("Done with C API fn");
    };
}

pub fn make_fuse_ll_oper(ops:&FuseLowLevelOps)
                         -> Struct_fuse_lowlevel_ops {
    return Struct_fuse_lowlevel_ops {
        init: Some(init_impl),
        destroy: Some(destroy_impl),

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
    }
}

#[fixed_stack_segment]
fn userdata_ptr_from_req(req:fuse_req_t) -> *mut c_void {
    unsafe {
        fuse_req_userdata(req)
    }
}

/*
 * Run a function with a borrowed pointer to the Ops pointed to by the given
 * userdata pointer.  The "arg" parameter is for passing extra data into the
 * function a la task::spawn_with (needed to push owned pointers into the
 * closure)
 */
fn userdata_from_ptr<T, U>(userdata_ptr:*mut c_void, arg:T,
                         func:&fn(&FuseUserData, T) -> U) -> U {
    unsafe {
        func(*(userdata_ptr as *~FuseUserData), arg)
    }
}

fn get_fuse_userdata<T, U>(req:fuse_req_t, arg:T,
                           func:&fn(&FuseUserData, T) -> U) -> U {
    let userdata = userdata_ptr_from_req(req);
    userdata_from_ptr(userdata, arg, func)
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

fn send_to_dispatch<T:Send>(req:fuse_req_t, arg:T, 
                       blk:~fn(&FuseUserData, T)) {
    // Toss use this to pass ownedship of arg and blk deep into the nested
    // closures...
    do get_fuse_userdata(req, (arg, blk)) |userdata, (arg, blk)| {
        let c = Cell::new((arg, blk));
        userdata.dispatch_chan.send(~FSOperation{
            operation_fn: |req| {
                    do get_fuse_userdata(req, ()) 
                        |userdata, ()| {
                        let (arg, blk) = c.take();
                        blk(userdata, arg);
                    }
                },
                req: req
            });
    }
}

fn run_for_reply<T>(req:fuse_req_t, reply_success:ReplySuccessFn<T>,
                    do_op:~fn(&FuseLowLevelOps) -> ErrnoResult<T>) {
    do send_to_dispatch(req, (do_op, reply_success))
        |userdata, (do_op, reply_success)| {
        send_fuse_reply(do_op(*userdata.ops.get()), req, reply_success);
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
        fuse_reply_attr(req, ptr::to_unsafe_ptr(&reply.attr),
                        reply.attr_timeout);
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

#[fixed_stack_segment]
fn reply_failure_err(req:fuse_req_t)
{
    unsafe {
        fuse_reply_err(req, EIO);
    }
}

fn openreply_to_fileinfo(reply: OpenReply) -> Struct_fuse_file_info {
    Struct_fuse_file_info{
        direct_io: to_bit(reply.direct_io) as c_uint,
        keep_cache: to_bit(reply.keep_cache) as c_uint,
        fh: reply.fh,
        ..Default::default()
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
            let mut lengths = entries.iter().map(|x| x.name.as_bytes().len()
                                                 as size_t);
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
                    let added_size = do entry.name.with_ref |name_cstr| {
                        let stbuf = libc::stat{
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
    do userdata_from_ptr(userdata, ()) |userdata, _| {
        (*userdata.ops.get()).init();
        userdata.session_chan.send(userdata.session.take());
    }
}

extern fn destroy_impl(userdata:*mut c_void) {
    do userdata_from_ptr(userdata, ()) |userdata, _| {
        userdata.ops.get().destroy();
    }
}

extern fn lookup_impl(req:fuse_req_t,  parent:fuse_ino_t, name:*c_schar) {
    do run_for_reply(req, reply_entryparam) |ops| {
        unsafe { ops.lookup(parent, &CString::new(name, false)) }
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

extern fn setattr_impl(req: fuse_req_t, ino: fuse_ino_t, attr:*libc::stat,
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
            if to_set & FUSE_SET_ATTR_MODE != 0 {
                attrs_to_set.push(Mode((*attr).st_mode))
            }
            if to_set & FUSE_SET_ATTR_UID != 0 {
                attrs_to_set.push(Uid((*attr).st_uid))
            }
            if to_set & FUSE_SET_ATTR_GID != 0 {
                attrs_to_set.push(Gid((*attr).st_gid))
            }
            if to_set & FUSE_SET_ATTR_SIZE != 0 {
                attrs_to_set.push(Size((*attr).st_size))
            }
            if to_set & FUSE_SET_ATTR_ATIME != 0 {
                attrs_to_set.push(Atime((*attr).st_atime))
            }
            if to_set & FUSE_SET_ATTR_MTIME != 0 {
                attrs_to_set.push(Mtime((*attr).st_mtime))
            }
            if to_set & FUSE_SET_ATTR_ATIME_NOW != 0 {
                attrs_to_set.push(Atime_now)
            }
            if to_set & FUSE_SET_ATTR_MTIME_NOW != 0 {
                attrs_to_set.push(Mtime_now)
            }
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
        unsafe { ops.mknod(parent, &CString::new(name,false), mode, rdev) }
    }
}

extern fn mkdir_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar,
                     mode:mode_t) {
    do run_for_reply(req, reply_entryparam) |ops| {
        unsafe { ops.mkdir(parent, &CString::new(name,false), mode) }        
    }
}

extern fn unlink_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe { ops.unlink(parent, &CString::new(name,false)) }        
    }
}

extern fn rmdir_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe { ops.rmdir(parent, &CString::new(name,false)) }
    }
}

extern fn symlink_impl(req: fuse_req_t, link: *c_schar, parent: fuse_ino_t,
                       name: *c_schar) {
    do run_for_reply(req, reply_entryparam) |ops| {
        unsafe {
            ops.symlink(&CString::new(link,false), parent, 
                        &CString::new(name,false))
        }        
    }
}

extern fn rename_impl(req: fuse_req_t, parent: fuse_ino_t, name: *c_schar,
                      newparent: fuse_ino_t, newname: *c_schar) {
    do run_for_reply(req, reply_zero_err) |ops| {
        unsafe {
            ops.rename(parent, &CString::new(name,false), newparent,
                       &CString::new(newname,false))
        }        
    }
}

extern fn link_impl(req: fuse_req_t, ino: fuse_ino_t, newparent: fuse_ino_t,
                    newname: *c_schar) {
    do run_for_reply(req, reply_entryparam) |ops| {
        unsafe {
            ops.link(ino, newparent, &CString::new(newname,false))
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

extern fn readdir_impl(req: fuse_req_t, ino: fuse_ino_t, size: size_t,
                       off: off_t, fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_readdir) |ops| {
        unsafe {
            ops.readdir(ino, size, off, (*fi).fh)
        }.and_then(|rr| Ok((size, rr)))
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
                ops.setxattr(ino, &CString::new(name,false), vec, flags)
            }
        }
    }
}

extern fn getxattr_impl(req: fuse_req_t, ino: fuse_ino_t, name: *c_schar,
                        size: size_t) {
    if size == 0 {
        do run_for_reply(req, reply_xattr) |ops| {
            unsafe { ops.getxattr_size(ino, &CString::new(name,false)) }
        }
    } else {
        do run_for_reply(req, reply_read) |ops| {
            unsafe { ops.getxattr(ino, &CString::new(name,false), size) }
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
        unsafe { ops.removexattr(ino, &CString::new(name,false)) }       
    }
}

extern fn access_impl(req: fuse_req_t,
                      ino: fuse_ino_t, mask: c_int) {
    do run_for_reply(req, reply_zero_err) |ops| {
        ops.access(ino, mask)
    }
}

extern fn create_impl(req: fuse_req_t, parent: fuse_ino_t, name: *c_schar,
                      mode: mode_t, fi: *Struct_fuse_file_info) {
    do run_for_reply(req, reply_create) |ops| {
        unsafe {
            ops.create(parent, &CString::new(name,false), mode, (*fi).flags)
        }
    }
}
