use once_cell::sync::Lazy;
use rand::Rng;
use serde::Deserialize;
use tap::Tap;

use anyhow::Context;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    fs::{create_dir_all, File},
    io::Read,
    io::Write,
    ops::Deref,
    path::PathBuf,
};

/// The daemon RPC key
pub static GEPH_RPC_KEY: Lazy<String> = Lazy::new(|| {
    get_rpc_key().unwrap_or(format!("geph-rpc-key-{}", rand::thread_rng().gen::<u128>()))
});

/// Configuration for starting the daemon
#[derive(Deserialize, Debug)]
pub struct DaemonConfig {
    pub username: String,
    pub password: String,
    pub exit_hostname: String,
    pub force_bridges: bool,
    pub vpn_mode: bool,
    pub prc_whitelist: bool,
    pub listen_all: bool,
    pub force_protocol: Option<String>,
}

const DAEMON_PATH: &str = "geph4-client";

pub static DAEMON_VERSION: Lazy<String> = Lazy::new(|| {
    let mut cmd = std::process::Command::new(DAEMON_PATH);
    cmd.arg("--version");

    #[cfg(windows)]
    cmd.creation_flags(0x08000000);

    String::from_utf8_lossy(&cmd.output().unwrap().stdout)
        .replace("geph4-client", "")
        .trim()
        .to_string()
});

fn get_rpc_key() -> anyhow::Result<String> {
    let key_dir = dirs::config_dir().context("Unable to get config dir")?;
    let key_path = key_dir.join("rpc_key");
    let mut key_file = File::open(key_path)?;
    let mut contents = String::new();
    key_file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn set_rpc_key() -> anyhow::Result<()> {
    let key_dir = dirs::config_dir()
        .context("Unable to get config dir")?
        .join("geph4-credentials");
    if !key_dir.exists() {
        create_dir_all(&key_dir)?;
    }
    let key_path = key_dir.join("rpc_key");
    let mut key_file = File::create(key_path)?;
    write!(key_file, "{}", GEPH_RPC_KEY.deref())?;
    Ok(())
}

/// Returns the daemon version.
pub fn daemon_version() -> anyhow::Result<String> {
    Ok(DAEMON_VERSION.clone())
}

/// Returns the directory where all the log files are found.
pub fn debugpack_path() -> PathBuf {
    let mut base = dirs::data_local_dir().expect("no local dir");
    base.push("geph4-logs.db");
    base
}

impl DaemonConfig {
    /// Starts the daemon, returning a death handle.
    pub fn start(self) -> anyhow::Result<std::process::Child> {
        // std::env::set_var("GEPH_RPC_KEY", GEPH_RPC_KEY.clone());
        set_rpc_key()?;
        let common_args = Vec::new()
            .tap_mut(|v| {
                v.push("--exit-server".into());
                v.push(self.exit_hostname.clone());
                if let Some(force) = self.force_protocol.clone() {
                    v.push("--force-protocol".into());
                    v.push(force);
                }
                v.push("--debugpack-path".into());
                v.push(debugpack_path().to_string_lossy().to_string());
            })
            .tap_mut(|v| {
                if self.prc_whitelist {
                    v.push("--exclude-prc".into())
                }
            })
            .tap_mut(|v| {
                if self.force_bridges {
                    v.push("--use-bridges".into())
                }
            })
            .tap_mut(|v| {
                if self.listen_all {
                    v.push("--socks5-listen".into());
                    v.push("0.0.0.0:9909".into());
                    v.push("--http-listen".into());
                    v.push("0.0.0.0:9910".into());
                }
            })
            .tap_mut(|v| {
                v.push("auth-password".to_string());
                v.push("--username".to_string());
                v.push(self.username.clone());
                v.push("--password".into());
                v.push(self.password.clone());
            });

        if self.vpn_mode {
            #[cfg(target_os = "linux")]
            {
                let mut cmd = std::process::Command::new("pkexec");
                cmd.arg(DAEMON_PATH);
                cmd.arg("connect");
                cmd.arg("--vpn-mode").arg("tun-route");
                cmd.args(&common_args);
                let child = cmd.spawn().context("cannot spawn non-VPN child")?;
                Ok(child)
            }
            #[cfg(target_os = "windows")]
            {
                if !is_elevated::is_elevated() {
                    anyhow::bail!("VPN mode requires admin privileges on Windows!!!")
                }
                let mut cmd = std::process::Command::new(DAEMON_PATH);
                cmd.arg("connect");
                cmd.arg("--vpn-mode").arg("windivert");
                cmd.args(&common_args);
                #[cfg(windows)]
                cmd.creation_flags(0x08000000);
                let mut child = cmd.spawn().context("cannot spawn non-VPN child")?;
                Ok(child)
            }
            #[cfg(target_os = "macos")]
            {
                anyhow::bail!("VPN mode not supported on macOS")
            }
        } else {
            let mut cmd = std::process::Command::new(DAEMON_PATH);
            cmd.arg("connect");
            cmd.args(&common_args);
            #[cfg(windows)]
            cmd.creation_flags(0x08000000);
            let child = cmd.spawn().context("cannot spawn non-VPN child")?;
            eprintln!("*** CHILD ***");
            Ok(child)
        }
    }
}
