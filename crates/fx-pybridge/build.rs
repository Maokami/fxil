// crates/fx-pybridge/build.rs
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=PYO3_PYTHON");

    if let Ok(python_interpreter_path) = env::var("PYO3_PYTHON") {
        let get_site_packages_cmd = "import sysconfig; print(sysconfig.get_path('platlib'))";

        match Command::new(&python_interpreter_path)
            .arg("-c")
            .arg(get_site_packages_cmd)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let site_packages_path = String::from_utf8(output.stdout)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();

                    if !site_packages_path.is_empty() && PathBuf::from(&site_packages_path).exists() {
                        println!("cargo:rustc-env=FX_PYBRIDGE_TEST_SITE_PACKAGES={}", site_packages_path);
                        // For debugging purposes, uncomment the following lines to see the output:
                        // eprintln!("build.rs: Set FX_PYBRIDGE_TEST_SITE_PACKAGES to {}", site_packages_path);
                    } else {
                        // eprintln!("build.rs: Failed to get valid site-packages path from Python. Output: '{}'", site_packages_path);
                    }
                } else {
                    // let stderr = String::from_utf8_lossy(&output.stderr);
                    // eprintln!("build.rs: Python script for site-packages failed. Stderr: {}", stderr);
                }
            }
            Err(_e) => {
                // eprintln!("build.rs: Failed to execute Python interpreter at {}: {}", python_interpreter_path, e);
            }
        }
    } else {
        // eprintln!("build.rs: PYO3_PYTHON environment variable is not set. Cannot determine site-packages for tests.");
    }
}