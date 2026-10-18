//! The Android entry: the platform picker's document becomes the asset source.
//!
//! A phone has no command line and no extracted tree, so the image arrives through the system's
//! file picker: `RlobKitPickerActivity` (the helper activity `rlobkit-dialogs` contributes to the
//! APK) hands the chosen SAF content URI back, the document is read once into memory and
//! installed through the same [`crate::assets::install_disc_image`] seam the web drop zone
//! calls. The picker call blocks this thread - the native `android_main` thread, not the Java UI
//! thread - which is what lets the activity result the platform delivers on the UI thread land
//! while the call waits.
//!
//! A `disc.iso`/`disc.rvz` already in the app's own data dir (`adb push` into the external files
//! dir survives, the internal one needs `run-as`) skips the picker, mirroring the desktop
//! default. A cancelled pick leaves no image installed, and the extracted
//! [`crate::assets::ASSETS_ENV`] fallback - effectively unset on a device - decides from there.

use std::path::{Path, PathBuf};

use rlobkit_dialogs::{OpenFileOptions, RlobKit, RlobKitType};
use winit::platform::android::activity::AndroidApp;

use crate::disc::DiscSummary;
use crate::source::{self, SourceError};

/// The image extensions the picker offers.
const DISC_EXTENSIONS: [&str; 3] = ["iso", "gcm", "rvz"];

/// The MIME types the picker filters on. Android reports a `.gcm` or `.rvz` as
/// `application/octet-stream`, so a filter of `application/x-iso9660-image` alone would hide the
/// compressed images the reader accepts.
const DISC_MIME_TYPES: [&str; 2] = ["application/x-iso9660-image", "application/octet-stream"];

const PICKER_TITLE: &str = "Choose a GameCube disc image";

/// Install the session's image from the platform: the app data dir's default first, the picker
/// after. `Ok(None)` is a cancelled pick with no usable default, which leaves the extracted
/// fallback in charge.
pub fn install_source(app: &AndroidApp) -> Result<Option<DiscSummary>, SourceError> {
    rlobkit_dialogs::init_with_android_context(app.vm_as_ptr(), app.activity_as_ptr());
    let candidates = data_dirs(app)
        .into_iter()
        .flat_map(|dir| source::DEFAULT_DISC_NAMES.map(|name| dir.join(name)));
    match source::install_first(candidates) {
        Some(Ok(summary)) => return Ok(Some(summary)),
        // A default that exists but is unreadable is logged with its typed reason and does not
        // lock the player out of the picker; deleting it is not the only way forward.
        Some(Err(error)) => {
            log::warn!("clearsleeper: default disc image unusable ({error}); opening the picker")
        }
        None => {}
    }
    pick()
}

/// The app's own data dirs, external before internal: the external files dir is the one an `adb
/// push` reaches without `run-as`, and it is visible over MTP.
fn data_dirs(app: &AndroidApp) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(dir) = app.external_data_path() {
        dirs.push(dir);
    }
    if let Some(dir) = app.internal_data_path() {
        dirs.push(dir);
    }
    dirs
}

/// Open the picker and install what it hands back. A cancelled pick is `None`; a document the
/// platform cannot read out of its URI is the typed failure, never a boot that hides it.
fn pick() -> Result<Option<DiscSummary>, SourceError> {
    let options = OpenFileOptions {
        file_type: RlobKitType::Custom {
            extensions: DISC_EXTENSIONS
                .iter()
                .map(|ext| (*ext).to_owned())
                .collect(),
            mime_types: DISC_MIME_TYPES
                .iter()
                .map(|mime| (*mime).to_owned())
                .collect(),
        },
        title: Some(PICKER_TITLE.to_owned()),
        ..Default::default()
    };
    let picked = futures_lite::future::block_on(RlobKit::open_file_picker(options))
        .map_err(|source| SourceError::Picker { source })?;
    let Some(file) = picked.and_then(|files| files.into_iter().next()) else {
        return Ok(None);
    };
    let name = file.name().to_owned();
    let bytes = file.read_bytes().map_err(|source| SourceError::Document {
        name: name.clone(),
        source,
    })?;
    source::install_bytes(Path::new(&name), bytes).map(Some)
}
