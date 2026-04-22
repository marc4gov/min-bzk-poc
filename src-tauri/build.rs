fn main() {
    tauri_build::build();

    // Compile Swift bridge on macOS
    #[cfg(target_os = "macos")]
    {
        let swift_files = vec!["bridges/mlx.swift"];
        let out_dir = std::env::var("OUT_DIR").unwrap();

        for swift_file in swift_files {
            let output = std::process::Command::new("swiftc")
                .arg("-emit-library")
                .arg("-O")
                .arg("-target")
                .arg("x86_64-apple-macosx")
                .arg("-o")
                .arg(format!("{}/libmlx_bridge.a", out_dir))
                .arg(format!("src-tauri/{}", swift_file))
                .output();

            match output {
                Ok(result) => {
                    if !result.status.success() {
                        eprintln!("Swift compilation failed: {}", String::from_utf8_lossy(&result.stderr));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to execute swiftc: {}", e);
                }
            }
        }

        println!("cargo:rustc-link-lib=static=mlx_bridge");
        println!("cargo:rustc-link-search=native={}", out_dir);
    }
}
