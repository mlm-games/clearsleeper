use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use glam::Vec3;

use super::actor::Actor;
use super::ba1::BA1_ACTOR;
use super::collision::Collision;
use super::events::{SimEvent, SimEvents};
use super::ground;
use super::health::Health;
use super::player::Player;
use super::walk;
use crate::assets::MsgArchives;

pub const NPC_FA1_ACTORS: [&str; 1] = ["Sfairy"];

pub fn is_npc_fa1(name: &str) -> bool {
    NPC_FA1_ACTORS.contains(&name)
}

// daNpc_Fa1_c::Type_e - d_a_npc_fa1.h:33-41
const TYPE_NORMAL: i8 = 0; // <- :34
const TYPE_TIMER: i8 = 1; // <- :35
const TYPE_LINK: i8 = 2; // <- :36
const TYPE_BABA: i8 = 3; // <- :37
const TYPE_AREA: i8 = 4; // <- :38
const TYPE_LINK_DOWN: i8 = 5; // <- :39
const TYPE_HOVER: i8 = 6; // <- :40

// daNpc_Fa1_c::Mode_e - d_a_npc_fa1.h:43-53
const MODE_NORMAL_MOVE: u8 = 0; // <- :44
const MODE_PLAYER_MOVE: u8 = 1; // <- :45
const MODE_ESCAPE_MOVE: u8 = 2; // <- :46
const MODE_BOTTLE_APPEAR_MOVE: u8 = 3; // <- :47
const MODE_BOTTLE_BABA_WAIT: u8 = 4; // <- :48
const MODE_BOTTLE_BABA_MOVE: u8 = 5; // <- :49
const MODE_BOTTLE_BABA_MOVE2: u8 = 6; // <- :50
const MODE_BIGELF_CHANGE: u8 = 7; // <- :51
const MODE_HOVER_MOVE: u8 = 8; // <- :52

// daNpc_Fa1_c::Normal_SubMode_e - d_a_npc_fa1.h:55-60
const NORMAL_STRAIGHT: u8 = 0; // <- :56
const NORMAL_TURN: u8 = 1; // <- :57
const NORMAL_AREAMOVE: u8 = 2; // <- :58
const NORMAL_AREAOUTMOVE: u8 = 3; // <- :59
// daNpc_Fa1_c::Bottle_SubMode_e - d_a_npc_fa1.h:62-66
const BOTTLE_UP1: u8 = 0; // <- :63
const BOTTLE_DOWN: u8 = 1; // <- :64
const BOTTLE_UP2: u8 = 2; // <- :65
// daNpc_Fa1_c::BottleBaba_SubMode_e - d_a_npc_fa1.h:68-71
const BABA_BABADOWN: u8 = 0; // <- :69
const BABA_UP2: u8 = 1; // <- :70

// daNpc_Fa1_HIO_c ctor prm - d_a_npc_fa1.cpp:130-154
const HIO_M08: f32 = 3.0; // <- :131
const HIO_M0C: f32 = 3.5; // <- :132
const HIO_M10: f32 = 40.0; // <- :133
const HIO_M14: f32 = 12.0; // <- :134
const HIO_M18: f32 = 8.0; // <- :135
const HIO_M1C: f32 = 200.0; // <- :136
const HIO_M20: f32 = 100.0; // <- :137
const HIO_M24: f32 = -50.0; // <- :138
const HIO_M28: f32 = 2.0; // <- :139
const HIO_M2C: f32 = 3.0; // <- :140
const HIO_M30: f32 = -2.5; // <- :141
const HIO_M34: f32 = 5.0; // <- :142
const HIO_M38: f32 = -3.0; // <- :143
const HIO_M3C: i16 = 4000; // <- :144
const HIO_M3E: i16 = 5000; // <- :145
const HIO_M40: i16 = 0x55; // <- :146
const HIO_M42: i16 = 0x2D; // <- :147
const HIO_M44: i16 = 500; // <- :148
const HIO_M46: i16 = 300; // <- :149
const HIO_M48: i16 = 0x2000; // <- :150
const HIO_M4A: i16 = 0x55; // <- :151
const HIO_M4C: i16 = 0x157C; // <- :152
const HIO_M4E: i16 = 400; // <- :153
// daNpc_Fa1_HIO2_c ctor prm - :113-123
const H2_M08: f32 = 20.0; // <- :114
const H2_M0C: f32 = 1.5; // <- :115
const H2_M10: f32 = -2.5; // <- :116
const H2_M14: f32 = 5.0; // <- :117
const H2_M18: f32 = -2.5; // <- :118
const H2_M1C: f32 = 5.0; // <- :119
const H2_M20: i16 = 0x3600; // <- :120
const H2_M22: i16 = 3; // <- :121
const H2_M24: i16 = 0xB4; // <- :122
// daNpc_Fa1_HIO3_c ctor prm - :100-106
const H3_M08: f32 = 1.0; // <- :101
const H3_M0C: f32 = 0.1; // <- :102
const H3_M10: f32 = 50.0; // <- :103
const H3_M14: f32 = 20.0; // <- :104
const H3_M18: i16 = 0x3600; // <- :105

// createInit - d_a_npc_fa1.cpp:247-298
const STATUS_ESCAPE: u8 = 0x01; // <- setStatus(1), :271
const AREA_SCALE: f32 = 500.0; // <- scale.x = 500.0f, :266
const TIMER_YAW_STEP: i32 = 0x100; // <- cLib_getRndValue<u16>(0, 0xFF) * 0x100, :273
const GROUND_OFFSET: f32 = 60.0; // <- local_18.y += 60.0f, :280
const HOME_GROUND_OFFSET: f32 = 50.0; // <- home.pos.y = mGroundY + 50.0f, :284

// init_normal_move - :392-403
const NORMAL_GRAVITY: f32 = 0.1; // <- gravity = 0.1f, :395
const NORMAL_MAX_FALL: f32 = -1.0; // <- maxFallSpeed = -1.0f, :396
const NORMAL_TIMER_RANGE: f32 = 60.0; // <- cLib_getRndValue<u16>(l_HIO.prm.m44, 60), :397
// init_straight / init_turn - :424, :449
const STRAIGHT_TIMER_BASE: f32 = 60.0; // <- cLib_getRndValue<u8>(60, 60)
const STRAIGHT_TIMER_RANGE: f32 = 60.0; // <- cM_rndF(60)
const STRAIGHT2_GRAVITY: f32 = 1.0; // <- init_straight2 gravity, :432
// straight / turn - :436-461
const TURN_SCALE: i16 = 0x20; // <- cLib_addCalcAngleS scale, :457, :483, :509
const TURN_MAX: i16 = 0x800; // <- maxStep, :457, :483, :509
const TURN_MIN: i16 = 1; // <- minStep, :457, :483, :509
const NECK_STEP: i16 = 4; // <- (r4 - r3) / 4, :881, :893
const NECK_MAX: i16 = 0x800; // <- clamp, :882, :894
const NECK_LIMIT: i16 = 0x4000; // <- clamp, :879, :890
// areaMove / areaOutMove - :480, :506
const AREA_FLIP_TIMER_BASE: f32 = 0xF as f32; // <- cLib_getRndValue(0xF, 0x14), :480, :506
const AREA_FLIP_TIMER_RANGE: f32 = 0x14 as f32; // <- cM_rndF(0x14)
const AREA_YAW_STEP: i16 = 0x2000; // <- m79A ? -0x2000 : 0x2000, :482, :508
// get_player_move - :526-553
const PLAY_SPEED_STEP: f32 = 0.05; // <- fVar1 += 0.05f, :528, :624
const PLAY_SPEED_MAX: f32 = 1.5; // <- fVar1 > 1.5f, :529, :625
const PLAYER_OFFSET_YAW: i16 = 0x4000; // <- current.angle.y + 0x4000, :541, :633
const PLAYER_RADIUS: f32 = 60.0; // <- setPlayerR(60.0f), :518, :652
const BOTTLE_UP1_YAW: i16 = -0x4000; // <- cM_atan2s(m768.x, m768.z) + -0x4000, :677
// bottle_appear_move - :650-653
const BOTTLE_R_STEP: f32 = 2.0; // <- mPlayerR += 2.0f, :650
// escape_move - :573-584
const NECK_ESCAPE_STEP: i16 = 0x800; // <- sVar2 += 0x800, :575
// bottle_baba_move / move2 - :773, :816
const BABA_Y_MOVE1: f32 = 150.0; // <- current.pos.y > m764->current.pos.y + 150.0f, :773
const BABA_Y_MOVE2: f32 = 180.0; // <- current.pos.y > m764->current.pos.y + 180.0f, :816
const BABA_OFFSET_X: f32 = 30.0; // <- current.pos.x = m764->current.pos.x + 30.0f, :740
// bigelf_change - :853-863
const BIGELF_SPEED: f32 = H3_M14; // <- speedF = l_HIO.m78.prm.m14, :853
const BIGELF_YAW: i16 = H3_M18; // <- sVar1 + l_HIO.m78.prm.m18, :863

// item_func_recover_faily - d_item.cpp:669-671
const RECOVER_FAIRY_LIFE: u8 = 40; // <- dComIfGp_setItemLifeCount(40.0f) quarter-units, d_item.cpp:671
const ITEM_RECOVER_FAIRY: u8 = 0x16; // <- dItemNo_RECOVER_FAIRY_e, d_item_data.h:32

// fopAcM_seStartCurrent ids - JAZelAudio_SE.h
const SE_OBJ_FAILY_GET: u32 = 0x695D; // <- :1877
const SE_OBJ_BOTTLE_FAILY: u32 = 0x6975; // <- :1899
const SE_OBJ_BA_FAILY_1: u32 = 0x6A2D; // <- :2064
const SE_OBJ_BA_FAILY_2: u32 = 0x6A2E; // <- :2065

// hover_move - :608
const HOVER_TURN_SCALE: i16 = 8; // <- cLib_addCalcAngleS scale, :608
const HOVER_TURN_MAX: i16 = 0x2000; // <- maxStep, :608
const HOVER_TURN_MIN: i16 = 0x400; // <- minStep, :608

// fopAc_CULLBOX_0_e - f_op_actor_mng.cpp:558
const CULL_MIN: [f32; 3] = [-40.0, 0.0, -40.0]; // <- fopAc_CULLBOX_0_e, f_op_actor_mng.cpp:558; prm :1030
const CULL_MAX: [f32; 3] = [40.0, 125.0, 40.0]; // <- fopAc_CULLBOX_0_e, f_op_actor_mng.cpp:558; prm :1030

const RND_MUL: [i32; 3] = [0xAB, 0xAC, 0xAA]; // <- cM_rnd, c_math.cpp:178-180
const RND_MOD: [i32; 3] = [0x763D, 0x7663, 0x7673]; // <- cM_rnd, c_math.cpp:178-180
const RND_SCALE: [f32; 3] = [30269.0, 30307.0, 30323.0]; // <- cM_rnd, c_math.cpp:181
const RND_SEED: i32 = 100; // <- cM_initRnd actor seeds, c_math.cpp:170-175

fn rnd(seed: &mut [i32; 3]) -> f32 {
    for lane in 0..3 {
        seed[lane] = (seed[lane] * RND_MUL[lane]) % RND_MOD[lane];
    }
    let sum = seed[0] as f32 / RND_SCALE[0]
        + seed[1] as f32 / RND_SCALE[1]
        + seed[2] as f32 / RND_SCALE[2];
    (sum % 1.0).abs()
}

/// `cLib_getRndValue` (`c_lib.h:80-82`) narrowed to the u8/u16 shapes used here.
fn rnd_u16(seed: &mut [i32; 3], min: f32, range: f32) -> u16 {
    (min + rnd(seed) * range) as u16
}

fn calc_timer(value: &mut u16) -> bool {
    if *value != 0 {
        *value -= 1;
        false
    } else {
        true
    }
}

fn calc_timer_u8(value: &mut u8) -> bool {
    if *value != 0 {
        *value -= 1;
        false
    } else {
        true
    }
}

fn abs_xz(v: Vec3) -> f32 {
    (v.x * v.x + v.z * v.z).sqrt()
}

fn target_angle_y(from: Vec3, to: Vec3) -> i16 {
    walk::to_s16((to.x - from.x).atan2(to.z - from.z))
}

fn atan2s(y: f32, x: f32) -> i16 {
    walk::to_s16(y.atan2(x))
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct NpcFa1 {
    pub entered: bool,
    pub deleted: bool,
    pub kind: i8,
    pub status: u8,
    pub mode: u8,
    pub sub_mode: u8,
    pub timer: u16,
    pub move_timer: u8,
    pub area_timer: i16,
    pub area_flip: i16,
    pub spin: i16,
    pub spin_max: i16,
    pub spin_step: i16,
    pub baba_max_fall: f32,
    pub baba_gravity: f32,
    pub neck_angle: i16,
    pub player_r: f32,
    pub ground_y: f32,
    pub speed_f: f32,
    pub speed_y: f32,
    pub gravity: f32,
    pub max_fall: f32,
    pub play_speed: f32,
    pub scale_x: f32,
    pub home_pos: Vec3,
    pub shape_angle: i16,
    pub offset: Vec3,
    pub seed: [i32; 3],
}

impl NpcFa1 {
    /// `mType = fopAcM_GetParam(this)` truncated to the `s8` field
    /// (`d_a_npc_fa1.h:794`), :248.
    pub fn spawn(parameters: u32) -> Self {
        Self {
            entered: false,
            deleted: false,
            kind: (parameters & 0xFF) as u8 as i8,
            status: 0,
            mode: MODE_NORMAL_MOVE,
            sub_mode: NORMAL_STRAIGHT,
            timer: 0,
            move_timer: 0,
            area_timer: 0,
            area_flip: 0,
            spin: 0,
            spin_max: 0,
            spin_step: 0,
            baba_max_fall: 0.0,
            baba_gravity: 0.0,
            neck_angle: 0,
            player_r: 0.0,
            ground_y: 0.0,
            speed_f: 0.0,
            speed_y: 0.0,
            gravity: 0.0,
            max_fall: 0.0,
            play_speed: 1.0,
            scale_x: 1.0,
            home_pos: Vec3::ZERO,
            shape_angle: 0,
            offset: Vec3::ZERO,
            seed: [RND_SEED; 3],
        }
    }

    pub fn cull(&self) -> ([f32; 3], [f32; 3]) {
        (CULL_MIN, CULL_MAX)
    }

    pub fn drawn(&self) -> bool {
        true
    }

    pub fn is_baba(&self) -> bool {
        self.kind == TYPE_BABA
    }

    pub fn is_link(&self) -> bool {
        self.kind == TYPE_LINK
    }

    pub fn is_link_down(&self) -> bool {
        self.kind == TYPE_LINK_DOWN
    }

    pub fn is_hover(&self) -> bool {
        self.kind == TYPE_HOVER
    }

    pub fn is_area(&self) -> bool {
        self.kind == TYPE_AREA
    }

    pub fn is_timer(&self) -> bool {
        self.kind == TYPE_TIMER
    }

    fn check_status(&self, bit: u8) -> bool {
        self.status & bit != 0
    }

    fn set_status(&mut self, status: u8) {
        self.status |= status;
    }

    /// `daNpc_Fa1_c::createInit` (`:247-298`), minus the `mStts`/`mCyl`
    /// registration, `setMtx`, the point light and the particle emitter.
    pub fn create_init(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.offset = Vec3::ZERO;
        self.set_status(0);
        if self.is_baba() {
            self.init_bottle_baba_wait();
        } else if self.is_link() || self.is_link_down() {
            self.init_bottle_appear_move(actor, ctx);
        } else if self.is_hover() {
            self.init_hover_move();
        } else {
            if self.is_area() {
                self.scale_x = AREA_SCALE;
            } else {
                self.scale_x = HIO_M10;
            }
            if self.is_timer() {
                self.set_status(STATUS_ESCAPE);
                let roll = rnd_u16(&mut self.seed, 0.0, 0xFF as f32) as i16;
                actor.angle[1] = actor.angle[1].wrapping_add(roll * TIMER_YAW_STEP as i16);
            } else {
                self.kind = TYPE_NORMAL;
            }
            self.init_normal_move();
        }
        let probe = Vec3::new(actor.pos.x, actor.pos.y + GROUND_OFFSET, actor.pos.z);
        let ground_y = ctx.surface(probe).unwrap_or(0.0);
        self.ground_y = ground_y;
        if !self.is_hover() && ground_y != f32::NEG_INFINITY {
            self.home_pos.y = ground_y + HOME_GROUND_OFFSET;
        } else {
            self.home_pos = actor.pos;
        }
        if self.is_link_down() {
            ctx.give_item(ITEM_RECOVER_FAIRY);
            ctx.heal(RECOVER_FAIRY_LIFE);
        }
        self.entered = true;
    }

    /// `daNpc_Fa1_c::position_move` (`:351-374`).
    fn position_move(&mut self, actor: &mut Actor, limit: f32, accel: f32) {
        let dy = actor.pos.y - self.home_pos.y;
        if dy > limit {
            self.max_fall = -accel;
        } else if dy < -limit {
            self.max_fall = accel;
        }
        if self.max_fall < 0.0 {
            self.speed_y -= self.gravity;
            if self.speed_y < self.max_fall {
                self.speed_y = self.max_fall;
            }
        } else {
            self.speed_y += self.gravity;
            if self.speed_y > self.max_fall {
                self.speed_y = self.max_fall;
            }
        }
        let rad = walk::from_s16(actor.angle[1]);
        let speed = Vec3::new(
            self.speed_f * rad.sin(),
            self.speed_y,
            self.speed_f * rad.cos(),
        );
        actor.pos += speed;
    }

    /// `daNpc_Fa1_c::BGCheck` (`:377-389`), minus the `GetTriPla` normal and the
    /// `tevStr` room/colour fetches.
    fn bg_check(&mut self, actor: &Actor, ctx: &Ctx<'_>) {
        if let Some(ground_y) = ctx.surface(actor.pos) {
            self.ground_y = ground_y;
        }
    }

    /// `daNpc_Fa1_c::findPlayer` (`:871-903`).
    fn find_player(&mut self, player_pos: Option<Vec3>, actor: &Actor) {
        let Some(player) = player_pos else { return };
        let delta = player - actor.pos;
        let mut r4 = atan2s(delta.x, delta.z).wrapping_sub(actor.angle[1]);
        let mut r3 = self.neck_angle;
        if r4 < 0 {
            if r4 < -NECK_LIMIT {
                r4 = -NECK_LIMIT;
            }
            let mut r0 = r4.wrapping_sub(r3) / NECK_STEP;
            if r0 < -NECK_MAX {
                r0 = -NECK_MAX;
            }
            r3 = r3.wrapping_add(r0);
            if r3 < r4 {
                r3 = r4;
            }
        } else {
            if r4 > NECK_LIMIT {
                r4 = NECK_LIMIT;
            }
            let mut r0 = r4.wrapping_sub(r3) / NECK_STEP;
            if r0 > NECK_MAX {
                r0 = NECK_MAX;
            }
            r3 = r3.wrapping_add(r0);
            if r3 > r4 {
                r3 = r4;
            }
        }
        self.neck_angle = r3;
    }

    /// `daNpc_Fa1_c::init_normal_move` (`:392-403`).
    fn init_normal_move(&mut self) {
        self.mode = MODE_NORMAL_MOVE;
        self.speed_f = HIO_M08;
        self.gravity = NORMAL_GRAVITY;
        self.max_fall = NORMAL_MAX_FALL;
        self.timer = rnd_u16(&mut self.seed, HIO_M44 as f32, NORMAL_TIMER_RANGE);
        if self.is_area() {
            self.init_area_move();
        } else {
            self.init_straight2();
        }
    }

    /// `daNpc_Fa1_c::init_straight` (`:422-426`).
    fn init_straight(&mut self) {
        self.sub_mode = NORMAL_STRAIGHT;
        self.move_timer = rnd_u16(&mut self.seed, STRAIGHT_TIMER_BASE, STRAIGHT_TIMER_RANGE) as u8;
        self.gravity = NORMAL_GRAVITY;
    }

    /// `daNpc_Fa1_c::init_straight2` (`:429-433`).
    fn init_straight2(&mut self) {
        self.sub_mode = NORMAL_STRAIGHT;
        self.move_timer = 0;
        self.gravity = STRAIGHT2_GRAVITY;
    }

    /// `daNpc_Fa1_c::straight` (`:436-444`).
    fn straight(&mut self, actor: &Actor) {
        self.scale_x = HIO_M10;
        if !calc_timer_u8(&mut self.move_timer) && abs_xz(actor.pos - self.home_pos) > self.scale_x
        {
            self.init_turn();
        }
    }

    /// `daNpc_Fa1_c::init_turn` (`:447-451`).
    fn init_turn(&mut self) {
        self.sub_mode = NORMAL_TURN;
        self.move_timer = rnd_u16(&mut self.seed, STRAIGHT_TIMER_BASE, STRAIGHT_TIMER_RANGE) as u8;
        self.gravity = NORMAL_GRAVITY;
    }

    /// `daNpc_Fa1_c::turn` (`:454-461`).
    fn turn(&mut self, actor: &mut Actor) {
        if calc_timer_u8(&mut self.move_timer) {
            let delta = actor.pos - self.home_pos;
            let want = atan2s(-delta.x, -delta.z);
            actor.angle[1] =
                walk::add_calc_angle_s(actor.angle[1], want, TURN_SCALE, TURN_MAX, TURN_MIN);
        } else {
            self.init_straight();
        }
    }

    /// `daNpc_Fa1_c::init_areaMove` (`:464-469`).
    fn init_area_move(&mut self) {
        self.sub_mode = NORMAL_AREAMOVE;
        self.gravity = STRAIGHT2_GRAVITY;
        self.area_timer = 0;
        self.area_flip = 0;
    }

    /// `daNpc_Fa1_c::areaMove` (`:472-485`).
    fn area_move(&mut self, actor: &mut Actor) {
        if abs_xz(actor.pos - self.home_pos) > self.scale_x {
            self.init_area_out_move();
            return;
        }
        let mut want = actor.angle[1];
        if !calc_timer_i16(&mut self.area_timer) {
            self.area_flip ^= 1;
            self.area_timer = (rnd_u16(&mut self.seed, AREA_FLIP_TIMER_BASE, AREA_FLIP_TIMER_RANGE)
                & 0xFF) as i16;
        }
        want = want.wrapping_add(if self.area_flip != 0 {
            -AREA_YAW_STEP
        } else {
            AREA_YAW_STEP
        });
        actor.angle[1] =
            walk::add_calc_angle_s(actor.angle[1], want, TURN_SCALE, TURN_MAX, TURN_MIN);
    }

    /// `daNpc_Fa1_c::init_areaOutMove` (`:488-493`).
    fn init_area_out_move(&mut self) {
        self.sub_mode = NORMAL_AREAOUTMOVE;
        self.gravity = NORMAL_GRAVITY;
        self.area_timer = 0;
        self.area_flip = 0;
    }

    /// `daNpc_Fa1_c::areaOutMove` (`:496-511`).
    fn area_out_move(&mut self, actor: &mut Actor) {
        if abs_xz(actor.pos - self.home_pos) < self.scale_x {
            self.init_area_move();
            return;
        }
        let mut want = target_angle_y(actor.pos, self.home_pos);
        if !calc_timer_i16(&mut self.area_timer) {
            self.area_flip ^= 1;
            self.area_timer = (rnd_u16(&mut self.seed, AREA_FLIP_TIMER_BASE, AREA_FLIP_TIMER_RANGE)
                & 0xFF) as i16;
        }
        want = want.wrapping_add(if self.area_flip != 0 {
            -AREA_YAW_STEP
        } else {
            AREA_YAW_STEP
        });
        actor.angle[1] =
            walk::add_calc_angle_s(actor.angle[1], want, TURN_SCALE, TURN_MAX, TURN_MIN);
    }

    /// `daNpc_Fa1_c::init_get_player_move` (`:514-523`).
    fn init_get_player_move(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.mode = MODE_PLAYER_MOVE;
        self.timer = HIO_M42 as u16;
        self.spin = HIO_M3C;
        self.player_r = PLAYER_RADIUS;
        actor.angle[1] = 0;
        self.neck_angle = 0;
        actor.pos.y = ctx.player_pos.map_or(actor.pos.y, |pos| pos.y);
        ctx.se(SE_OBJ_FAILY_GET);
    }

    /// `daNpc_Fa1_c::get_player_move` (`:526-553`).
    fn get_player_move(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.play_speed += PLAY_SPEED_STEP;
        if self.play_speed > PLAY_SPEED_MAX {
            self.play_speed = PLAY_SPEED_MAX;
        }
        let mut step = self.spin;
        actor.angle[1] = actor.angle[1].wrapping_sub(step);
        step = step.wrapping_add(HIO_M40);
        if step > HIO_M3E {
            step = HIO_M3E;
        }
        self.spin = step;
        let Some(player) = ctx.player_pos else {
            return;
        };
        let yaw = actor.angle[1].wrapping_add(PLAYER_OFFSET_YAW);
        let rad = walk::from_s16(yaw);
        self.offset.x = self.player_r * rad.sin();
        self.offset.z = self.player_r * rad.cos();
        self.offset.y += HIO_M0C;
        actor.pos = player + self.offset;
        self.bg_check(actor, ctx);
        if self.timer == 0 {
            ctx.give_item(ITEM_RECOVER_FAIRY);
            ctx.heal(RECOVER_FAIRY_LIFE);
            self.deleted = true;
        }
    }

    /// `daNpc_Fa1_c::init_escape_move` (`:556-561`).
    fn init_escape_move(&mut self) {
        self.mode = MODE_ESCAPE_MOVE;
        self.gravity = HIO_M18;
        self.max_fall = HIO_M14;
        self.timer = HIO_M46 as u16;
    }

    /// `daNpc_Fa1_c::escape_move` (`:564-593`).
    fn escape_move(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.speed_y += self.gravity;
        if self.speed_y > self.max_fall {
            self.speed_y = self.max_fall;
        }
        let rad = walk::from_s16(actor.angle[1]);
        let speed = Vec3::new(
            self.speed_f * rad.sin(),
            self.speed_y,
            self.speed_f * rad.cos(),
        );
        actor.pos += speed;
        self.bg_check(actor, ctx);
        let mut step = self.neck_angle;
        if step < 0 {
            step = step.wrapping_add(NECK_ESCAPE_STEP);
            if step > 0 {
                step = 0;
            }
        } else if step > 0 {
            step = step.wrapping_sub(NECK_ESCAPE_STEP);
            if step < 0 {
                step = 0;
            }
        }
        self.neck_angle = step;
        if self.timer == 0 {
            self.deleted = true;
        }
    }

    /// `daNpc_Fa1_c::init_hover_move` (`:596-601`).
    fn init_hover_move(&mut self) {
        self.mode = MODE_HOVER_MOVE;
        self.gravity = H3_M0C;
        self.max_fall = -H3_M08;
        self.speed_f = 0.0;
    }

    /// `daNpc_Fa1_c::hover_move` (`:604-609`).
    fn hover_move(&mut self, actor: &mut Actor, ctx: &Ctx<'_>) {
        self.position_move(actor, H3_M10, H3_M08);
        self.bg_check(actor, ctx);
        let Some(player) = ctx.player_pos else { return };
        let want = target_angle_y(actor.pos, player);
        actor.angle[1] = walk::add_calc_angle_s(
            actor.angle[1],
            want,
            HOVER_TURN_SCALE,
            HOVER_TURN_MAX,
            HOVER_TURN_MIN,
        );
    }

    /// `daNpc_Fa1_c::init_bottle_appear_move` (`:612-618`).
    fn init_bottle_appear_move(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.mode = MODE_BOTTLE_APPEAR_MOVE;
        self.init_up1(actor, ctx);
        if self.is_link_down() {
            ctx.give_item(ITEM_RECOVER_FAIRY);
            ctx.heal(RECOVER_FAIRY_LIFE);
        }
    }

    /// `daNpc_Fa1_c::bottle_appear_move` (`:621-662`).
    fn bottle_appear_move(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.play_speed += PLAY_SPEED_STEP;
        if self.play_speed > PLAY_SPEED_MAX {
            self.play_speed = PLAY_SPEED_MAX;
        }
        actor.angle[1] = actor.angle[1].wrapping_sub(self.spin);
        self.spin = self.spin.wrapping_add(self.spin_step);
        if self.spin > self.spin_max {
            self.spin = self.spin_max;
        }
        let Some(player) = ctx.player_pos else { return };
        let yaw = actor.angle[1].wrapping_add(PLAYER_OFFSET_YAW);
        let rad = walk::from_s16(yaw);
        self.offset.x = self.player_r * rad.sin();
        self.offset.z = self.player_r * rad.cos();
        self.offset.y += self.speed_y;
        actor.pos = player + self.offset;
        if self.max_fall < 0.0 {
            self.speed_y -= self.gravity;
            if self.speed_y < self.max_fall {
                self.speed_y = self.max_fall;
            }
        } else {
            self.speed_y += self.gravity;
            if self.speed_y > self.max_fall {
                self.speed_y = self.max_fall;
            }
        }
        self.player_r += BOTTLE_R_STEP;
        if self.player_r > PLAYER_RADIUS {
            self.player_r = PLAYER_RADIUS;
        }
        self.bg_check(actor, ctx);
        match self.sub_mode {
            BOTTLE_UP1 => self.up1(),
            BOTTLE_DOWN => self.down(ctx),
            _ => self.up2(),
        }
        if self.timer == 0 {
            if !self.is_link_down() {
                ctx.give_item(ITEM_RECOVER_FAIRY);
                ctx.heal(RECOVER_FAIRY_LIFE);
            }
            self.deleted = true;
        }
    }

    /// `daNpc_Fa1_c::init_up1` (`:665-678`).
    fn init_up1(&mut self, actor: &mut Actor, ctx: &Ctx<'_>) {
        self.sub_mode = BOTTLE_UP1;
        self.timer = HIO_M4A as u16;
        self.max_fall = HIO_M30;
        self.speed_y = HIO_M28;
        self.gravity = HIO_M34;
        self.spin = 0;
        self.spin_max = 0;
        self.spin_step = 0;
        self.offset = ctx.actor_pos - ctx.player_pos.unwrap_or(ctx.actor_pos);
        self.player_r = abs_xz(self.offset);
        let yaw = atan2s(self.offset.x, self.offset.z).wrapping_add(BOTTLE_UP1_YAW);
        actor.angle[1] = yaw;
    }

    /// `daNpc_Fa1_c::up1` (`:681-685`).
    fn up1(&mut self) {
        if self.speed_y < 0.0 {
            self.init_down();
        }
    }

    /// `daNpc_Fa1_c::init_down` (`:688-692`).
    fn init_down(&mut self) {
        self.sub_mode = BOTTLE_DOWN;
        self.spin_max = HIO_M4C;
        self.spin_step = HIO_M4E;
    }

    /// `daNpc_Fa1_c::down` (`:695-698`).
    fn down(&mut self, ctx: &mut Ctx<'_>) {
        if self.offset.y < HIO_M38 {
            self.init_up2(ctx);
        }
    }

    /// `daNpc_Fa1_c::init_up2` (`:702-706`).
    fn init_up2(&mut self, ctx: &mut Ctx<'_>) {
        self.sub_mode = BOTTLE_UP2;
        self.max_fall = HIO_M2C;
        ctx.se(SE_OBJ_BOTTLE_FAILY);
    }

    /// `daNpc_Fa1_c::up2` (`:709-711`).
    fn up2(&mut self) {}

    /// `daNpc_Fa1_c::init_bottle_baba_wait` (`:714-721`).
    fn init_bottle_baba_wait(&mut self) {
        self.mode = MODE_BOTTLE_BABA_WAIT;
        self.speed_y = 0.0;
        self.speed_f = 0.0;
        self.max_fall = 0.0;
        self.gravity = 0.0;
        self.timer = H2_M24 as u16;
    }

    /// `daNpc_Fa1_c::bottle_baba_wait` (`:724-728`).
    fn bottle_baba_wait(&mut self, ctx: &mut Ctx<'_>) {
        if self.timer == 0 {
            self.init_bottle_baba_move2(ctx);
        }
    }

    /// `daNpc_Fa1_c::init_bottle_baba_move` (`:731-744`).
    fn init_bottle_baba_move(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.mode = MODE_BOTTLE_BABA_MOVE;
        self.timer = H2_M22 as u16;
        self.speed_f = H2_M08;
        self.area_timer = 1;
        self.baba_max_fall = H2_M10;
        self.baba_gravity = H2_M0C;
        if let Some(target) = ctx.ba1 {
            actor.pos.x = target.x + BABA_OFFSET_X;
            actor.pos.z = target.z;
        }
        self.init_baba_down(ctx);
    }

    /// `daNpc_Fa1_c::bottle_baba_move` (`:747-777`).
    fn bottle_baba_move(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        let Some(target) = ctx.ba1 else {
            self.init_bottle_baba_move(actor, ctx);
            self.bg_check(actor, ctx);
            return;
        };
        self.speed_f = H2_M08;
        let delta = target - actor.pos;
        let yaw = atan2s(delta.x, delta.z).wrapping_add(H2_M20);
        actor.angle[1] = yaw;
        let rad = walk::from_s16(yaw);
        let speed = Vec3::new(self.speed_f * rad.sin(), 0.0, self.speed_f * rad.cos());
        let dy = self.apply_y();
        actor.pos += Vec3::new(speed.x, dy, speed.z);
        self.bg_check(actor, ctx);
        let sub = self.sub_mode;
        if sub == BABA_BABADOWN {
            self.baba_down(actor.pos.y, ctx);
        } else {
            self.init_baba_up(ctx);
        }
        if sub == BABA_UP2 && actor.pos.y > target.y + BABA_Y_MOVE1 {
            self.deleted = true;
        }
    }

    /// `daNpc_Fa1_c::init_bottle_baba_move2` (`:780-788`).
    fn init_bottle_baba_move2(&mut self, ctx: &mut Ctx<'_>) {
        self.mode = MODE_BOTTLE_BABA_MOVE2;
        self.speed_f = H2_M08;
        self.area_timer = 0;
        self.baba_max_fall = H2_M18;
        self.baba_gravity = H2_M14;
        self.init_baba_down(ctx);
    }

    /// `daNpc_Fa1_c::bottle_baba_move2` (`:791-820`).
    fn bottle_baba_move2(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        let Some(player) = ctx.player_pos else {
            self.init_bottle_baba_move2(ctx);
            self.bg_check(actor, ctx);
            return;
        };
        self.speed_f = H2_M08;
        let delta = player - actor.pos;
        let yaw = atan2s(delta.x, delta.z).wrapping_add(H2_M20);
        actor.angle[1] = yaw;
        let rad = walk::from_s16(yaw);
        let speed = Vec3::new(self.speed_f * rad.sin(), 0.0, self.speed_f * rad.cos());
        let dy = self.apply_y();
        actor.pos += Vec3::new(speed.x, dy, speed.z);
        self.bg_check(actor, ctx);
        if self.sub_mode == BABA_BABADOWN {
            self.baba_down(actor.pos.y, ctx);
        } else {
            self.init_baba_up(ctx);
        }
        if actor.pos.y > player.y + BABA_Y_MOVE2 {
            self.init_bottle_baba_move(actor, ctx);
        }
    }

    fn apply_y(&mut self) -> f32 {
        self.speed_y = 0.0;
        if self.max_fall < 0.0 {
            self.speed_y -= self.gravity;
            if self.speed_y < self.max_fall {
                self.speed_y = self.max_fall;
            }
        } else {
            self.speed_y += self.gravity;
            if self.speed_y > self.max_fall {
                self.speed_y = self.max_fall;
            }
        }
        self.speed_y
    }

    /// `daNpc_Fa1_c::init_baba_down` (`:823-831`).
    fn init_baba_down(&mut self, ctx: &mut Ctx<'_>) {
        self.sub_mode = BABA_BABADOWN;
        self.speed_y = 0.0;
        self.max_fall = self.baba_max_fall;
        self.gravity = H2_M1C;
        if self.mode == MODE_BOTTLE_BABA_MOVE {
            ctx.se(SE_OBJ_BA_FAILY_2);
        }
    }

    /// `daNpc_Fa1_c::baba_down` (`:834-838`).
    fn baba_down(&mut self, y: f32, ctx: &mut Ctx<'_>) {
        if let Some(player) = ctx.player_pos
            && y < player.y
        {
            self.init_baba_up(ctx);
        }
    }

    /// `daNpc_Fa1_c::init_baba_up` (`:841-847`).
    fn init_baba_up(&mut self, ctx: &mut Ctx<'_>) {
        self.sub_mode = BABA_UP2;
        self.max_fall = self.baba_gravity;
        if self.mode == MODE_BOTTLE_BABA_MOVE2 {
            ctx.se(SE_OBJ_BA_FAILY_1);
        }
    }

    /// `daNpc_Fa1_c::init_bigelf_change` (`:850-857`).
    fn init_bigelf_change(&mut self, actor: &mut Actor) {
        self.mode = MODE_BIGELF_CHANGE;
        self.speed_f = BIGELF_SPEED;
        let rad = walk::from_s16(actor.angle[1]);
        actor.pos += Vec3::new(self.speed_f * rad.sin(), 0.0, self.speed_f * rad.cos());
    }

    /// `daNpc_Fa1_c::bigelf_change` (`:860-868`).
    fn bigelf_change(&mut self, actor: &mut Actor, ctx: &Ctx<'_>) {
        let delta = self.home_pos - actor.pos;
        let yaw = atan2s(delta.x, delta.z).wrapping_add(BIGELF_YAW);
        actor.angle[1] = yaw;
        let rad = walk::from_s16(yaw);
        actor.pos += Vec3::new(self.speed_f * rad.sin(), 0.0, self.speed_f * rad.cos());
        self.bg_check(actor, ctx);
    }

    /// `daNpc_Fa1_c::normal_move` (`:406-419`), minus `mCyl.ChkCoHit()` (`:414`).
    fn normal_move(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        self.speed_f = HIO_M08;
        match self.sub_mode {
            NORMAL_STRAIGHT => self.straight(actor),
            NORMAL_TURN => self.turn(actor),
            NORMAL_AREAMOVE => self.area_move(actor),
            _ => self.area_out_move(actor),
        }
        self.position_move(actor, 10.0, 1.0);
        self.bg_check(actor, ctx);
        self.find_player(ctx.player_pos, actor);
        if !self.check_bin_catch() && self.check_status(STATUS_ESCAPE) && self.timer == 0 {
            self.init_escape_move();
        }
    }

    /// `daNpc_Fa1_c::checkBinCatch` (`:333-348`) minus the catch request; the
    /// `eventInfo.checkCommandCatch()` and `dComIfGp_att_CatchRequest` calls have
    /// no sim input, so this is always false.
    fn check_bin_catch(&self) -> bool {
        false
    }

    /// `daNpc_Fa1_c::_execute` (`:316-330`), minus the renderer, `setMtx`, the
    /// point light, the emitter rate and `dComIfGp_att_LookRequest`.
    pub fn execute(&mut self, actor: &mut Actor, ctx: &mut Ctx<'_>) {
        calc_timer(&mut self.timer);
        match self.mode {
            MODE_NORMAL_MOVE => self.normal_move(actor, ctx),
            MODE_PLAYER_MOVE => self.get_player_move(actor, ctx),
            MODE_ESCAPE_MOVE => self.escape_move(actor, ctx),
            MODE_BOTTLE_APPEAR_MOVE => self.bottle_appear_move(actor, ctx),
            MODE_BOTTLE_BABA_WAIT => self.bottle_baba_wait(ctx),
            MODE_BOTTLE_BABA_MOVE => self.bottle_baba_move(actor, ctx),
            MODE_BOTTLE_BABA_MOVE2 => self.bottle_baba_move2(actor, ctx),
            MODE_BIGELF_CHANGE => self.bigelf_change(actor, ctx),
            _ => self.hover_move(actor, ctx),
        }
        self.shape_angle = actor.angle[1];
    }
}

fn calc_timer_i16(value: &mut i16) -> bool {
    if *value != 0 {
        *value -= 1;
        false
    } else {
        true
    }
}

pub struct Ctx<'a> {
    pub player_pos: Option<Vec3>,
    pub actor_pos: Vec3,
    pub actor_angle: i16,
    pub ba1: Option<Vec3>,
    pub collision: Option<&'a Collision>,
    pub health: &'a mut Health,
    pub events: &'a mut SimEvents,
    pub equipped: &'a mut super::inventory::Equipped,
    pub got: &'a mut super::inventory::Got,
}

impl Ctx<'_> {
    fn surface(&self, pos: Vec3) -> Option<f32> {
        ground::surface(self.collision, pos)
    }

    fn se(&mut self, id: u32) {
        self.events.list.push(SimEvent::Se(id));
    }

    fn heal(&mut self, amount: u8) {
        self.health.heal(amount);
    }

    /// `execItemGet(dItemNo_RECOVER_FAIRY_e)` (`d_item.cpp:669-671`): the item is
    /// recorded in `Got`/`Equipped` and its effect is the life top-up.
    fn give_item(&mut self, item: u8) {
        if let Some(slot) = super::inventory::slot_of(item) {
            self.got.gain(slot, 0);
        }
        self.equipped.collect(item);
    }
}

#[derive(SystemParam)]
pub struct Env<'w> {
    pub collision: Option<Res<'w, Collision>>,
    pub health: ResMut<'w, Health>,
    pub events: ResMut<'w, SimEvents>,
    pub equipped: ResMut<'w, super::inventory::Equipped>,
    pub got: ResMut<'w, super::inventory::Got>,
    pub archive: Option<Res<'w, MsgArchives>>,
}

#[derive(SystemParam)]
pub struct Msg<'w, 's> {
    pub players: Query<'w, 's, &'static Player>,
}

pub fn control(
    mut world: Env,
    msg: Msg,
    others: Query<(Entity, &'static Actor), Without<NpcFa1>>,
    mut actors: Query<(Entity, &mut Actor, &mut NpcFa1)>,
    mut commands: Commands,
) {
    let player = msg.players.iter().next().copied();
    let ba1 = others
        .iter()
        .find(|(_, actor)| actor.name == BA1_ACTOR)
        .map(|(_, actor)| actor.pos);
    let collision = world.collision.as_deref();
    for (entity, mut actor, mut fa1) in &mut actors {
        let mut ctx = Ctx {
            player_pos: player.map(|link| link.pos),
            actor_pos: actor.pos,
            actor_angle: actor.angle[1],
            ba1,
            collision,
            health: &mut world.health,
            events: &mut world.events,
            equipped: &mut world.equipped,
            got: &mut world.got,
        };
        if !fa1.entered {
            fa1.create_init(&mut actor, &mut ctx);
        }
        ctx.actor_pos = actor.pos;
        ctx.actor_angle = actor.angle[1];
        fa1.execute(&mut actor, &mut ctx);
        actor.pos = ctx.actor_pos;
        actor.angle[1] = ctx.actor_angle;
        if fa1.deleted {
            let _ = commands.entity(entity).despawn();
        }
    }
}
