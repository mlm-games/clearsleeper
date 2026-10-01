//! Entry: desktop bin; web boots from the lib (`wasm_bindgen(start)`),
//! Android NativeActivity uses `android_main` (`cargo rapk`).

#[cfg(not(target_os = "android"))]
fn main() {
    clearsleeper::run();
}

#[cfg(target_os = "android")]
fn main() {}
