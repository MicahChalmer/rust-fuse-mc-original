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

use std::cmp::{min, max};
use rust_fuse::lowlevel::*;
use rust_fuse::stat::stat::arch::default_stat;
use std::c_str::CString;
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

fn cstr_as_bytes_no_term<'a>(cs:&'a CString) -> &'a[u8] {
    let ab = cs.as_bytes();
    ab.slice_to(max(ab.len()-1,0))
}

pub struct HelloFs;
impl FuseLowLevelOps for HelloFs {
    fn getattr(&self, ino: fuse_ino_t) -> ErrnoResult<AttrReply> {
        let hello_stat = match(ino) {
            INO_ROOT_DIR => Some(root_dir_stat()),
            INO_HELLO_FILE => Some(hello_file_stat()),
            _ => None
        };
        match hello_stat {
            Some(st) => Ok(AttrReply{ attr: st,
                                      attr_timeout: 1.0 }),
            None => Err(ENOENT)
        }
    }
    fn getattr_is_implemented(&self) -> bool { true }

    fn lookup(&self, parent: fuse_ino_t, name: &CString)
              -> ErrnoResult<EntryReply> {
        if parent != INO_ROOT_DIR || 
            cstr_as_bytes_no_term(name)
            != HELLO_FILE_NAME.as_bytes() {
            Err(ENOENT) 
        } else { Ok(Struct_fuse_entry_param {
                    ino: INO_HELLO_FILE,
                    generation: 0,
                    attr: hello_file_stat(),
                    attr_timeout: 1.0,
                    entry_timeout: 1.0
                })
        }
    }
    fn lookup_is_implemented(&self) -> bool { true }
    fn readdir(&self, ino: fuse_ino_t, _size: size_t, off: off_t,
               _fh: u64) -> ErrnoResult<ReaddirReply> {
        if ino != INO_ROOT_DIR {
            Err(ENOENT)
        } else {
            let mut entries = ~[
                DirEntry{ino: INO_ROOT_DIR,
                         name: ".".to_c_str(), 
                         mode: root_dir_stat().st_mode,
                         next_offset: 1},
                DirEntry{ino: INO_ROOT_DIR,
                         name: "..".to_c_str(),
                         mode: root_dir_stat().st_mode,
                         next_offset: 2}, 
                DirEntry{ 
                    ino: INO_HELLO_FILE, 
                    name: HELLO_FILE_NAME.to_c_str(), 
                    mode: hello_file_stat().st_mode, 
                    next_offset: 3},
                ];
            let num_entries = entries.len();
            let start = min(num_entries, off as uint);
            entries.reverse();
            entries.truncate(num_entries - start);
            entries.reverse();
            Ok(DirEntries(entries))
        }
    }
    fn readdir_is_implemented(&self) -> bool { true }

    fn open(&self, ino: fuse_ino_t, flags: c_int) -> ErrnoResult<OpenReply> {
        if ino != INO_HELLO_FILE {
            Err(ENOENT)
        } else if flags & 3 != O_RDONLY {
            Err(EACCES)
        } else {
            Ok(OpenReply{direct_io: false, keep_cache: false,
                         fh: 0})
        }
    }
    fn open_is_implemented(&self) -> bool { true }

    fn read(&self, ino: fuse_ino_t, size: size_t, off: off_t, _fh: u64) 
            -> ErrnoResult<ReadReply> {
        if ino != INO_HELLO_FILE {
            Err(ENOENT)
        } else {
            let slice_to_read = HELLO_STR.as_bytes().slice(
                off as uint, min(HELLO_STR.len(),size as uint));
            Ok(DataBuffer(slice_to_read.into_owned()))
        }
    }
    fn read_is_implemented(&self) -> bool { true }
}

