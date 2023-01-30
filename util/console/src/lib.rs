#[cfg(not(target_arch = "wasm32"))]
pub fn _println(s: &str) {
    print!("{s}");
}

#[cfg(target_arch = "wasm32")]
pub fn _println(s: &str) {
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(s))
}

#[macro_export]
macro_rules! log {
    () => {
        $crate::_println("")
    };
    ($($arg:tt)*) => {{
        $crate::_println(&format!($($arg)*));
    }};
}
