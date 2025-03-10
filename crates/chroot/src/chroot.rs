use crate::command::Command;
use std::{
    ffi::OsStr,
    io::Result,
    path::{Path, PathBuf},
    process::Stdio,
};
use sys_mount::*;

/// Defines the location where a `chroot` will be performed, as well as storing
/// handles to all of the binding mounts that the chroot requires.
pub struct Chroot<'a> {
    pub path: PathBuf,
    dev_mount: Mount,
    pts_mount: Mount,
    proc_mount: Mount,
    run_mount: Mount,
    sys_mount: Mount,
    clear_envs: bool,
    envs: Vec<(&'a str, &'a str)>,
}

impl<'a> Chroot<'a> {
    /// Performs binding mounts of all required paths to ensure that a chroot
    /// is successful.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().canonicalize()?;
        let builder = Mount::builder().flags(MountFlags::BIND).fstype("none");

        let dev_mount = builder.clone().mount("/dev", &path.join("dev"))?;
        let pts_mount = builder.clone().mount("/dev/pts", &path.join("dev").join("pts"))?;
        let proc_mount = builder.clone().mount("/proc", &path.join("proc"))?;
        let run_mount = builder.clone().mount("/run", &path.join("run"))?;
        let sys_mount = builder.clone().mount("/sys", &path.join("sys"))?;

        Ok(Chroot {
            path,
            dev_mount,
            pts_mount,
            proc_mount,
            run_mount,
            sys_mount,
            clear_envs: false,
            envs: Vec::new(),
        })
    }

    /// Set an environment variable to define for this chroot.
    pub fn env(&mut self, key: &'a str, value: &'a str) {
        self.envs.push((key, value));
    }

    /// Clear all environment variables for this chroot.
    pub fn clear_envs(&mut self, clear: bool) {
        self.clear_envs = clear;
    }

    /// Executes an external command with `chroot`.
    pub fn command<S: AsRef<OsStr>, T: AsRef<OsStr>, I: IntoIterator<Item = T>>(
        &self,
        cmd: S,
        args: I,
    ) -> Command {
        let mut command = cascade! {
            Command::new("chroot");
            ..arg(&self.path);
            ..arg(cmd.as_ref());
            ..args(args);
            ..stderr(Stdio::piped());
            ..stdout(Stdio::piped());
        };

        if self.clear_envs {
            command.env_clear();
        }

        for &(key, value) in &self.envs {
            command.env(key, value);
        }

        command
    }

    /// Return true if the filesystem was unmounted, false if it was already
    /// unmounted
    pub fn unmount(&mut self, lazy: bool) -> Result<()> {
        let flags = if lazy { UnmountFlags::DETACH } else { UnmountFlags::empty() };
        self.sys_mount.unmount(flags)?;
        self.run_mount.unmount(flags)?;
        self.proc_mount.unmount(flags)?;
        self.pts_mount.unmount(flags)?;
        self.dev_mount.unmount(flags)?;
        Ok(())
    }
}

impl<'a> Drop for Chroot<'a> {
    fn drop(&mut self) {
        // Ensure unmounting
        let _ = self.sys_mount.unmount(UnmountFlags::DETACH);
        let _ = self.run_mount.unmount(UnmountFlags::DETACH);
        let _ = self.proc_mount.unmount(UnmountFlags::DETACH);
        let _ = self.pts_mount.unmount(UnmountFlags::DETACH);
        let _ = self.dev_mount.unmount(UnmountFlags::DETACH);
    }
}
