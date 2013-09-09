extern mod rust_fuse;

use std::libc::{
    S_IFDIR,
    S_IFREG,
    ENOENT,
    EACCES,
    O_RDONLY,
    stat,
    off_t,
    size_t,
    mode_t,
    c_int
};

use std::cmp::min;
use rust_fuse::lowlevel::*;
use std::os;
use std::path::stat::arch::default_stat;
use std::num::zero;

static HELLO_STR:&'static str = "Hello rusty world!\n";
static HELLO_FILE_NAME:&'static str = "hello_from_rust";
static INO_ROOT_DIR:fuse_ino_t = 1;
static INO_HELLO_FILE:fuse_ino_t = 2;
fn root_dir_stat() -> stat {
    stat{
        // 493: octal 755.  Rust lacks octal literals
        st_mode: (S_IFDIR | 493) as mode_t,
        st_nlink: 2,
        .. default_stat()
    }
}
fn hello_file_stat() -> stat {
    stat{
        // 292: octal 0444
        st_mode: (S_IFREG | 292) as mode_t,
        st_nlink: 1,
        st_size: HELLO_STR.len() as off_t,
        .. default_stat()
    }
}

fn hello_stat(ino: fuse_ino_t) -> Option<stat> {
    match(ino) {
        INO_ROOT_DIR => Some(root_dir_stat()),
        INO_HELLO_FILE => Some(hello_file_stat()),
        _ => None
    }
}

struct HelloFs;



fn main() {
    fuse_main(os::args(), ~FuseLowLevelOps{

            getattr: Some(|ino: fuse_ino_t|  {
                    match hello_stat(ino) {
                        Some(st) => Ok(AttrReply{ attr: st, attr_timeout: 1.0 }),
                        None => Err(ENOENT)
                    }
                }),

            lookup: Some(|parent: fuse_ino_t, name:&str| 
                         {
                    if parent != INO_ROOT_DIR || name != HELLO_FILE_NAME {
                        Err(ENOENT) 
                    } else { Ok(Struct_fuse_entry_param {
                                ino: INO_HELLO_FILE,
                                generation: 0,
                                attr: hello_file_stat(),
                                attr_timeout: 1.0,
                                entry_timeout: 1.0
                            })
                    }
                }),

            readdir: Some(|ino:fuse_ino_t, _size: size_t, off: off_t, _fh: u64| 
                          {
                    if ino != INO_ROOT_DIR {
                        Err(ENOENT)
                    } else {
                        let entries = [
                                       DirEntry{ino: INO_ROOT_DIR, name: ~".", 
                                    mode: root_dir_stat().st_mode, next_offset: 1},
                                       DirEntry{ino: INO_ROOT_DIR, name: ~"..", 
                                    mode: root_dir_stat().st_mode, next_offset: 2}, 
                                       DirEntry{ 
                                    ino: INO_HELLO_FILE, 
                                    name: HELLO_FILE_NAME.into_owned(), 
                                    mode: hello_file_stat().st_mode, 
                                    next_offset: 3},
                                       ];
                        let slice = entries.slice_from(min(entries.len(), off as uint));
                        Ok(DirEntries(slice.into_owned()))
                    }
                }),

            open: Some(|ino: fuse_ino_t, flags: c_int|  {
                    if ino != INO_HELLO_FILE {
                        Err(ENOENT)
                    } else if flags & 3 != O_RDONLY {
                        Err(EACCES)
                    } else {
                        Ok(OpenReply{direct_io: false, keep_cache: false, fh: 0})
                    }
                }),

            read: Some(|ino: fuse_ino_t, size: size_t, off: off_t, _fh: u64|
                       {
                    if ino != INO_HELLO_FILE {
                        Err(ENOENT)
                    } else {
                        let slice_to_read = HELLO_STR.slice(off as uint, 
                                                            min(HELLO_STR.len(),size as uint));
                        Ok(DataBuffer(slice_to_read.as_bytes().into_owned()))
                    }
                }),
            ..zero()
        });
}
