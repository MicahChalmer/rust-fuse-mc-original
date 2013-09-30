use std::path::Path;
use std::util::NonCopyable;
use std::os;
use extra::tempfile::mkdtemp;

pub struct TempDirAutoCleanup {
    path:Path,
    priv token: NonCopyable
}
impl TempDirAutoCleanup {
    pub fn new_opt(tmpdir:&Path, suffix:&str) -> Option<~TempDirAutoCleanup> {
        do mkdtemp(tmpdir, suffix).map() |path| {
            info!("Created temp dir %s", path.to_str());
            ~TempDirAutoCleanup{path:path.clone(), token: NonCopyable::new()}
        }
    }
}
impl Drop for TempDirAutoCleanup {
    fn drop(&mut self) {
        info!("About to remove dir %s", self.path.to_str());
        os::remove_dir_recursive(&self.path);
    }
}

