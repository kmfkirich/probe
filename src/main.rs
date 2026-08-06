use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::os::unix::process::CommandExt;

fn sh(cmd: &str) {
    println!("$ {}", cmd);
    let status = Command::new("bash").arg("-c").arg(cmd).status();
    println!("(exit: {:?})", status);
}

fn main() {
    let home = env::var("HOME").unwrap_or_else(|_| "/home/container".to_string());
    let bin_path = format!("{}/mtproto-proxy", home);

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

    // persist a random secret across restarts
    let secret_file = format!("{}/mtproxy_secret.txt", home);
    let secret = if let Ok(s) = env::var("MTPROXY_SECRET") {
        s
    } else if Path::new(&secret_file).exists() {
        fs::read_to_string(&secret_file)
            .unwrap_or_default()
            .trim()
            .to_string()
    } else {
        let out = Command::new("bash")
            .arg("-c")
            .arg("head -c 16 /dev/urandom | xxd -ps")
            .output();
        let s = match out {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => "0123456789abcdef0123456789abcdef".to_string(),
        };
        let _ = fs::write(&secret_file, &s);
        s
    };

    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "443".to_string());

    println!("=====================================================");
    println!("MTProxy is starting");
    println!("Port: {}", port);
    println!("Secret: {}", secret);
    println!("Connect link (replace YOUR_IP with your server IP):");
    println!("tg://proxy?server=YOUR_IP&port={}&secret=dd{}", port, secret);
    println!("=====================================================");

    let err = Command::new(&bin_path)
        .args([
            "-S", &secret,
            "-H", &port,
            "--direct",
            "-p", "8888",
            "--aes-pwd", "/dev/null",
        ])
        .exec();

    println!("exec failed to start mtproto-proxy: {:?}", err);
}
