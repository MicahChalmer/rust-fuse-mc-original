// This is a copy-paste of the code in libstd/path.rs, which for some reason is
// private, even though just about any user of a C library that uses the stat
// structure would need it...

#[cfg(target_os = "linux")]
#[cfg(target_os = "android")]
pub mod stat {
    #[cfg(target_arch = "x86")]
    pub mod arch {
        use std::libc;

        pub fn default_stat() -> libc::stat {
            libc::stat {
                st_dev: 0,
                __pad1: 0,
                st_ino: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                __pad2: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_atime: 0,
                st_atime_nsec: 0,
                st_mtime: 0,
                st_mtime_nsec: 0,
                st_ctime: 0,
                st_ctime_nsec: 0,
                __unused4: 0,
                __unused5: 0,
            }
        }
    }

    #[cfg(target_arch = "arm")]
    pub mod arch {
        use std::libc;

        pub fn default_stat() -> libc::stat {
            libc::stat {
                st_dev: 0,
                __pad0: [0, ..4],
                __st_ino: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                __pad3: [0, ..4],
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_atime: 0,
                st_atime_nsec: 0,
                st_mtime: 0,
                st_mtime_nsec: 0,
                st_ctime: 0,
                st_ctime_nsec: 0,
                st_ino: 0
            }
        }
    }

    #[cfg(target_arch = "mips")]
    pub mod arch {
        use std::libc;

        pub fn default_stat() -> libc::stat {
            libc::stat {
                st_dev: 0,
                st_pad1: [0, ..3],
                st_ino: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_pad2: [0, ..2],
                st_size: 0,
                st_pad3: 0,
                st_atime: 0,
                st_atime_nsec: 0,
                st_mtime: 0,
                st_mtime_nsec: 0,
                st_ctime: 0,
                st_ctime_nsec: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_pad5: [0, ..14],
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub mod arch {
        use std::libc;

        pub fn default_stat() -> libc::stat {
            libc::stat {
                st_dev: 0,
                st_ino: 0,
                st_nlink: 0,
                st_mode: 0,
                st_uid: 0,
                st_gid: 0,
                __pad0: 0,
                st_rdev: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_atime: 0,
                st_atime_nsec: 0,
                st_mtime: 0,
                st_mtime_nsec: 0,
                st_ctime: 0,
                st_ctime_nsec: 0,
                __unused: [0, 0, 0],
            }
        }
    }
}

#[cfg(target_os = "freebsd")]
pub mod stat {
    #[cfg(target_arch = "x86_64")]
    pub mod arch {
        use std::libc;

        pub fn default_stat() -> libc::stat {
            libc::stat {
                st_dev: 0,
                st_ino: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_atime: 0,
                st_atime_nsec: 0,
                st_mtime: 0,
                st_mtime_nsec: 0,
                st_ctime: 0,
                st_ctime_nsec: 0,
                st_size: 0,
                st_blocks: 0,
                st_blksize: 0,
                st_flags: 0,
                st_gen: 0,
                st_lspare: 0,
                st_birthtime: 0,
                st_birthtime_nsec: 0,
                __unused: [0, 0],
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub mod stat {
    pub mod arch {
        use std::libc;

        pub fn default_stat() -> libc::stat {
            libc::stat {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_ino: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_atime: 0,
                st_atime_nsec: 0,
                st_mtime: 0,
                st_mtime_nsec: 0,
                st_ctime: 0,
                st_ctime_nsec: 0,
                st_birthtime: 0,
                st_birthtime_nsec: 0,
                st_size: 0,
                st_blocks: 0,
                st_blksize: 0,
                st_flags: 0,
                st_gen: 0,
                st_lspare: 0,
                st_qspare: [0, 0],
            }
        }
    }
}

#[cfg(target_os = "win32")]
pub mod stat {
    pub mod arch {
        use std::libc;
        pub fn default_stat() -> libc::stat {
            libc::stat {
                st_dev: 0,
                st_ino: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_size: 0,
                st_atime: 0,
                st_mtime: 0,
                st_ctime: 0,
            }
        }
    }
}
