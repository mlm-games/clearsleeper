use std::collections::BTreeMap;

use crate::input::{PadBridge, PadSnapshot};
use repame_shell::{GamepadEvent, GamepadPoller};

pub struct PadSource {
    poller: GamepadPoller,
    ports: BTreeMap<u32, PadBridge>,
}

impl PadSource {
    pub fn new() -> Self {
        Self {
            poller: GamepadPoller::new(),
            ports: BTreeMap::new(),
        }
    }

    pub fn poll(&mut self) -> PadSnapshot {
        // tww/src/m_Do/m_Do_controller_pad.cpp:91-101: each port has its own state.
        for event in self.poller.poll() {
            let id = event.id().0;
            if matches!(event, GamepadEvent::Disconnected { .. }) {
                self.ports.remove(&id);
            } else {
                self.ports.entry(id).or_default().feed(&event);
            }
        }
        self.ports
            .first_key_value()
            .map_or_else(PadSnapshot::default, |(_, pad)| pad.snapshot())
    }
}

impl Default for PadSource {
    fn default() -> Self {
        Self::new()
    }
}
