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
    c_int
};

use std::cmp::min;
use rust_fuse::*;
use std::os;
use std::path::stat::arch::default_stat;
use std::vec::MutableCloneableVector;

static HELLO_STR:&'static str = "Hello rusty world!\n";
static HELLO_FILE_FULLPATH:&'static str = "/hello_from_rust";

struct HelloFs;

impl FuseOperations for HelloFs {
    fn getattr(&self, path:&str) -> Result<stat, errno> {
        match path {
            "/" => Ok(stat{
                    // 493: octal 755.  Rust lacks octal literals
                    st_mode: (S_IFDIR | 493) as u32,
                    st_nlink: 2,
                    .. default_stat()
                }),
            x if x == HELLO_FILE_FULLPATH => Ok(stat{
                    // 292: octal 0444
                    st_mode: (S_IFREG | 292) as u32,
                    st_nlink: 1,
                    st_size: HELLO_STR.len() as i64,
                    .. default_stat()
                }),
            _ => Err(ENOENT)
        }
    }

    fn readdir(&self, path:&str, filler: fuse_fill_dir_func,
               _offset: off_t, _info: &fuse_file_info) -> Result<(), errno> {
        match path {
            "/" => Ok({
                    filler(".", None, 0);
                    filler("..", None, 0);
                    filler(HELLO_FILE_FULLPATH.slice_from(1), None, 0);
                }),
            _ => Err(ENOENT)
        }
    }

    fn open(&self, path:&str, info: &fuse_file_info) -> Result<filehandle, errno> {
        match path {
            x if x == HELLO_FILE_FULLPATH => 
                if info.flags & 3 != O_RDONLY { Err(EACCES) } else { Ok(0) },
            _ => Err(ENOENT)
        }
    }
        
    fn read(&self, path:&str, buf:&mut [u8], size: size_t, offset: off_t,
            _info: &fuse_file_info) -> Result<c_int, errno> {
        if path != HELLO_FILE_FULLPATH {
            return Err(ENOENT)
        };

        let slice_to_read = HELLO_STR.slice(offset as uint, 
                                            min(HELLO_STR.len(),size as uint));
        Ok(buf.copy_from(slice_to_read.as_bytes()) as c_int)
    }
}

fn main() {
    std::os::set_exit_status(fuse_main(os::args(), ~HelloFs as ~FuseOperations))
}
