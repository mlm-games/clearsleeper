//! The session's asset source, resolved before the shell boots: which image to read, from where.
//!
//! The reader ([`crate::disc`]) and the seam it feeds ([`crate::assets::AssetRoot`]) are platform
//! neutral; this module is the adapter between them and the launch. Desktop takes
//! `--disc <path>`, a bare image path, or the default `disc.iso`/`disc.rvz` beside the executable
//! or in the app's data dir; Android hands the document the platform picker returned over
//! (`src/android.rs`); the web's drop zone calls [`crate::assets::install_disc_image`] directly.
//! Every platform ends in that same install, so no platform grows a second reader of the image.
//!
//! An image a launch names must open: a file that cannot be read, or bytes that are not one of
//! the four Wind Waker discs [`crate::disc`] accepts, is a typed [`SourceError`] - never a
//! session that boots with an empty world and leaves the reason in a log line. An image that is
//! simply absent is not an error: the extracted [`crate::assets::ASSETS_ENV`] tree, the
//! development fallback on every platform, takes over.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::disc::{DiscError, DiscSummary};

/// The names a default image is looked for under, beside the executable and in the data dir.
pub const DEFAULT_DISC_NAMES: [&str; 2] = ["disc.iso", "disc.rvz"];

/// The desktop entry's one-line usage, printed for `--help` and after a launch error.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub const USAGE: &str = "usage: clearsleeper [--disc <image.iso|image.gcm|image.rvz>] [<image>]";

#[derive(Debug)]
pub enum SourceError {
    /// The named image path could not be read.
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The bytes read are not an image [`DiscImage::open`](crate::disc::DiscImage::open) accepts.
    Image { path: PathBuf, source: DiscError },
    /// The platform picker could not hand a document over.
    #[cfg(target_os = "android")]
    Picker {
        source: rlobkit_dialogs::RlobKitError,
    },
    /// A document the picker handed over could not be read out of its content URI.
    #[cfg(target_os = "android")]
    Document {
        name: String,
        source: rlobkit_dialogs::RlobKitError,
    },
    /// The launch arguments name something the desktop entry does not take.
    Usage { detail: String },
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "cannot read the disc image {}: {source}", path.display())
            }
            Self::Image { path, source } => {
                write!(f, "cannot use the disc image {}: {source}", path.display())
            }
            #[cfg(target_os = "android")]
            Self::Picker { source } => write!(f, "the file picker failed: {source}"),
            #[cfg(target_os = "android")]
            Self::Document { name, source } => {
                write!(f, "cannot read the picked document {name}: {source}")
            }
            Self::Usage { detail } => write!(f, "{detail}"),
        }
    }
}

impl std::error::Error for SourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Image { source, .. } => Some(source),
            #[cfg(target_os = "android")]
            Self::Picker { source } => Some(source),
            #[cfg(target_os = "android")]
            Self::Document { source, .. } => Some(source),
            Self::Usage { .. } => None,
        }
    }
}

/// Read `path` and install it as the session's image, through the same seam the web drop zone
/// and the Android picker call.
pub fn install_file(path: &Path) -> Result<DiscSummary, SourceError> {
    let bytes = std::fs::read(path).map_err(|source| SourceError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    install_bytes(path, bytes)
}

/// Parse `bytes` as an image and install it; `label` is the name the failures report. The bytes
/// are taken as they were handed over, so a platform holding a shareable buffer does not copy
/// the disc.
pub fn install_bytes(label: &Path, bytes: impl Into<Vec<u8>>) -> Result<DiscSummary, SourceError> {
    crate::assets::install_disc_image(bytes).map_err(|source| SourceError::Image {
        path: label.to_path_buf(),
        source,
    })
}

/// Install the first candidate that exists, in the order given. `None` when none is there; an
/// image that is there but unreadable is its typed failure and stops the walk, rather than
/// falling through to the next name and silently playing a different disc.
pub fn install_first(
    candidates: impl IntoIterator<Item = PathBuf>,
) -> Option<Result<DiscSummary, SourceError>> {
    for path in candidates {
        if path.is_file() {
            return Some(install_file(&path));
        }
    }
    None
}

/// What a desktop launch names: the disc image to install, when one is named.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Launch {
    pub disc: Option<PathBuf>,
}

/// Parse the desktop arguments: `--disc <path>` (or `--disc=<path>`), or one bare image path.
/// Anything else is [`SourceError::Usage`]; no image at all is an empty launch and leaves the
/// defaults and the extracted fallback in charge.
pub fn parse_launch(args: impl IntoIterator<Item = OsString>) -> Result<Launch, SourceError> {
    let mut launch = Launch::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let text = arg.to_str();
        if arg == "--disc" {
            let path = args.next().ok_or_else(|| SourceError::Usage {
                detail: "--disc needs an image path".to_owned(),
            })?;
            set_disc(&mut launch, PathBuf::from(path))?;
        } else if let Some(path) = text.and_then(|text| text.strip_prefix("--disc=")) {
            set_disc(&mut launch, PathBuf::from(path))?;
        } else if text.is_some_and(|text| text.starts_with('-')) {
            return Err(SourceError::Usage {
                detail: format!("unknown option {}", arg.to_string_lossy()),
            });
        } else {
            set_disc(&mut launch, PathBuf::from(arg))?;
        }
    }
    Ok(launch)
}

fn set_disc(launch: &mut Launch, path: PathBuf) -> Result<(), SourceError> {
    if launch.disc.is_some() {
        return Err(SourceError::Usage {
            detail: format!("more than one disc image was named ({})", path.display()),
        });
    }
    launch.disc = Some(path);
    Ok(())
}

/// The defaults a desktop launch tries, in order: `DEFAULT_DISC_NAMES` beside the executable,
/// then under the app's data dir ([`crate::save::slots_dir`]). The exe dir covers a zip the game
/// shipped in, the data dir a copy installed once for every build next to the saves.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn default_disc_paths() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(exe) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        dirs.push(exe);
    }
    dirs.push(crate::save::slots_dir());
    dirs.into_iter()
        .flat_map(|dir| DEFAULT_DISC_NAMES.map(|name| dir.join(name)))
        .collect()
}

/// Resolve a desktop launch: the named image first, then the default candidates. `Ok(None)` is
/// "no disc anywhere", which leaves [`AssetRoot::from_env`](crate::assets::AssetRoot::from_env)
/// to read the extracted tree or report the empty root.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn install_launch(
    args: impl IntoIterator<Item = OsString>,
) -> Result<Option<DiscSummary>, SourceError> {
    let launch = parse_launch(args)?;
    if let Some(path) = &launch.disc {
        return install_file(path).map(Some);
    }
    match install_first(default_disc_paths()) {
        Some(installed) => installed.map(Some),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_arguments_name_one_image_or_none() {
        let launch = parse_launch(["--disc", "a.iso"].map(OsString::from)).unwrap();
        assert_eq!(launch.disc, Some(PathBuf::from("a.iso")));

        let launch = parse_launch([OsString::from("b.rvz")]).unwrap();
        assert_eq!(launch.disc, Some(PathBuf::from("b.rvz")));

        let launch = parse_launch([OsString::from("--disc=c.gcm")]).unwrap();
        assert_eq!(launch.disc, Some(PathBuf::from("c.gcm")));

        assert_eq!(parse_launch([]).unwrap(), Launch::default());

        assert!(matches!(
            parse_launch([OsString::from("--disc")]),
            Err(SourceError::Usage { .. })
        ));
        assert!(matches!(
            parse_launch([OsString::from("--nope")]),
            Err(SourceError::Usage { .. })
        ));
        assert!(matches!(
            parse_launch([OsString::from("a.iso"), OsString::from("b.iso")]),
            Err(SourceError::Usage { .. })
        ));
    }
}
