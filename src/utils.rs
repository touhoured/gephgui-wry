use geph4_client::config::AuthKind;

pub fn to_flags(auth_kind: AuthKind) -> Vec<String> {
    match credentials {
        AuthKind::AuthPassword { username, password } => vec![
            String::from("password"),
            String::from("--username"),
            username,
            String::from("--password"),
            password,
        ],
        AuthKind::AuthSignature { secret } => vec![
            String::from("signature"),
            String::from("--private-key"),
            secret,
        ],
    }
}
