use std::process::Command;

fn run(label: &str, cmd: &str) {
    println!("=== {} ===", label);
    let output = Command::new("bash").arg("-c").arg(cmd).output();
    match output {
        Ok(o) => {
            println!("{}", String::from_utf8_lossy(&o.stdout));
            println!("{}", String::from_utf8_lossy(&o.stderr));
        }
        Err(e) => println!("failed to run: {}", e),
    }
}

fn main() {
    run("whoami", "whoami");
    run("gcc", "gcc --version");
    run("make", "make --version");
    run("apt-get", "apt-get update 2>&1 | head -5");
    run("port env", "echo $SERVER_PORT");
}
