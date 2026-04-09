use anyhow::{Context, anyhow};
use std::{
    env,
    ffi::OsString,
    fs, io,
    os::unix::{ffi::OsStrExt, process::CommandExt},
    path::{Path, PathBuf},
    process::Command,
};
use tracing::{info, warn};

const DEFAULT_UID: u32 = 65532;
const DEFAULT_GID: u32 = 65532;

#[derive(Debug, Clone, Copy)]
struct UserConfig {
    uid: u32,
    gid: u32,
}

fn main() -> anyhow::Result<()> {
    lyra_tracing::init();

    let command = parse_command();
    let target = parse_target_user()?;
    let config_dir: PathBuf = env::var_os("LYRA_DATA_DIR")
        .expect("LYRA_DATA_DIR unset in docker")
        .into();

    let initial_uid = current_uid();
    let initial_gid = current_gid();

    info!(
        initial_uid,
        initial_gid,
        target_uid = target.uid,
        target_gid = target.gid,
        config_dir = %config_dir.display(),
        "starting lyra docker init"
    );

    if let Err(error) = ensure_config_dir(&config_dir) {
        warn!(
            error = ?error,
            path = %config_dir.display(),
            "failed to create config directory"
        );
    }

    if initial_uid == 0 {
        if let Err(error) = chown_path(&config_dir, target.uid, target.gid) {
            warn!(
                error = ?error,
                path = %config_dir.display(),
                target_uid = target.uid,
                target_gid = target.gid,
                "failed to update config ownership"
            );
        }

        drop_privileges(target).context("failed to drop privileges")?;
    }

    if let Err(error) = warn_if_not_writable(&config_dir) {
        warn!(
            error = ?error,
            path = %config_dir.display(),
            "config directory may not be writable after init; Lyra may fail on startup"
        );
    }

    info!(
        uid = current_uid(),
        gid = current_gid(),
        command = ?command,
        "starting Lyra"
    );

    let error = Command::new(&command[0]).args(&command[1..]).exec();
    Err(anyhow!("failed to exec {:?}: {}", command, error))
}

fn parse_target_user() -> anyhow::Result<UserConfig> {
    let uid = parse_env_u32("PUID")?.unwrap_or(DEFAULT_UID);
    let gid = parse_env_u32("PGID")?.unwrap_or(DEFAULT_GID);
    Ok(UserConfig { uid, gid })
}

fn parse_env_u32(name: &str) -> anyhow::Result<Option<u32>> {
    match env::var(name) {
        Ok(value) => {
            let value = value.trim();
            if value.is_empty() {
                return Ok(None);
            }

            let parsed = value
                .parse::<u32>()
                .with_context(|| format!("invalid {} value {:?}", name, value))?;
            Ok(Some(parsed))
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(anyhow!("failed to read {}: {}", name, error)),
    }
}

fn parse_command() -> Vec<OsString> {
    let mut args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        args.push(OsString::from("/app/lyra"));
    }
    args
}

fn ensure_config_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

fn chown_path(path: &Path, uid: u32, gid: u32) -> io::Result<()> {
    let path = path_to_cstring(path)?;

    let result = unsafe { libc::lchown(path.as_ptr(), uid, gid) };
    if result == 0 {
        return Ok(());
    }

    Err(io::Error::last_os_error())
}

// Reset supplementary groups before switching IDs so the final Lyra process runs with only the
// requested uid/gid instead of inheriting root's broader access.
fn drop_privileges(target: UserConfig) -> anyhow::Result<()> {
    if unsafe { libc::setgroups(0, std::ptr::null()) } != 0 {
        return Err(anyhow!(io::Error::last_os_error())).context("setgroups failed");
    }

    if unsafe { libc::setgid(target.gid) } != 0 {
        return Err(anyhow!(io::Error::last_os_error())).context("setgid failed");
    }

    if unsafe { libc::setuid(target.uid) } != 0 {
        return Err(anyhow!(io::Error::last_os_error())).context("setuid failed");
    }

    Ok(())
}

fn warn_if_not_writable(path: &Path) -> io::Result<()> {
    if !fs::metadata(path)?.is_dir() {
        return Err(io::Error::other("config path is not a directory"));
    }

    let path = path_to_cstring(path)?;
    let result = unsafe { libc::access(path.as_ptr(), libc::W_OK | libc::X_OK) };
    if result == 0 {
        return Ok(());
    }

    Err(io::Error::last_os_error())
}

fn path_to_cstring(path: &Path) -> io::Result<std::ffi::CString> {
    std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "path contains interior NUL bytes",
        )
    })
}

fn current_uid() -> u32 {
    unsafe { libc::geteuid() }
}

fn current_gid() -> u32 {
    unsafe { libc::getegid() }
}
