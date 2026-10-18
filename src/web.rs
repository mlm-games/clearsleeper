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

use crate::assets;
use crate::disc::MAX_IMAGE_BYTES;

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
    assets::install_disc_image(bytes)
        .map(|summary| summary.to_string())
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Start the shell loop over the installed image.
#[wasm_bindgen(js_name = boot)]
pub fn boot() {
    crate::report::install();
    crate::run();
}
