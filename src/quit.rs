use repose_core::{Modifier, View};
use repose_ui::ZStack;

pub struct Quit {
    latched: bool,
}

impl Default for Quit {
    fn default() -> Self {
        Self { latched: false }
    }
}

impl Quit {
    pub fn request(&mut self) {
        self.latched = true;
        exit_now();
    }

    pub fn latched(&self) -> bool {
        self.latched
    }

    pub fn parked() -> View {
        ZStack(Modifier::new().fill_max_size())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn exit_now() {
    std::process::exit(0);
}

#[cfg(target_arch = "wasm32")]
fn exit_now() {}
