//! Panics, reported where the message would otherwise be lost.
//!
//! A wasm build aborts rather than unwinds, so a panic reaches the page as a trap
//! (`RuntimeError: unreachable executed`) that carries nothing but the fact that the instance
//! died. The hook here reports the message first: `console.error` keeps it in the browser's log
//! and `window.__clearsleeperPanic`, which `index.html` defines, puts it on the drop zone's
//! status line, where a dropped disc's failure belongs. Other platforms already print panics, so
//! there the report is nothing and the hook hands off to the one it replaced.

use std::panic::PanicHookInfo;
use std::sync::Once;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = "
export function clearsleeper_report_panic(message) {
    console.error(message);
    if (typeof window !== 'undefined' && typeof window.__clearsleeperPanic === 'function') {
        window.__clearsleeperPanic(message);
    }
}
")]
extern "C" {
    fn clearsleeper_report_panic(message: &str);
}

/// Report panics through the wasm snippet below, once, keeping whatever hook was installed
/// before.
pub fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            report(&message(info));
            previous(info);
        }));
    });
}

fn message(info: &PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    let text = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("panicked");
    match info.location() {
        Some(location) => format!("{text} ({}:{})", location.file(), location.line()),
        None => text.to_owned(),
    }
}

#[cfg(target_arch = "wasm32")]
fn report(message: &str) {
    clearsleeper_report_panic(message);
}

#[cfg(not(target_arch = "wasm32"))]
fn report(_message: &str) {}
