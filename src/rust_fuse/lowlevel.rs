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
use std::comm::{oneshot, ChanOne};
use std::str;

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
 * Structure of callbacks to pass into fuse_main.
 *
 * It would be best if this could be a trait, and a user could just implement
 * the methods as desired, leaving the rest as defaults.  Unfortunately that's
 * not quite possible.  FUSE has default behavior for some ops that can't just
 * be invoked from a callback--the only way to get it is to pass NULL for the
 * callback pointers into the fuse_lowlevel_ops structure.  But we can't know
 * at run time which default methods of a trait were overridden, which means we
 * don't know which entries in the Struct_fuse_lowlevel_ops to null out.
 *
 * Instead, we've got a struct full of optional fns, equivalent to the struct
 * of nullable function pointers in C.  It's the best we can do without some
 * sort of reflection API that rust doesn't have, or a way to call the FUSE
 * default behavior from a callback, which FUSE does not have.
 *
 */
#[deriving(Default)]
pub struct FuseLowLevelOps {
    init: Option<~fn()>,
    destroy: Option<~fn()>,
    lookup: Option<~fn(parent: fuse_ino_t, name: &CString)
                             -> ErrnoResult<EntryReply>>,
    forget: Option<~fn(ino:fuse_ino_t, nlookup:c_ulong)>,
    getattr: Option<~fn(ino: fuse_ino_t) -> ErrnoResult<AttrReply>>,
    setattr: Option<~fn(ino: fuse_ino_t, _attrs_toset:&[AttrToSet],
                              fh:Option<u64>) -> ErrnoResult<AttrReply>>,
    readlink: Option<~fn(ino: fuse_ino_t) -> ErrnoResult<~str>>,
    mknod: Option<~fn(parent: fuse_ino_t, name: &CString, 
                            mode: mode_t, rdev: dev_t) 
                            -> ErrnoResult<EntryReply>>,
    mkdir: Option<~fn(parent: fuse_ino_t, name: &CString,
                            mode: mode_t) -> ErrnoResult<EntryReply>>,
    unlink: Option<~fn(parent: fuse_ino_t, name: &CString)
                             -> ErrnoResult<()>>,
    rmdir: Option<~fn(parent: fuse_ino_t, name: &CString)
                            -> ErrnoResult<()>>,
    symlink: Option<~fn(link:&CString, parent: fuse_ino_t,
                              name: &CString) -> ErrnoResult<EntryReply>>,
    rename: Option<~fn(parent: fuse_ino_t, name: &CString,
                             newparent: fuse_ino_t, newname: &CString)
                             -> ErrnoResult<()>>,
    link: Option<~fn(ino: fuse_ino_t, newparent: fuse_ino_t,
                           newname: &CString) -> ErrnoResult<EntryReply>>,
    open: Option<~fn(ino: fuse_ino_t, flags: c_int)
                           -> ErrnoResult<OpenReply>>,
    read: Option<~fn(ino: fuse_ino_t, size: size_t, off: off_t,
                           fh: u64) -> ErrnoResult<ReadReply>>,
    // TODO: is writepage a bool, or an actual number that needs to be;
    // preserved?;
    write: Option<~fn(ino: fuse_ino_t, buf:&[u8], off: off_t,
                            fh: u64, writepage: bool)
                            -> ErrnoResult<size_t>>,
    flush: Option<~fn(ino: fuse_ino_t, _lockowner: u64, fh: u64)
                            -> ErrnoResult<()>>,
    release: Option<~fn(ino: fuse_ino_t, flags: c_int, fh: u64)
                              -> ErrnoResult<()>>,
    fsync: Option<~fn(ino: fuse_ino_t, datasync: bool, fh: u64)
                            -> ErrnoResult<()>>,
    opendir: Option<~fn(ino: fuse_ino_t) -> ErrnoResult<OpenReply>>,
    readdir: Option<~fn(ino: fuse_ino_t, size: size_t, off: off_t,
                              fh: u64) -> ErrnoResult<ReaddirReply>>,
    releasedir: Option<~fn(ino: fuse_ino_t, fh: u64)
                                 -> ErrnoResult<()>>,
    fsyncdir: Option<~fn(ino: fuse_ino_t, datasync: bool, fh: u64)
                               -> ErrnoResult<()>>,
    statfs: Option<~fn(ino: fuse_ino_t) -> ErrnoResult<Struct_statvfs>>,
    setxattr: Option<~fn(ino: fuse_ino_t, name: &CString,
                               value: &[u8], flags: c_int)
                               -> ErrnoResult<()>>,
    // TODO: examine this--ReadReply may not be appropraite here;
    getxattr: Option<~fn(ino: fuse_ino_t, name: &CString,
                               size: size_t) -> ErrnoResult<ReadReply>>,
    // Called on getxattr with size of zero (meaning a query of total size);
    getxattr_size: Option<~fn(ino: fuse_ino_t, name: &CString)
                                    -> ErrnoResult<size_t>>,
    // TODO: examine this--ReadReply may not be appropraite here;
    listxattr: Option<~fn(ino: fuse_ino_t, size: size_t)
                                -> ErrnoResult<ReadReply>>,
    // Called on listxattr with size of zero (meaning a query of total size);
    listxattr_size: Option<~fn(ino: fuse_ino_t) -> ErrnoResult<size_t>>,
    removexattr: Option<~fn(ino: fuse_ino_t, name: &CString)
                                  -> ErrnoResult<()>>,
    access: Option<~fn(ino: fuse_ino_t, mask: c_int)
                             -> ErrnoResult<()>>,
    create: Option<~fn(parent: fuse_ino_t, name: &CString,
                             mode: mode_t, flags: c_int)
                             -> ErrnoResult<CreateReply>>,

    // TODO: The following, still need implementing:
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

    /// This path is used to allow a call to unmount() (or the FuseMount option
    /// falling out of scope) to cleanly stop the C API thread.  The C API does
    /// blocking read calls, and the only way to interrupt it is via signals,
    /// which rust does not support yet.  So when we want to make sure the C
    /// API wakes up, we read this path, and intercept any attempts to look it
    /// up to make sure it's not valid.
    ///
    /// With the right signal support, this will hopefully become unnecessary.
    /// Until then, you can at least set this to something else on the off
    /// chance that the default one would interfere with something.
    ridiculous_hack_filename: ~str,
    
    /// Command line arguments to pass through to the FUSE API.  See the
    /// `fuse_ll_help` function in the FUSE source for what can go here.
    args:~[~[u8]]
}
impl Default for FuseMountOptions {
    fn default() -> FuseMountOptions {
        FuseMountOptions{
            // The most unlikely filename I could think of is a bunch of
            // non-printable UTF8.
            ridiculous_hack_filename: ~"\x01\x02\x03\x04\x05\x06\x07\x08",
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
    priv nocopies: NonCopyable,
    priv ridiculous_hack_filename: ~str
}
impl FuseMount {

    /// Mount the FUSE file system using the functions in `ops`, with options
    /// (including the mount point) taken from `options.args`.  This function
    /// will fail if the options are not valid as per FUSE, or if FUSE fails to
    /// mount.
    pub fn new(options:~FuseMountOptions, ops:~FuseLowLevelOps)
           -> ~FuseMount {

        // The C API needs its own OS thread because it will block.  We want to
        // run all of the filesystem commands we get in "parallel" on their own
        // rust tasks, but we don't want to spawn a new OS thread for each of
        // them.  So instead, we have this: the C API gets its own task on its
        // own OS thread, and does nothing but send commands through a chan to
        // the dispatch task.  The dispatch task is running on the same
        // scheduler as the task calling `FuseMount::new`.  Its job is to
        // receive commands and start a task (again on the default scheduler)
        // in which to run each one.

        let FuseMountOptions{args:args, ridiculous_hack_filename:hackname} =
            *options;

        let (dispatch_port, dispatch_chan) = stream::<~FSOperation>();
        let (finish_port, finish_chan) = stream::<TaskResult>();
        // This is how we get the session pointer out of the C API thread,
        // so that we can use it to end the session later.
        let (session_port, session_chan) = 
            oneshot::<~FuseSession>();

        // Spawn the C API task
        let mut c_api_task = task();
        c_api_task.sched_mode(SingleThreaded);
        c_api_task.linked();
        c_api_task.name(format!("FUSE C API - {:?}",
                                args.map(|v| str::from_utf8(*v))));
        c_api_task.opts.notify_chan = Some(finish_chan);
        let userdata = ~FuseUserData{
            args: args, 
            ops:ops,
            dispatch_chan:dispatch_chan,
            ridiculous_hack_filename: hackname.clone()};
        c_api_task.spawn_with(
            (session_chan, userdata),
            |(schan, userdata)| c_api_loop(schan, userdata));
        
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
            nocopies: NonCopyable::new(),
            ridiculous_hack_filename: hackname
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
            unsafe {
                fuse_session_exit(self.session.session);
                // At this point the C thread could be doing a blocking read
                // and won't wake up just because fuse_session_exit was called.
                // So, read the "ridiculous hack" filename just to wake it up!
                let hack_path = self.mount_point().push(
                    self.ridiculous_hack_filename);
                do task::try {
                    ::std::rt::io::file::stat(&hack_path);
                };
            }
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

// The FUSE userdata pointer will point to one of these.  The c extern fns
// use it to get back into the correct corresponding rust tasks.
struct FuseUserData {
    ops: ~FuseLowLevelOps,
    args: ~[~[u8]],
    // Send FS command functions through here to be dispatched to new tasks on
    // the right scheduler
    dispatch_chan:Chan<~FSOperation>,
    ridiculous_hack_filename: ~str
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
pub fn c_api_loop(session_chan: ChanOne<~FuseSession>, 
                  userdata:~FuseUserData) {
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

        let llo = make_fuse_ll_oper(userdata.ops);
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
        session_chan.send(~FuseSession{session:fuse_session,
                                       mount_point:PosixPath(mountpoint_str)});

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
    }
}

#[fixed_stack_segment]
fn userdata_ptr_from_req(req:fuse_req_t) -> *mut c_void {
    unsafe {
        fuse_req_userdata(req)
    }
}

/*
 * Run a function with a borrowed pointer to the FuseLowLevelOps struct pointed
 * to by the given userdata pointer.  The "arg" parameter is for passing extra
 * data into the function a la task::spawn_with (needed to push owned pointers
 * into the closure)
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
        send_fuse_reply(do_op(&(*userdata.ops)), req, reply_success);
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

fn handle_unimpl<F, T>(opt:&Option<F>, name:&str, 
                       imp:&fn(&F) -> ErrnoResult<T>) -> ErrnoResult<T> {
    match *opt {
        Some(ref f) => imp(f),
        None => {
            error!("FUSE called %s, but there was no fn supplied.  This is a bug in rust_fuse.",
                   name);
            Err(libc::ENOSYS)
        }
    }
}

extern fn init_impl(userdata:*mut c_void, _conn:*Struct_fuse_conn_info) {
    do userdata_from_ptr(userdata, ()) |userdata, _| {
        match userdata.ops.init {
            Some(ref f) => (*f)(),
            None=>()
        }
    }
}

extern fn destroy_impl(userdata:*mut c_void) {
    do userdata_from_ptr(userdata, ()) |userdata, _| {
        match userdata.ops.destroy {
            Some(ref f) => (*f)(),
            None=>()
        }
    }
}

macro_rules! run_for_reply_if_impl {
    ($opfunc:ident, $replyfunc:expr, $body:block) => (
        do run_for_reply(req, $replyfunc) |ops| {
            do handle_unimpl(&ops.$opfunc, stringify!($opfunc)) |f|
                $body
        })
}

extern fn lookup_impl(req:fuse_req_t,  parent:fuse_ino_t, name:*c_schar) {
    do send_to_dispatch(req, ()) |userdata, ()| {
        unsafe { 
            let name_cstr = CString::new(name, false);
            if (cstr_as_bytes_no_term(&name_cstr) == 
                userdata.ridiculous_hack_filename.as_bytes()) {
                reply_failure_err(req);
            } else {
                let result = do handle_unimpl(&userdata.ops.lookup, "lookup") |f| {
                    (*f)(parent, &CString::new(name,false))
                };
                send_fuse_reply(result, req, reply_entryparam);
            }
        }
    }
}

extern fn forget_impl(req: fuse_req_t, ino: fuse_ino_t, nlookup:c_ulong) {
    run_for_reply_if_impl!(forget, reply_none, {
            (*f)(ino, nlookup); Ok(())
        })
}

extern fn getattr_impl(req:fuse_req_t, ino: fuse_ino_t,
                       _fi:*Struct_fuse_file_info) {
    run_for_reply_if_impl!(getattr, reply_attr, {
            (*f)(ino)
        })
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
    run_for_reply_if_impl!(setattr, reply_attr, {
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

                (*f)(ino, attrs_to_set, fi.to_option().map(|fi| fi.fh))
            }
        })
}

extern fn readlink_impl(req: fuse_req_t, ino: fuse_ino_t) {
    run_for_reply_if_impl!(readlink, reply_readlink, {
            (*f)(ino)
        })
}

extern fn mknod_impl(req:fuse_req_t, parent: fuse_ino_t, name:*c_schar,
                     mode: mode_t, rdev: dev_t) {
    run_for_reply_if_impl!(mknod, reply_entryparam, {
            unsafe { (*f)(parent, &CString::new(name,false), mode, rdev) }
        })
}

extern fn mkdir_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar,
                     mode:mode_t) {
    run_for_reply_if_impl!(mkdir, reply_entryparam, {
            unsafe { (*f)(parent, &CString::new(name,false), mode) }
        })
}

extern fn unlink_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar) {
    run_for_reply_if_impl!(unlink, reply_zero_err, {
            unsafe { (*f)(parent, &CString::new(name,false)) }
        })
}

extern fn rmdir_impl(req: fuse_req_t, parent: fuse_ino_t, name:*c_schar) {
    run_for_reply_if_impl!(rmdir, reply_zero_err, {
            unsafe { (*f)(parent, &CString::new(name,false)) }
        })
}

extern fn symlink_impl(req: fuse_req_t, link: *c_schar, parent: fuse_ino_t,
                       name: *c_schar) {
    run_for_reply_if_impl!(symlink, reply_entryparam, {
            unsafe {
                (*f)(&CString::new(link,false), parent, 
                     &CString::new(name,false))
            }
        })
}

extern fn rename_impl(req: fuse_req_t, parent: fuse_ino_t, name: *c_schar,
                      newparent: fuse_ino_t, newname: *c_schar) {
    run_for_reply_if_impl!(rename, reply_zero_err, {
            unsafe {
                (*f)(parent, &CString::new(name,false), newparent,
                     &CString::new(newname,false))
            }
        })
}

extern fn link_impl(req: fuse_req_t, ino: fuse_ino_t, newparent: fuse_ino_t,
                    newname: *c_schar) {
    run_for_reply_if_impl!(link, reply_entryparam, {
            unsafe {
                (*f)(ino, newparent, &CString::new(newname,false))
            }
        })
}

extern fn open_impl(req: fuse_req_t, ino: fuse_ino_t,
                    fi: *Struct_fuse_file_info) {
    run_for_reply_if_impl!(open, reply_open, {
            unsafe {
                (*f)(ino, (*fi).flags)
            }
        })
}

extern fn read_impl(req: fuse_req_t, ino: fuse_ino_t, size: size_t, off: off_t,
                    fi: *Struct_fuse_file_info) {
    run_for_reply_if_impl!(read, reply_read, {
            unsafe {
                (*f)(ino, size, off, (*fi).fh)
            }
        })
}

extern fn write_impl(req: fuse_req_t, ino: fuse_ino_t, buf: *u8,
                     size: size_t, off: off_t, fi: *Struct_fuse_file_info) {
    run_for_reply_if_impl!(write, reply_write, {
            unsafe {
                do vec::raw::buf_as_slice(buf, size as uint) |vec| {
                    (*f)(ino, vec, off, (*fi).fh, ((*fi).writepage != 0))
                }
            }
        })
}

extern fn flush_impl(req: fuse_req_t, ino: fuse_ino_t,
                     fi: *Struct_fuse_file_info) {
    run_for_reply_if_impl!(flush, reply_zero_err, {
            unsafe {
                (*f)(ino, (*fi).lock_owner, (*fi).fh)
            }
        })
}

extern fn release_impl(req: fuse_req_t, ino: fuse_ino_t,
                       fi: *Struct_fuse_file_info) {
    run_for_reply_if_impl!(release, reply_zero_err, {
            unsafe {
                (*f)(ino, (*fi).flags, (*fi).fh)
            }
        })
}

extern fn fsync_impl(req: fuse_req_t, ino: fuse_ino_t, datasync: c_int,
                     fi: *Struct_fuse_file_info) {
    run_for_reply_if_impl!(fsync, reply_zero_err, {
            unsafe {
                (*f)(ino, (datasync != 0), (*fi).fh)
            }
        })
}

extern fn opendir_impl(req: fuse_req_t, ino: fuse_ino_t,
                       _fi: *Struct_fuse_file_info) {
    run_for_reply_if_impl!(opendir, reply_open, {
            (*f)(ino)
        })
}

extern fn readdir_impl(req: fuse_req_t, ino: fuse_ino_t, size: size_t,
                       off: off_t, fi: *Struct_fuse_file_info) {
    run_for_reply_if_impl!(readdir, reply_readdir, {
            unsafe {
                (*f)(ino, size, off, (*fi).fh)
            }.and_then(|rr| Ok((size, rr)))
        })
}

extern fn releasedir_impl(req: fuse_req_t, ino: fuse_ino_t,
                          fi: *Struct_fuse_file_info) {
    run_for_reply_if_impl!(releasedir, reply_zero_err, {
            unsafe {
                (*f)(ino, (*fi).fh)
            }
        })
}

extern fn fsyncdir_impl(req: fuse_req_t, ino: fuse_ino_t, datasync: c_int,
                        fi: *Struct_fuse_file_info) {
    run_for_reply_if_impl!(fsyncdir, reply_zero_err, {
            unsafe {
                (*f)(ino, (datasync != 0), (*fi).fh)
            }
        })
}

extern fn statfs_impl(req: fuse_req_t, ino: fuse_ino_t) {
    run_for_reply_if_impl!(statfs, reply_statfs, {
            (*f)(ino)
        })
}

extern fn setxattr_impl(req: fuse_req_t, ino: fuse_ino_t, name: *c_schar,
                        value: *u8, size: size_t, flags: c_int) {
    run_for_reply_if_impl!(setxattr, reply_zero_err, {
            unsafe {
                do vec::raw::buf_as_slice(value, size as uint) |vec| {
                    (*f)(ino, &CString::new(name,false), vec, flags)
                }
            }
        })
}

extern fn getxattr_impl(req: fuse_req_t, ino: fuse_ino_t, name: *c_schar,
                        size: size_t) {
    if size == 0 {
        run_for_reply_if_impl!(getxattr_size, reply_xattr, {
                unsafe { (*f)(ino, &CString::new(name,false)) }
            })
    } else {
        run_for_reply_if_impl!(getxattr, reply_read, {
                unsafe { (*f)(ino, &CString::new(name,false), size) }
            })
    }
}

extern fn listxattr_impl(req: fuse_req_t, ino: fuse_ino_t, size: size_t) {
    if size == 0 {
        run_for_reply_if_impl!(listxattr_size, reply_xattr, {
                (*f)(ino)
            })
    } else {
        run_for_reply_if_impl!(listxattr, reply_read, {
                (*f)(ino, size)
            })
    }
}

extern fn removexattr_impl(req: fuse_req_t, ino: fuse_ino_t, name: *c_schar) {
    run_for_reply_if_impl!(removexattr, reply_zero_err, {
            unsafe { (*f)(ino, &CString::new(name,false)) }
        })
}

extern fn access_impl(req: fuse_req_t,
                      ino: fuse_ino_t, mask: c_int) {
    run_for_reply_if_impl!(access, reply_zero_err, {
            (*f)(ino, mask)
        })
}

extern fn create_impl(req: fuse_req_t, parent: fuse_ino_t, name: *c_schar,
                      mode: mode_t, fi: *Struct_fuse_file_info) {
    run_for_reply_if_impl!(create, reply_create, {
            unsafe {
                (*f)(parent, &CString::new(name,false), mode, (*fi).flags)
            }
        })
}
