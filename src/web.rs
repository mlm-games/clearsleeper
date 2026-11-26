//! The web entry: a dropped disc image becomes the asset source, then the shell starts.
//!
//! The page (the root `index.html` the trunk build consumes) reads the file the player drops and
//! hands its bytes to [`install_disc_image`]. A container this build cannot read comes back as
//! the typed [`DiscError`] message the page shows, not a session that boots without assets.
//! [`boot`] starts the shell loop once that succeeded, because the constructor loads the world's
//! assets and would otherwise run against an empty root.
//!
//! Both entries start by installing [`crate::report`]'s panic hook: this build aborts on a panic,
//! so the message would otherwise reach the page as nothing but a trap.

use wasm_bindgen::prelude::*;

#[wasm_bindgen::prelude::wasm_bindgen(inline_js = "
export function clearsleeper_report_boot_error(message) {
    if (typeof window !== 'undefined' && typeof window.__clearsleeperPanic === 'function') {
        window.__clearsleeperPanic(message);
    } else {
        console.error(message);
    }
}
")]
extern "C" {
    fn clearsleeper_report_boot_error(message: &str);
}

/// Surface a failure to start the shell on the page instead of leaving a blank canvas.
pub fn report_boot_error(message: &str) {
    clearsleeper_report_boot_error(message);
}

use crate::assets;
use crate::disc::MAX_IMAGE_BYTES;

#[wasm_bindgen]
extern "C" {
    /// `index.html` binds this to the console, so the web build's input log lands in the
    /// browser's console the way the desktop build writes it to its log file.
    #[wasm_bindgen(js_name = clearsleeperLog)]
    fn clearsleeper_log(text: &str);

    /// The page's hidden file input, opened by the native start UI's Load Disc button.
    #[wasm_bindgen(js_name = clearsleeperPickDisc)]
    fn js_pick_disc();

    /// The page's hidden directory input, opened by the native start UI's Load Folder button.
    #[wasm_bindgen(js_name = clearsleeperPickFolder)]
    fn js_pick_folder();
}

/// One line of the input log, printed to the browser's console.
pub fn log_line(text: &str) {
    clearsleeper_log(text);
}

thread_local! {
    /// Disc acquisition progress, reported by the page and drawn by the native start UI.
    static DISC_STATUS: std::cell::RefCell<String> =
        const { std::cell::RefCell::new(String::new()) };
}

/// The page's latest disc status, drawn on the native start screen.
pub fn disc_status() -> String {
    DISC_STATUS.with(|slot| slot.borrow().clone())
}

/// Called by the page while it reads and installs a disc image.
#[wasm_bindgen(js_name = clearsleeperDiscStatus)]
pub fn set_disc_status(text: &str) {
    DISC_STATUS.with(|slot| *slot.borrow_mut() = text.to_owned());
}

/// Open the page's file picker from the native start UI's Load Disc button.
pub fn request_disc_file() {
    js_pick_disc();
}

/// Open the page's folder picker from the native start UI's Load Folder button.
pub fn request_disc_folder() {
    js_pick_folder();
}

/// The largest image the page should hand over, which `index.html` checks the picked file
/// against: `wasm-bindgen`'s glue copies the bytes into wasm memory before this entry runs, and
/// an allocation there aborts the tab, so an image over the limit is refused in the page.
#[wasm_bindgen(js_name = discImageLimit)]
pub fn disc_image_limit() -> f64 {
    MAX_IMAGE_BYTES as f64
}

/// Parse `bytes` as a GameCube disc image and install it as the asset source.
///
/// Answers with the image's summary (`GZLE01 (The Legend of Zelda: The Wind Waker), 4055 files`)
/// for the page's status line, or the failure's message.
#[wasm_bindgen(js_name = installDiscImage)]
pub fn install_disc_image(bytes: Vec<u8>) -> Result<String, JsValue> {
    crate::report::install();
    let summary =
        assets::install_disc_image(bytes).map_err(|error| JsValue::from_str(&error.to_string()))?;
    if let Err(detail) = assets::verify_playable() {
        assets::clear_installed();
        return Err(JsValue::from_str(&format!(
            "{summary} mounted but cannot start: {detail}"
        )));
    }
    Ok(summary.to_string())
}

/// Install an extracted disc tree handed over by the folder picker. `bytes` holds every file
/// concatenated in `paths` order; `lengths` says where each one ends.
#[wasm_bindgen(js_name = installExtracted)]
pub fn install_extracted(
    paths: Vec<String>,
    lengths: Vec<u32>,
    bytes: Vec<u8>,
) -> Result<String, JsValue> {
    crate::report::install();
    if paths.len() != lengths.len() {
        return Err(JsValue::from_str("the folder's paths and lengths disagree"));
    }
    let mut cursor = 0usize;
    let mut files = Vec::with_capacity(paths.len());
    for (path, len) in paths.into_iter().zip(lengths) {
        let end = cursor.saturating_add(len as usize);
        let data = bytes.get(cursor..end).ok_or_else(|| {
            JsValue::from_str("the folder's file bytes are shorter than its lengths")
        })?;
        files.push((path, data.to_vec()));
        cursor = end;
    }
    let summary = assets::install_extracted_memory(files)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    if let Err(detail) = assets::verify_playable() {
        assets::clear_installed();
        return Err(JsValue::from_str(&format!(
            "{summary} mounted but cannot start: {detail}"
        )));
    }
    Ok(summary.to_string())
}

#[wasm_bindgen(js_name = forceSettings)]
pub fn force_settings() {
    crate::FORCE_SETTINGS.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Start the shell loop over the installed image.
#[wasm_bindgen(js_name = boot)]
pub fn boot() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static BOOTED: AtomicBool = AtomicBool::new(false);
    if BOOTED.swap(true, Ordering::SeqCst) {
        return;
    }
    crate::report::install();
    crate::run();
    // `repose_platform::run_web_app` installs its own panic hook, dropping the file:line
    // this build reports. Put ours back so a boot panic names its source.
    crate::report::install_again();
}
