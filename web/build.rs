// build.rs
use std::process::Command;

fn main() {
    // Run stylance using the stylance.toml configuration (from workspace root)
    let status = Command::new("stylance")
        .arg("..") // Use parent directory (workspace root) with stylance.toml config
        .status()
        .expect("Failed to execute stylance command");
    
    if !status.success() {
        panic!("stylance failed to generate CSS");
    }
    
    // Tell cargo to rerun build script if CSS modules change
    println!("cargo:rerun-if-changed=src/**/*.module.css");
    println!("cargo:rerun-if-changed=../stylance.toml");
}