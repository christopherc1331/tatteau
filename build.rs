// build.rs
use std::process::Command;

fn main() {
    // Run stylance to generate scoped CSS
    let status = Command::new("stylance")
        .args(["web", "--output-file", "target/site/stylance.css", "--folder", "src"])
        .status()
        .expect("Failed to execute stylance command");
    
    if !status.success() {
        panic!("stylance failed to generate CSS");
    }
    
    // Tell cargo to rerun build script if CSS modules change
    println!("cargo:rerun-if-changed=web/src/**/*.module.css");
    println!("cargo:rerun-if-changed=stylance.toml");
}