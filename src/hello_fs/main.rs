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
    fn getattr(&self, path:&str) -> ErrorOrResult<errno, stat> {
        match path {
            "/" => Result({
                    let mut st = default_stat();
                    // 493: octal 755.  Rust lacks octal literals
                    st.st_mode = (S_IFDIR | 493) as u32;
                    st.st_nlink = 2;
                    st
                }),
            HELLO_FILE_FULLPATH => Result({
                    let mut st = default_stat();
                    // 292: octal 0444
                    st.st_mode = (S_IFREG | 292) as u32;
                    st.st_nlink = 1;
                    st.st_size = HELLO_STR.len() as i64;
                    st
                }),
            _ => Error(ENOENT)
        }
    }

    fn readdir(&self, path:&str, filler: fuse_fill_dir_func,
               _offset: off_t, _info: &fuse_file_info) -> ErrorOrResult<errno, ()> {
        match path {
            "/" => Result({
                    filler(".", None, 0);
                    filler("..", None, 0);
                    filler(HELLO_FILE_FULLPATH.slice_from(1), None, 0);
                }),
            _ => Error(ENOENT)
        }
    }

    fn open(&self, path:&str, info: &fuse_file_info) -> ErrorOrResult<errno, filehandle> {
        match path {
            HELLO_FILE_FULLPATH => 
                if info.flags & 3 != O_RDONLY { Error(EACCES) } else { Result(0) },
            _ => Error(ENOENT)
        }
    }
        
    fn read(&self, path:&str, buf:&mut [u8], size: size_t, offset: off_t,
            _info: &fuse_file_info) -> ErrorOrResult<errno, c_int> {
        if path != HELLO_FILE_FULLPATH {
            return Error(ENOENT)
        };

        let slice_to_read = HELLO_STR.slice(offset as uint, 
                                            min(HELLO_STR.len(),size as uint));
        Result(buf.copy_from(slice_to_read.as_bytes()) as c_int)
    }
}

fn main() {
    std::os::set_exit_status(fuse_main(os::args(), ~HelloFs))
}
