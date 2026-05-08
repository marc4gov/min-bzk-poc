fn main() {
    tauri_build::build();

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Only link llama.cpp if the library was actually built
    let llama_cpp_path = std::path::Path::new("third-party/llama.cpp");
    let lib_path = std::path::Path::new("third-party/llama.cpp/build/bin");

    if llama_cpp_path.exists() && lib_path.exists() {
        // Check for the actual library file
        #[cfg(target_os = "macos")]
        let lib_file = lib_path.join("libllama.dylib");

        #[cfg(target_os = "linux")]
        let lib_file = lib_path.join("libllama.so");

        #[cfg(target_os = "windows")]
        let lib_file = lib_path.join("llama.dll");

        if lib_file.exists() {
            println!("cargo:rustc-link-search=native={}", lib_path.display());
            println!("cargo:rustc-link-lib=dylib=llama");

            // Set rpath so the library can be found at runtime
            #[cfg(target_os = "macos")]
            {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());

                // Link required frameworks for macOS
                println!("cargo:rustc-link-lib=framework=Foundation");
                println!("cargo:rustc-link-lib=framework=Metal");
                println!("cargo:rustc-link-lib=framework=CoreML");
                println!("cargo:rustc-link-lib=framework=Accelerate");
            }

            println!("cargo:warning=llama.cpp found and linked at: {}", lib_path.display());
        }
    }

    // Compile Swift bridges on macOS
    #[cfg(target_os = "macos")]
    {
        let bridges_dir = std::path::Path::new("src-tauri/bridges");
        if bridges_dir.exists() {
            for entry in std::fs::read_dir(bridges_dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("swift") {
                    let stem = path.file_stem().unwrap().to_string_lossy();
                    let lib_name = format!("lib{}_bridge.a", stem);
                    let lib_path = format!("{}/{}", out_dir, lib_name);

                    if let Ok(output) = std::process::Command::new("swiftc")
                        .arg("-emit-library")
                        .arg("-static")
                        .arg("-O")
                        .arg("-o")
                        .arg(&lib_path)
                        .arg(&path)
                        .output()
                    {
                        if output.status.success() {
                            println!("cargo:rustc-link-lib=static={}_bridge", stem);
                            println!("cargo:rustc-link-search=native={}", out_dir);
                        }
                    }
                }
            }
        }
    }
}
