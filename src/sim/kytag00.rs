use bevy_ecs::prelude::*;

use super::actor::Actor;
use super::flags::Switches;
use super::player::Player;

pub const KYTAG00_ACTORS: [&str; 3] = ["ky_tag0", "ky00you", "kytag00"];

pub fn is_kytag00(name: &str) -> bool {
    KYTAG00_ACTORS.contains(&name)
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CameraEye(pub glam::Vec3);

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct EnvLight {
    pub rain_count: f32,
    pub rain_count_orig: f32,
    pub snow_count: f32,
    pub moya_count: f32,
    pub moya_mode: u8,
    pub housi_count: f32,
    pub thunder_mode: u8,
    pub colpat_prev_gather: u8,
    pub colpat_curr_gather: u8,
    pub colpat_blend_gather: f32,
    pub colpat_mode_gather: u8,
    pub colpat_weather: u8,
    pub envr_idx_prev: u8,
    pub envr_idx_curr: u8,
}

fn add_calc(value: &mut f32, target: f32, scale: f32, max_step: f32, min_step: f32) {
    let diff = target - *value;
    let step = diff * scale;
    let clamped = if step.abs() < min_step {
        min_step
    } else {
        step
    };
    let limited = clamped.clamp(-max_step, max_step);
    *value += limited;
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Kytag00 {
    pub psel_idx: u8,
    pub ef_mode: u8,
    pub thickness: i32,
    pub inner_fade_y: i32,
    pub switch_no: u8,
    pub invert: u8,
    pub mode: u8,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub target: f32,
    pub psel_set: bool,
    pub ef_set: bool,
    pub scale_y: f32,
}

impl Kytag00 {
    pub fn spawn(parameters: u32, angle: [i16; 3], scale: [f32; 3], switches: &Switches) -> Self {
        let psel_idx = (parameters & 0xFF) as u8;
        let ef_mode = ((parameters >> 8) & 0xFF) as u8;
        let mut thickness = ((parameters >> 16) & 0xFF) as i32;
        let mut inner_fade_y = ((parameters >> 24) & 0xFF) as i32;
        let switch_no = (angle[0] & 0xFF) as u8;
        let invert = ((angle[0] >> 8) & 0xFF) as u8;
        let mode = (angle[2] & 0xFF) as u8;

        let on = switch_no != 0xFF && switches.test(switch_no);
        let target = if invert == 0 {
            if on { 0.0 } else { 1.0 }
        } else if on {
            1.0
        } else {
            0.0
        };

        if thickness == 0xFF {
            thickness = 10;
        }
        if inner_fade_y == 0xFF {
            inner_fade_y = 10;
        }

        let (inner_radius, outer_radius) = if mode == 0 {
            (
                scale[0] * 5000.0,
                scale[0] * 5000.0 + thickness as f32 * 100.0,
            )
        } else {
            (scale[0] * 500.0, scale[0] * 500.0 + thickness as f32 * 10.0)
        };

        Self {
            psel_idx,
            ef_mode,
            thickness,
            inner_fade_y,
            switch_no,
            invert,
            mode,
            inner_radius,
            outer_radius,
            target,
            psel_set: false,
            ef_set: false,
            scale_y: scale[1],
        }
    }
}

fn check_pos(
    tag: &Kytag00,
    actor: &Actor,
    eye: glam::Vec3,
    player: glam::Vec3,
    events: bool,
) -> glam::Vec3 {
    let camera_dist = actor.pos.distance(eye);
    let player_dist = actor.pos.distance(player);
    if events && tag.mode == 0 {
        if camera_dist < player_dist {
            eye
        } else {
            player
        }
    } else {
        player
    }
}

fn blend_of(tag: &Kytag00, actor: &Actor, pos: glam::Vec3) -> Option<f32> {
    let fade_y = tag.inner_fade_y as f32 * 100.0;
    let flat = glam::Vec3::new(pos.x, actor.pos.y, pos.z);
    let dist = actor.pos.distance(flat);
    let top = actor.pos.y + tag.scale_y * 5000.0;
    if dist < tag.outer_radius
        && pos.y >= actor.pos.y - fade_y
        && pos.y < top + fade_y
        && tag.target > 0.0
    {
        let mut blend = 1.0f32;
        let mut vert = 1.0f32;
        let fade_radius = tag.outer_radius - tag.inner_radius;
        if fade_radius != 0.0 {
            blend = (tag.outer_radius - dist) / fade_radius;
            if blend > 1.0 {
                blend = 1.0;
            }
        }
        if pos.y <= actor.pos.y {
            if fade_y != 0.0 {
                vert = (actor.pos.y - pos.y) / fade_y;
                if vert < 0.0 {
                    vert = 0.0;
                }
                vert = 1.0 - vert;
            }
        } else if fade_y != 0.0 {
            vert = (pos.y - top) / fade_y;
            if vert < 0.0 {
                vert = 0.0;
            }
            vert = 1.0 - vert;
        }
        Some(blend * vert * tag.target)
    } else {
        None
    }
}

fn rain_set(count: f32, light: &mut EnvLight, night_stop: bool) {
    let full = count * count * count * 250.0;
    let new_count = if night_stop {
        if light.rain_count < full as i32 as f32 {
            full
        } else {
            0.0
        }
    } else {
        full
    };
    if new_count > light.rain_count_orig {
        light.rain_count = new_count;
    }
}

fn rain_cut(light: &mut EnvLight, night_stop: bool) {
    if !night_stop {
        light.rain_count = light.rain_count_orig;
    }
}

pub fn control(
    mut tags: Query<(&mut Kytag00, &Actor)>,
    players: Query<&Player>,
    camera: Res<CameraEye>,
    mut light: ResMut<EnvLight>,
    switches: Res<Switches>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    let eye = camera.0;
    let player_pos = player.pos;
    for (mut tag, actor) in tags.iter_mut() {
        if tag.switch_no != 0xFF && switches.test(tag.switch_no) {
            add_calc(&mut tag.target, 1.0, 0.1, 0.01, 0.0001);
        } else {
            add_calc(&mut tag.target, 0.0, 0.1, 0.01, 0.0001);
        }
        let snapshot = *tag;
        let pos = check_pos(&snapshot, actor, eye, player_pos, true);
        weather_move(&snapshot, actor, pos, &mut light);
        effect_move(&mut tag, actor, pos, &mut light);
    }
}

fn weather_move(tag: &Kytag00, actor: &Actor, pos: glam::Vec3, light: &mut EnvLight) {
    let Some(blend) = blend_of(tag, actor, pos) else {
        return;
    };
    if light.envr_idx_prev != light.envr_idx_curr {
        return;
    }
    light.colpat_prev_gather = match tag.psel_idx {
        0 => light.colpat_weather,
        1 => {
            if blend > 0.5 {
                light.colpat_prev_gather = light.colpat_weather;
                light.colpat_curr_gather = 1;
            } else {
                light.colpat_prev_gather = 1;
                light.colpat_curr_gather = light.colpat_weather;
            }
            light.colpat_weather
        }
        _ => light.colpat_prev_gather,
    };
    match tag.psel_idx {
        0 => {
            light.colpat_curr_gather = 0;
            light.colpat_blend_gather = blend;
            light.colpat_mode_gather = 1;
        }
        1 => {
            light.colpat_curr_gather = 1;
            light.colpat_blend_gather = blend.min(1.0);
            light.colpat_mode_gather = 1;
        }
        2 => {
            light.colpat_curr_gather = 2;
            light.colpat_blend_gather = blend.min(1.0);
            light.colpat_mode_gather = 1;
        }
        _ => {}
    }
}

fn effect_move(tag: &mut Kytag00, actor: &Actor, pos: glam::Vec3, light: &mut EnvLight) {
    let Some(blend) = blend_of(tag, actor, pos) else {
        if tag.ef_set {
            tag.ef_set = false;
            clear_effect(tag, light);
        }
        return;
    };
    tag.ef_set = true;
    match tag.ef_mode {
        0x1 => rain_set(blend, light, false),
        0x2 => {
            light.snow_count = 250.0 * blend;
            light.moya_mode = 2;
            light.moya_count = 100.0 * blend;
        }
        0x3 => {
            light.moya_mode = 0;
            light.moya_count = 100.0 * blend;
        }
        0x4 => {
            light.moya_mode = 1;
            light.moya_count = 100.0 * blend;
        }
        0x5 => {
            light.moya_mode = 2;
            light.moya_count = 100.0 * blend;
        }
        0x7 => {
            if light.thunder_mode == 0 {
                light.thunder_mode = 2;
            }
        }
        0x8 => {
            if light.thunder_mode == 0 {
                light.thunder_mode = 2;
            }
            rain_set(blend, light, false);
        }
        0xA => {
            light.moya_mode = 3;
            light.moya_count = 100.0 * blend;
        }
        0xB => {
            light.moya_mode = 4;
            light.moya_count = 100.0 * blend;
        }
        _ => {}
    }
}

fn clear_effect(tag: &Kytag00, light: &mut EnvLight) {
    match tag.ef_mode {
        0x1 => rain_cut(light, false),
        0x2 => {
            light.snow_count = 0.0;
            light.moya_count = 0.0;
        }
        0x3 | 0x4 | 0x5 | 0xA => light.moya_count = 0.0,
        0x6 => light.housi_count = 0.0,
        0x7 | 0x8 | 0x9 => {
            if light.thunder_mode == 2 {
                light.thunder_mode = 0;
            }
            rain_cut(light, false);
        }
        _ => {}
    }
}

pub fn spawn(world: &mut World, source: &crate::dzs::StageActor) -> Entity {
    let actor = Actor {
        name: source.name.clone(),
        parameters: source.parameters,
        pos: glam::Vec3::from_array(source.pos),
        angle: source.angle,
        set_id: source.set_id,
        room: source.room,
        ..Default::default()
    };
    let tag = Kytag00::spawn(
        source.parameters,
        source.angle,
        source.scale,
        world.resource::<Switches>(),
    );
    world.spawn((actor, tag)).id()
}

pub fn init(world: &mut World) {
    world.init_resource::<EnvLight>();
    world.init_resource::<CameraEye>();
}
