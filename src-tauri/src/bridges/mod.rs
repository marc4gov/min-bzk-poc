use std::ffi::CString;
use std::os::raw::c_char;

// Link to the Swift bridge
#[cfg(target_os = "macos")]
#[link(name = "mlx_bridge", kind = "static")]
extern "C" {
    fn mlx_load_model(path: *const c_char) -> *mut std::ffi::c_void;
    fn mlx_generate(
        model: *mut std::ffi::c_void,
        prompt: *const c_char,
        temp: f32,
        top_p: f32,
        max_tokens: i32,
        callback: extern "C" fn(*const c_char),
    );
    fn mlx_unload_model(model: *mut std::ffi::c_void);
}

pub struct MLXBridge {
    model: Option<*mut std::ffi::c_void>,
}

impl MLXBridge {
    pub fn new() -> Self {
        Self { model: None }
    }

    #[cfg(target_os = "macos")]
    pub fn load_model(&mut self, path: &str) -> Result<(), String> {
        let c_path = CString::new(path).map_err(|e| e.to_string())?;
        let model = unsafe { mlx_load_model(c_path.as_ptr()) };

        if model.is_null() {
            return Err("Failed to load model".to_string());
        }

        self.model = Some(model);
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn load_model(&mut self, _path: &str) -> Result<(), String> {
        Err("MLX is only supported on macOS".to_string())
    }

    pub fn generate<F>(&self, prompt: &str, temp: f32, top_p: f32, max_tokens: i32, _callback: F)
    where
        F: FnMut(&str),
    {
        if let Some(model) = self.model {
            let c_prompt = CString::new(prompt).unwrap();
            unsafe {
                extern "C" fn trampoline<F>(_ctx: *const c_char)
                where
                    F: FnMut(&str),
                {
                    // Note: This is a simplified implementation
                    // A proper implementation would need to manage the callback context
                }
                mlx_generate(model, c_prompt.as_ptr(), temp, top_p, max_tokens, trampoline::<F>);
            }
        }
    }
}

impl Drop for MLXBridge {
    fn drop(&mut self) {
        if let Some(model) = self.model {
            unsafe { mlx_unload_model(model) };
        }
    }
}

impl Default for MLXBridge {
    fn default() -> Self {
        Self::new()
    }
}
