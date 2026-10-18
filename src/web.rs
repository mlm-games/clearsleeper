//! The web entry: a dropped disc image becomes the asset source, then the shell starts.
//!
//! The page (the root `index.html` the trunk build consumes) reads the file the player drops and
//! hands its bytes to [`install_disc_image`]. A container this build cannot read comes back as
//! the typed [`DiscError`] message the page shows, not a session that boots without assets.
//! [`boot`] starts the shell loop once that succeeded, because the constructor loads the world's
//! assets and would otherwise run against an empty root.

use wasm_bindgen::prelude::*;

use crate::assets;

/// Parse `bytes` as a GameCube disc image and install it as the asset source.
///
/// Answers with the image's summary (`GZLE01 (The Legend of Zelda: The Wind Waker), 4055 files`)
/// for the page's status line, or the failure's message.
#[wasm_bindgen(js_name = installDiscImage)]
pub fn install_disc_image(bytes: Vec<u8>) -> Result<String, JsValue> {
    assets::install_disc_image(bytes)
        .map(|summary| summary.to_string())
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Start the shell loop over the installed image.
#[wasm_bindgen(js_name = boot)]
pub fn boot() {
    crate::run();
}
