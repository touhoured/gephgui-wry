use anyhow::Context;
use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RpcAuthKind {
    Password { username: String, password: String },
    Signature {},
}
pub fn to_flags(auth_kind: RpcAuthKind) -> anyhow::Result<Vec<String>> {
    match auth_kind {
        RpcAuthKind::Password { username, password } => Ok(vec![
            String::from("auth-password"),
            String::from("--username"),
            username,
            String::from("--password"),
            password,
        ]),
        RpcAuthKind::Signature {} => {
            let sk_path = AppDirs::new(Some("geph4-sk"), false).context("failed to get sk path")?;
            Ok(vec![
                String::from("auth-keypair"),
                String::from("--sk-path"),
                sk_path
                    .config_dir
                    .to_str()
                    .context("converting sk-path to string failllled")?
                    .to_owned(),
            ])
        }
    }
}
