fn main() {
    tauri_build::build();

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Compile Swift bridges on macOS
    #[cfg(target_os = "macos")]
    {
        // Check if llama.swift exists and compile it
        let llama_swift = "src-tauri/bridges/llama.swift";
        if std::path::Path::new(llama_swift).exists() {
            let lib_path = format!("{}/libllama_bridge.a", out_dir);

            if let Ok(_) = std::process::Command::new("swiftc")
                .arg("-emit-library")
                .arg("-O")
                .arg("-o")
                .arg(&lib_path)
                .arg(llama_swift)
                .output()
            {
                println!("cargo:rustc-link-lib=static=llama_bridge");
                println!("cargo:rustc-link-search=native={}", out_dir);
                println!("cargo:warning=llama bridge compiled successfully");
            } else {
                println!("cargo:warning=llama bridge compilation failed, using mock engine");
            }
        }

        // Check if mlx.swift exists and compile it (for MLX backend)
        let mlx_swift = "src-tauri/bridges/mlx.swift";
        if std::path::Path::new(mlx_swift).exists() {
            let lib_path = format!("{}/libmlx_bridge.a", out_dir);

            if let Ok(_) = std::process::Command::new("swiftc")
                .arg("-emit-library")
                .arg("-O")
                .arg("-o")
                .arg(&lib_path)
                .arg(mlx_swift)
                .output()
            {
                println!("cargo:rustc-link-lib=static=mlx_bridge");
                println!("cargo:rustc-link-search=native={}", out_dir);
                println!("cargo:warning=MLX bridge compiled successfully");
            }
        }

        // Link against required system frameworks
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=CoreML");
    }
}
