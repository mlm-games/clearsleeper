use crate::j2d::{self, Screens};
use crate::meter_hud::MeterAssets;
use crate::sim::flags::EventFlags;
use crate::text::msgproc::MsgProcs;
use crate::text::raster::Canvas;
use crate::ui_meter_gauges::{GaugeEnv, GaugeOutput, MeterGauges, MeterTuning as GaugeTuning};
use crate::ui_meter_vitals::{Vitals, VitalsEnv};
use crate::ui_meter_widgets::{self, Input, MeterPeers, Widget};

/// Facts from the sim the HUD widgets need. `d_meter.cpp:7734-7750` binds the same screens
/// and the widgets read the save/play state through `dComIfGp_*`.
#[derive(Clone, Debug, Default)]
pub struct HudFacts {
    pub rupees: i32,
    pub keys: i32,
    pub events: EventFlags,
    pub equip: Vec<u8>,
    pub day_hour: i32,
    pub day_minute: i32,
    pub map_alpha: u8,
    pub stage_type: u32,
}

/// Owns the full retail meter widget set alongside the heart row. The ported state machines
/// are driven through `execute`/`draw`, and their quads are blitted onto the HUD canvas.
pub struct HudWidgets {
    widget: Widget,
    input: Input,
    vitals: Vitals,
    vitals_env: VitalsEnv,
    gauges: MeterGauges,
    gauge_env: GaugeEnv,
    gauge_out: GaugeOutput,
    gauge_tuning: GaugeTuning,
    procs: MsgProcs,
    created: bool,
}

impl Default for HudWidgets {
    fn default() -> Self {
        Self {
            widget: Widget::default(),
            input: Input::default(),
            vitals: Vitals::default(),
            vitals_env: VitalsEnv::default(),
            gauges: MeterGauges::default(),
            gauge_env: GaugeEnv::default(),
            gauge_out: GaugeOutput::default(),
            gauge_tuning: GaugeTuning::default(),
            procs: MsgProcs::default(),
            created: false,
        }
    }
}

fn screen<'a>(screens: &'a Screens, label: &str) -> Option<&'a j2d::Screen> {
    screens
        .cache
        .iter()
        .find(|(name, _)| name == label)
        .map(|(_, screen)| screen)
}

impl HudWidgets {
    pub fn create(&mut self, assets: &MeterAssets) -> bool {
        if self.created {
            return true;
        }
        let screens = &assets.screens;
        let (Some(p1), Some(p2), Some(p3)) = (
            screen(screens, ui_meter_widgets::SCREEN_PARTS1),
            screen(screens, ui_meter_widgets::SCREEN_PARTS2),
            screen(screens, ui_meter_widgets::SCREEN_PARTS3),
        ) else {
            return false;
        };
        let (Some(t1), Some(t2), Some(road)) = (
            screen(screens, crate::ui_meter_gauges::SCREEN_TIME_SWIM1),
            screen(screens, crate::ui_meter_gauges::SCREEN_TIME_SWIM2),
            screen(screens, ui_meter_widgets::SCREEN_CHOICE_ROAD),
        ) else {
            return false;
        };
        self.widget = ui_meter_widgets::create(p1, p2, p3, t1, t2, road, &mut self.input);
        self.created = true;
        true
    }

    pub fn step(&mut self, assets: &MeterAssets, facts: &HudFacts, canvas: &mut Canvas) {
        if !self.create(assets) {
            return;
        }
        self.input.rupee = facts.rupees;
        self.input.key_num = facts.keys;
        self.input.events = facts.events.clone();
        self.input.equip_slot = facts.equip.clone();
        self.input.day_hour = facts.day_hour;
        self.input.day_minute = facts.day_minute;
        self.input.map_alpha = facts.map_alpha;
        self.input.stage_type = facts.stage_type;

        let mut peers = MeterPeers {
            vitals: &mut self.vitals,
            vitals_env: &mut self.vitals_env,
            gauges: &mut self.gauges,
            gauge_env: &mut self.gauge_env,
            gauge_out: &mut self.gauge_out,
            gauge_tuning: &self.gauge_tuning,
            procs: &mut self.procs,
        };
        ui_meter_widgets::execute(&mut self.widget, &mut self.input, &mut peers);
        let draw = ui_meter_widgets::draw(&mut self.widget, &self.input, &mut peers);
        for quad in &draw.quads {
            if !quad.visible || quad.w <= 0.0 || quad.h <= 0.0 {
                continue;
            }
            let rect = [quad.x, quad.y, quad.x + quad.w, quad.y + quad.h];
            let tint = [quad.white.r, quad.white.g, quad.white.b, quad.white.a];
            let alpha = f32::from(quad.alpha) / 255.0;
            if let Some(name) = quad.texture.as_deref() {
                if let Some(image) = assets.textures.get(name) {
                    crate::panes::blit_texture(canvas, image, rect, tint, alpha);
                    continue;
                }
            }
            canvas.fill_rect(rect, [tint[0], tint[1], tint[2], quad.alpha]);
        }
    }
}
