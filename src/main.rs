use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::os::unix::process::CommandExt;

fn sh(cmd: &str) {
    println!("$ {}", cmd);
    let status = Command::new("bash").arg("-c").arg(cmd).status();
    println!("(exit: {:?})", status);
}

fn generate_secret() -> String {
    let mut buf = [0u8; 16];
    if let Ok(mut f) = File::open("/dev/urandom") {
        let _ = f.read_exact(&mut buf);
    }
    buf.iter().map(|b| format!("{:02x}", b)).collect()
}

fn is_valid_secret(s: &str) -> bool {
    s.len() == 32 && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn main() {
    let home = env::var("HOME").unwrap_or_else(|_| "/home/container".to_string());
    let bin_path = format!("{}/mtproto-proxy", home);
    let aes_pwd_path = format!("{}/proxy-secret", home);
    let conf_path = format!("{}/proxy-multi.conf", home);

    // detect architecture
    let arch_out = Command::new("uname").arg("-m").output();
    let arch = match arch_out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => "x86_64".to_string(),
    };
    let asset = if arch == "aarch64" {
        "mtproto-proxy-linux-arm64"
    } else {
        "mtproto-proxy-linux-amd64"
    };

    // download precompiled binary if not already there
    if !Path::new(&bin_path).exists() {
        let url = format!(
            "https://github.com/GetPageSpeed/MTProxy/releases/latest/download/{}",
            asset
        );
        sh(&format!("curl -L -o {} {}", bin_path, url));
        sh(&format!("chmod +x {}", bin_path));
    }

    // always refresh Telegram's official secret + relay config on every start
    sh(&format!(
        "curl -s -o {} https://core.telegram.org/getProxySecret",
        aes_pwd_path
    ));
    sh(&format!(
        "curl -s -o {} https://core.telegram.org/getProxyConfig",
        conf_path
    ));

    // load or generate a valid 32-hex-char proxy secret, persisted across restarts
    let secret_file = format!("{}/mtproxy_secret.txt", home);
    let mut secret = env::var("MTPROXY_SECRET").unwrap_or_default();

    if !is_valid_secret(&secret) {
        secret = fs::read_to_string(&secret_file)
            .unwrap_or_default()
            .trim()
            .to_string();
    }

    if !is_valid_secret(&secret) {
        secret = generate_secret();
        let _ = fs::write(&secret_file, &secret);
    }

    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "443".to_string());

    println!("=====================================================");
    println!("MTProxy is starting (relay mode via Telegram ME servers)");
    println!("Port: {}", port);
    println!("Secret: {}", secret);
    println!("Connect link (replace YOUR_IP with your server IP):");
    println!("tg://proxy?server=YOUR_IP&port={}&secret=dd{}", port, secret);
    println!("=====================================================");

    let err = Command::new(&bin_path)
        .args([
            "-p", "8888",
            "-H", &port,
            "-S", &secret,
            "--aes-pwd", &aes_pwd_path,
            "-M", "1",
            &conf_path,
        ])
        .exec();

    println!("exec failed to start mtproto-proxy: {:?}", err);
}
