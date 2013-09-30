use super::util::*;
use std::rt::io::extensions::ReaderUtil;
use std::rt::io::{file,Read,Open};
use std::os;
use std::hashmap::{HashSet};
use std::str;
use rust_fuse::lowlevel::*;

#[test]
fn hello_fs_works() {
    let tdg = TempDirAutoCleanup::new_opt(&os::tmpdir(),
                                          "hello_fs_works").unwrap();
    let path_str= tdg.path.to_str();
    let mount_args = ~["hello_fs".as_bytes().to_owned(),
                       path_str.as_bytes().to_owned()];
    // The first argument is for the executable
    let _mounter = FuseMount::new(~FuseMountOptions{args:mount_args,
                                                    ..Default::default()},
                                  super::testfs::hello::hello_fs());
    
    let expected_dirs = [~"hello_from_rust"];
    let mut edirs_map = expected_dirs.iter().map(|x| x.clone());
    let expected:HashSet<~str> = FromIterator::from_iterator(&mut edirs_map);
    info!("About to run the actual test with %s",tdg.path.to_str());
    let actual_dirs = file::readdir(&tdg.path).unwrap_or(~[]);
    let mut actmap = actual_dirs.iter().map(
        |x| x.filename().get_ref().into_owned());
    let actual:HashSet<~str> = FromIterator::from_iterator(&mut actmap);
    assert_eq!(expected,actual);

    let file_contents = file::open(&tdg.path.push("hello_from_rust"), 
                                   Open, Read).read_to_end();
    assert_eq!("Hello rusty world!\n", str::from_utf8_slice(file_contents));
}
