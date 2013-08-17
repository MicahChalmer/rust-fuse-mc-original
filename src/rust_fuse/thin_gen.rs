
pub fn make_fuse_ll_oper<Ops:FuseLowLevelOps>(ops:&Ops)
    -> fuse::Struct_fuse_lowlevel_ops {
    return fuse::Struct_fuse_lowlevel_ops {
        init: if ops.init_is_implemented() { init_impl } else { ptr::null() },
        destroy: if ops.destroy_is_implemented() { destroy_impl } else { ptr::null() },
        lookup: if ops.lookup_is_implemented() { lookup_impl } else { ptr::null() },
        forget: if ops.forget_is_implemented() { forget_impl } else { ptr::null() },
        getattr: if ops.getattr_is_implemented() { getattr_impl } else { ptr::null() },
        setattr: if ops.setattr_is_implemented() { setattr_impl } else { ptr::null() },
        readlink: if ops.readlink_is_implemented() { readlink_impl } else { ptr::null() },
        mknod: if ops.mknod_is_implemented() { mknod_impl } else { ptr::null() },
        mkdir: if ops.mkdir_is_implemented() { mkdir_impl } else { ptr::null() },
        unlink: if ops.unlink_is_implemented() { unlink_impl } else { ptr::null() },
        rmdir: if ops.rmdir_is_implemented() { rmdir_impl } else { ptr::null() },
        symlink: if ops.symlink_is_implemented() { symlink_impl } else { ptr::null() },
        rename: if ops.rename_is_implemented() { rename_impl } else { ptr::null() },
        link: if ops.link_is_implemented() { link_impl } else { ptr::null() },
        open: if ops.open_is_implemented() { open_impl } else { ptr::null() },
        read: if ops.read_is_implemented() { read_impl } else { ptr::null() },
        write: if ops.write_is_implemented() { write_impl } else { ptr::null() },
        flush: if ops.flush_is_implemented() { flush_impl } else { ptr::null() },
        release: if ops.release_is_implemented() { release_impl } else { ptr::null() },
        fsync: if ops.fsync_is_implemented() { fsync_impl } else { ptr::null() },
        opendir: if ops.opendir_is_implemented() { opendir_impl } else { ptr::null() },
        readdir: if ops.readdir_is_implemented() { readdir_impl } else { ptr::null() },
        releasedir: if ops.releasedir_is_implemented() { releasedir_impl } else { ptr::null() },
        fsyncdir: if ops.fsyncdir_is_implemented() { fsyncdir_impl } else { ptr::null() },
        statfs: if ops.statfs_is_implemented() { statfs_impl } else { ptr::null() },
        setxattr: if ops.setxattr_is_implemented() { setxattr_impl } else { ptr::null() },
        getxattr: if ops.getxattr_is_implemented() { getxattr_impl } else { ptr::null() },
        getxattr_size: if ops.getxattr_size_is_implemented() { getxattr_size_impl } else { ptr::null() },
        listxattr: if ops.listxattr_is_implemented() { listxattr_impl } else { ptr::null() },
        listxattr_size: if ops.listxattr_size_is_implemented() { listxattr_size_impl } else { ptr::null() },
        removexattr: if ops.removexattr_is_implemented() { removexattr_impl } else { ptr::null() },
        access: if ops.access_is_implemented() { access_impl } else { ptr::null() },
        create: if ops.create_is_implemented() { create_impl } else { ptr::null() },

    }
}

extern fn init_impl(userdata:*c_void, conn:*fuse::Struct_fuse_conn_info) {
    userdata_to_ops(userdata).init();
}

extern fn destroy_impl(userdata:*c_void) {
    userdata_to_ops(userdata).destroy();
}

extern fn lookup_impl() { fail!() }

extern fn forget_impl() { fail!() }

extern fn getattr_impl() { fail!() }

extern fn setattr_impl() { fail!() }

extern fn readlink_impl() { fail!() }

extern fn mknod_impl() { fail!() }

extern fn mkdir_impl() { fail!() }

extern fn unlink_impl() { fail!() }

extern fn rmdir_impl() { fail!() }

extern fn symlink_impl() { fail!() }

extern fn rename_impl() { fail!() }

extern fn link_impl() { fail!() }

extern fn open_impl() { fail!() }

extern fn read_impl() { fail!() }

extern fn write_impl() { fail!() }

extern fn flush_impl() { fail!() }

extern fn release_impl() { fail!() }

extern fn fsync_impl() { fail!() }

extern fn opendir_impl() { fail!() }

extern fn readdir_impl() { fail!() }

extern fn releasedir_impl() { fail!() }

extern fn fsyncdir_impl() { fail!() }

extern fn statfs_impl() { fail!() }

extern fn setxattr_impl() { fail!() }

extern fn getxattr_impl() { fail!() }

extern fn getxattr_size_impl() { fail!() }

extern fn listxattr_impl() { fail!() }

extern fn listxattr_size_impl() { fail!() }

extern fn removexattr_impl() { fail!() }

extern fn access_impl() { fail!() }

extern fn create_impl() { fail!() }
