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

mod fuse;

/// Information to be returned from open
#[deriving(Zero)]
pub struct OpenReply {
    direct_io:bool,
    keep_cache:bool,
    fh: u64
}

pub struct ReadReply;

macro_rules! declare_fuse_llops(
    ({$($name:ident : ($($pname:ident : $ptype:ty),*) -> $rt:ty),* }) => (

pub struct FuseLowLevelOps {
    init: Option<~fn()>,
    destroy: Option<~fn()>,
    forget: Option<~fn(ino:fuse::fuse_ino_t, nlookup:c_ulong)>,
    $($name : Option<~fn($($pname: $ptype),*) -> Result<$rt, c_int>>),*
}
)
)

declare_fuse_llops!({
        lookup: (parent:fuse::fuse_ino_t,name:&str) -> fuse::Struct_fuse_entry_param,
        getattr: (ino: fuse::fuse_ino_t, flags:c_int) -> OpenReply,
        read: (ino: fuse::fuse_ino_t, size: size_t, off: off_t, fh: u64)-> ReadReply
    })

pub fn fllo() -> FuseLowLevelOps {
    FuseLowLevelOps { init: None, destroy: None, forget: None, lookup: None,
        getattr: None, read: None }
}
