fn main() {
    tauri_build::build();

    // Compile Swift bridge on macOS - optional
    #[cfg(target_os = "macos")]
    {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let swift_file = "src-tauri/bridges/mlx.swift";
        let lib_path = format!("{}/libmlx_bridge.a", out_dir);

        // Check if swiftc is available
        let swiftc_available = std::process::Command::new("swiftc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if swiftc_available {
            let output = std::process::Command::new("swiftc")
                .arg("-emit-library")
                .arg("-O")
                .arg("-o")
                .arg(&lib_path)
                .arg(swift_file)
                .output();

            match output {
                Ok(result) if result.status.success() => {
                    println!("cargo:rustc-link-lib=static=mlx_bridge");
                    println!("cargo:rustc-link-search=native={}", out_dir);
                }
                _ => {
                    println!("cargo:warning=Swift bridge compilation failed, continuing without it");
                }
            }
        } else {
            println!("cargo:warning=swiftc not found, skipping MLX bridge compilation");
        }
    }
}
