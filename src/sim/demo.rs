//! The demo player - `d_demo.cpp` / `d_demo.h`, the JStudio layer a stage event's `PACKAGE`
//! staff mounts through `dComIfGp_demo_create` (`d_event_data.cpp:796-803`).
//!
//! Three pieces live here, in the reference's own shape:
//!
//! * [`DemoManager`] is `dDemo_manager_c` (`d_demo.cpp:673-769`): a mode (`0` not mounted, `1`
//!   playing, `2` finished), a frame count, a no-message frame count, and the object table.
//! * [`DemoObject`] and the five JStage objects - [`DemoActor`], [`DemoCamera`], [`DemoAmbient`],
//!   [`DemoLight`], [`DemoFog`] - are `dDemo_object_c`'s registries and the fields the timeline
//!   writes (`d_demo.h:52-313`), including `getP_BtpData` / `getP_BrkData` / `getP_BtkData`, the
//!   three parameter decoders the fork matched (`d_demo.cpp:54-123`).
//! * [`DemoStage`] is `dDemo_system_c::JSGFindObject` (`d_demo.cpp:642-671`): the lookup that turns
//!   a timeline's object name into a demo actor - fast-creating the `d_act0`..`d_act31` shells when
//!   nothing in the stage answers to that name, exactly the 32 spellings `demo00::DEMO00_ACTORS`
//!   carries - and the camera, ambient, light and fog tables.
//!
//! The reader itself - the STB parse and the `stb::TControl` timeline player - lives in
//! [`super::jstudio`] and is installed by [`init`]. `dDemo_manager_c::create` parses the mounted
//! file into a `JStudio::TControl` timeline and forwards it zero frames (`d_demo.cpp:719-740`);
//! `update` forwards one frame a step (`:756-769`). The reader parses without a stage, so the
//! create-time `forward(0)` runs at the mount site through [`DemoManager::start`], and every later
//! step is [`DemoManager::control`].
//!
//! # The gaps, and why they are typed
//!
//! The reader drives the actor, camera, ambient, light, fog, message, sound and particle
//! adaptors. What is left over is what the port has no subsystem for: the particle ids the
//! JPA bank cannot confirm, and the function-value bank lookups that answer with the NULL retail
//! dereferences. Every such command reaches the shell as a [`DemoCommandGap`] through
//! [`DemoManager::take_gaps`], and a command whose value is unrepresentable leaves its model
//! unchanged rather than writing a stand-in. A file the reader cannot parse at all - or a build
//! with no reader installed - leaves `mCurFile == NULL` and `mMode == 0`, which is where retail's
//! own failed `parse_next` leaves them (`:723-727`); it reports [`DemoGap::NoReader`] or
//! [`DemoGap::UnreadableFile`] respectively. An ID the mount cannot resolve fails the mount the
//! same way retail's parse does, reported as [`DemoGap::NoObject`] or
//! [`DemoGap::UnreadableName`].
//!
//! The node lookups are answered from [`DemoActor::set_model`]'s table, which is the write
//! `daDemo00_c::actStandby` makes out of the room's demo archive (`d_a_demo00.cpp:296-313`). That
//! archive is [`RoomDemoArchive`]: the room set names it `Demo%02d` out of its `LBNK` table
//! (`d_s_room.cpp:220-231`) and [`RoomDemoArchive::model`] resolves the staged shape id out of it.
//! A cut whose actor has no model - no archive named, or no object under the id - reports
//! [`DemoCommandGap::NodeName`] / [`DemoCommandGap::NodeTransform`] against the object the lookup
//! ran in, and the resolution's own miss is a [`DemoGap`].
//!
//! The morph half of `createHeap` is wired: the staged `mBckID` becomes the model's
//! [`Morph`], `actStandby`'s `setAnmFrameMax` reads its frame count, and
//! [`DemoNodeTable::set_frame`] poses the joint matrices at the frame `actPerformance` takes off
//! the demo actor (`d_a_demo00.cpp:412-452`), so a cut's model moves where its bind pose stood
//! still. The material half is wired too: the node table carries the model's `MAT3` material
//! table, and the btp, btk and brk keys `createHeap` inits (`:173-221`) register on the slots
//! their entries name and write what their cursors sample into them, the way
//! `J3DMaterialAnm::calc` does (`J3DMaterialAnm.cpp:51-76`). A staged key whose object the port
//! cannot read as that shape is named as [`DemoGap::UnreadablePass`] instead of being dropped;
//! one the archive does not carry at all is retail's own early return, which leaves the cut
//! without that pass.

use std::collections::HashMap;

use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};

use crate::archive::Archive;
use crate::j3d::bck::Morph;

use super::actor::Actor;
use super::demo00;
use super::door::StayRoom;
use super::events::SimEvents;
use super::jpa::Jpa;
use super::msg::{MsgBox, MsgTable};
use super::stage_data::ActorTable;

/// `dDemo_manager_c::mMode`'s three values: the constructor and `remove` write `0`
/// (`d_demo.cpp:703`, `:752`), `create` writes `1` (`:738`), and `update` writes `2` when the
/// timeline stops forwarding (`:766`).
pub const MODE_NONE: i32 = 0;
pub const MODE_PLAY: i32 = 1;
pub const MODE_END: i32 = 2;

/// `mControl->setSecondPerFrame(1/30.0f)` - `d_demo.cpp:696`, written there as the float its own
/// comment calls out.
pub const SECONDS_PER_FRAME: f32 = 1.0 / 30.0;

/// `mpActors[32]` - `d_demo.h:308`.
pub const ACTOR_MAX: usize = 32;
/// `mpLight[8]` - `d_demo.h:311`.
pub const LIGHT_MAX: usize = 8;

/// The prefix `JSGFindObject` matches before it fast-creates a demo shell - `d_demo.cpp:650`,
/// `sizeof("d_act")-1`.
pub const DEMO_ACTOR_PREFIX: &str = "d_act";

/// `JSGActor.h:8-16`'s data ids, the arms `getP_BtpData` and friends switch on.
pub const PRM_ID_UNK_1: u32 = 1;
pub const PRM_ID_UNK_2: u32 = 2;
pub const PRM_ID_UNK_4: u32 = 4;
pub const PRM_ID_UNK_5: u32 = 5;
pub const PRM_ID_UNK_6: u32 = 6;

/// `dDemo_actor_c::Enable_e` - `d_demo.h:54-64`.
pub const ACTOR_ENABLE_UNK: u16 = 1 << 0;
pub const ACTOR_ENABLE_TRANS: u16 = 1 << 1;
pub const ACTOR_ENABLE_SCALE: u16 = 1 << 2;
pub const ACTOR_ENABLE_ROTATE: u16 = 1 << 3;
pub const ACTOR_ENABLE_SHAPE: u16 = 1 << 4;
pub const ACTOR_ENABLE_ANM: u16 = 1 << 5;
pub const ACTOR_ENABLE_ANM_FRAME: u16 = 1 << 6;
pub const ACTOR_ENABLE_TEX_ANM: u16 = 1 << 7;
pub const ACTOR_ENABLE_TEX_ANM_FRAME: u16 = 1 << 8;

/// `dDemo_camera_c::Enable_e` - `d_demo.h:157-166`.
pub const CAMERA_ENABLE_PROJ_NEAR: u8 = 1 << 0;
pub const CAMERA_ENABLE_PROJ_FAR: u8 = 1 << 1;
pub const CAMERA_ENABLE_PROJ_FOVY: u8 = 1 << 2;
pub const CAMERA_ENABLE_PROJ_ASPECT: u8 = 1 << 3;
pub const CAMERA_ENABLE_VIEW_POS: u8 = 1 << 4;
pub const CAMERA_ENABLE_VIEW_UP_VEC: u8 = 1 << 5;
pub const CAMERA_ENABLE_VIEW_TARG_POS: u8 = 1 << 6;
pub const CAMERA_ENABLE_VIEW_ROLL: u8 = 1 << 7;

/// `dDemo_ambient_c::Enable_e` - `d_demo.h:210-212`.
pub const AMBIENT_ENABLE_COLOR: u8 = 1 << 0;

/// `dDemo_light_c::Enable_e` - `d_demo.h:227-233`.
pub const LIGHT_ENABLE_LIGHT_TYPE: u8 = 1 << 0;
pub const LIGHT_ENABLE_POSITION: u8 = 1 << 1;
pub const LIGHT_ENABLE_COLOR: u8 = 1 << 2;
pub const LIGHT_ENABLE_DIST_ATTEN: u8 = 1 << 3;
pub const LIGHT_ENABLE_ANGLE_ATTEN: u8 = 1 << 4;
pub const LIGHT_ENABLE_DIRECTION: u8 = 1 << 5;

/// `dDemo_fog_c::Enable_e` - `d_demo.h:262-267`.
pub const FOG_ENABLE_FOG_FN: u8 = 1 << 0;
pub const FOG_ENABLE_START_Z: u8 = 1 << 1;
pub const FOG_ENABLE_END_Z: u8 = 1 << 2;
pub const FOG_ENABLE_COLOR: u8 = 1 << 3;

/// `DEG2S` - `c_angle.h:15`: `(s16)(x * (0x8000 / 180.0f))`, the truncating cast included.
/// `JSGSetRotation` runs its degree triple through it (`d_demo.cpp:285-287`).
fn deg2s(deg: f32) -> i16 {
    (deg * (32768.0 / 180.0)) as i16
}

/// `S2DEG` - `c_angle.h:16`: `x * (180.0f / 0x8000)`, the same multiply `cM_sht2d` (`c_math.h:55`)
/// is. `JSGGetRotation` (`d_demo.h:103-105`) and `JSGGetViewRoll` (`d_demo.cpp:450-452`) read
/// back through it.
fn sh2d(value: f32) -> f32 {
    value * (180.0 / 32768.0)
}

/// `dDemo_prm_c` - `d_demo.h:42-50` - plus the size `JSGSetData` was handed, which is the
/// `field_0x54` `getPrm_Morf` gates on (`d_demo.cpp:139`, `:145`, `:151`, `:156`, `:161`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Prm {
    pub id: u32,
    /// The record `JSGSetData`'s pointer aimed at. The reference holds a raw pointer into the
    /// timeline's parameter pool; the port copies the bytes.
    pub data: Vec<u8>,
    pub size: u32,
}

impl Prm {
    fn bytes(&self, offset: usize, len: usize) -> Option<&[u8]> {
        self.data.get(offset..offset.checked_add(len)?)
    }

    /// One byte of the record, the shape `TValueIterator_raw<u8>` reads in `d_a_demo00.cpp:706-713`.
    pub fn u8_at(&self, offset: usize) -> Option<u8> {
        self.bytes(offset, 1).map(|bytes| bytes[0])
    }

    /// `*(const s16*)(prm + offset)`, the unaligned halfword read `d_demo.cpp:68-69` and `:112` do.
    pub fn s16_at(&self, offset: usize) -> Option<i16> {
        let bytes = self.bytes(offset, 2)?;
        Some(i16::from_be_bytes([bytes[0], bytes[1]]))
    }

    /// `*(const u32*)(prm + offset)` - `d_demo.cpp:70-72`, `:94`, `:114`.
    pub fn u32_at(&self, offset: usize) -> Option<u32> {
        let bytes = self.bytes(offset, 4)?;
        Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}

/// Which archive a demo object lookup was pointed at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemoArchive {
    /// The name the caller handed in: an actor's own archive, or the demo file's stem.
    Named(String),
    /// `dStage_roomControl_c::getDemoArcName()`, the room's demo archive (`d_demo.cpp:80`, `:101`,
    /// `:121`). [`RoomDemoArchive`] is where this port holds it; whichever layer fills the object
    /// store resolves this.
    Room,
    /// The reference's `arcName` local is never written on this path: `ENABLE_TEX_ANM` was set but
    /// the id carried no `0x10000` room-demo bit, so `dComIfG_getObjectIDRes` reads an uninitialized
    /// stack slot. There is no faithful value to pick.
    Uninitialized,
}

/// One `dComIfG_getObjectIDRes` result (`d_demo.cpp:81`): the archive and the object id. The btp
/// animation behind the id is not decoded - the port's demo model carries joints and transforms,
/// not the material a tex-pattern keys - so the id is what comes back and `frame_max` stays `None`
/// where the reference reads `anm->getFrameMax()` (`d_demo.cpp:83`).
#[derive(Debug, Clone, PartialEq)]
pub struct AnmRef {
    pub archive: DemoArchive,
    pub id: u16,
    pub frame_max: Option<f32>,
}

/// Everything the demo layer can name but not do. Every variant is a place the reference reads
/// something this port does not hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemoGap {
    /// No [`DemoReaders`] reader is installed, so a mounted file cannot become a timeline
    /// (`d_demo.cpp:720-727`).
    NoReader,
    /// The `PACKAGE` cut carries no `FileName` key, so nothing named a file to look up
    /// (`d_event_data.cpp:786`).
    NoFileName,
    /// The stage archive has no entry under the name the cut gave. `specialProcPackage` looks the
    /// name up in the room's demo archive first (`d_event_data.cpp:796-799`) and falls back to the
    /// stage archive (`:800-801`); the mount site reads both, so a name neither carries is this
    /// gap.
    NoDemoData { name: String },
    /// `mNumActor >= ARRAY_SIZE(mpActors)` - `d_demo.cpp:551-552`, `d_demo.h:308`.
    ActorTableFull,
    /// `dStage_searchName` answered NULL, so neither the search nor the fast-create could run
    /// (`f_op_actor_mng.cpp:182-184`, `:4081-4082`).
    NoNameInfo { name: String },
    /// `checkEnable(ENABLE_UNK_e)` was clear - `d_demo.cpp:63-64`, `:89-90`, `:107-108`.
    NoParameterRecord,
    /// The record's id is not one of the arms that carry this field (`d_demo.cpp:73`, `:95`,
    /// `:115`).
    UnknownParameter { id: u32 },
    /// The id equals the cached one, which the reference reports with the same NULL its callers
    /// read as "nothing changed" (`d_demo.cpp:76-77`, `:97-98`, `:117-118`).
    Unchanged,
    /// A reader is installed and answered `None`, its own contract for a file it cannot read
    /// (`d_demo.cpp:723-727`, where `parse_next` failing leaves `mMode == 0`).
    UnreadableFile,
    /// `JStudio_JStage::TCreateObject::find` answered NULL for a block's ID
    /// (`JStudio_JStage/control.cpp:54-73`): `JSGFindObject` found no stage object of the block's
    /// type, so `TCreateObject::create` answered false, `JStudio::TFactory::create` fell through
    /// to `stb::TFactory::create`'s NULL (`jstudio-control.cpp:54-65`), and
    /// `stb::TParse::parseBlock_object` failed the whole parse (`stb.cpp:403-421`). The mount is
    /// what fails here, at the first step, because the port's reader has no stage while parsing.
    NoObject { kind: &'static str, name: String },
    /// A block ID the port cannot read as a C string, which is the same failed lookup: the
    /// reference hands `get_ID()` to `JSGFindObject` as `(const char*)` (`d_demo.cpp:648`) and
    /// its `strcmp` runs to the next NUL in the file buffer, of which the port found none.
    UnreadableName { at: usize, name: Vec<u8> },
    /// The room's demo archive `name` - the `Demo%02d` `d_s_room.cpp:229` names and `:230`
    /// mounts - could not be mounted, so the cut model `daDemo00_c::createHeap` reads out of it
    /// (`d_a_demo00.cpp:169`) has no source. The empty name is a stage whose room sets named no
    /// archive at all: the empty `dStage_roomControl_c::getDemoArcName()` retail looks up.
    NoDemoArchive { name: String },
    /// The room's demo archive is mounted and `dComIfG_getObjectIDRes(archive, id)`
    /// (`d_com_inf_game.h:4122-4124`) answers nothing, so the cut model `daDemo00_c::createHeap`
    /// reads the staged shape id out of it (`d_a_demo00.cpp:169`) has no source.
    NoDemoModel { archive: String, id: u16 },
    /// The same lookup for the staged `mBckID` (`d_a_demo00.cpp:230`): the object the morph
    /// `daDemo00_c::createHeap` builds (`:232-249`) reads its keys from is not in the archive, or
    /// is not a `J3D1bck1` this port can read. The reference asserts the pointer
    /// (`:231`) before handing it to `mDoExt_McaMorf`.
    NoDemoAnm { archive: String, id: u16 },
    /// `daDemo00_c::createHeap` built an animation whose pass this port cannot apply to the cut
    /// model's material table: the archived object behind the staged `mBtpID`
    /// (`d_a_demo00.cpp:173-187`), `mBtkID` (`:189-207`) or `mBrkID` (`:209-221`) is present but
    /// not a shape this port reads as that key. The reference hands whatever the object holds to
    /// `mDoExt_*Anm::init`, so its cut keeps animating where this port's material slots keep
    /// what the last pass left. A *missing* object is not this gap: retail's own NULL check
    /// returns early with the pass skipped.
    UnreadablePass {
        kind: &'static str,
        archive: String,
        id: u16,
    },
}

impl std::fmt::Display for DemoGap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoReader => write!(formatter, "no STB timeline reader is installed"),
            Self::NoFileName => write!(formatter, "the package cut names no demo file"),
            Self::NoDemoData { name } => {
                write!(formatter, "no demo file named {name} in the demo archives")
            }
            Self::ActorTableFull => write!(formatter, "the demo actor table is full"),
            Self::NoNameInfo { name } => write!(formatter, "{name} is not in the name table"),
            Self::NoParameterRecord => {
                write!(formatter, "the demo actor carries no parameter record")
            }
            Self::UnknownParameter { id } => {
                write!(
                    formatter,
                    "parameter record {id} carries no field of this kind"
                )
            }
            Self::Unchanged => write!(formatter, "the value is unchanged"),
            Self::UnreadableFile => write!(formatter, "the installed reader cannot read this file"),
            Self::UnreadableName { at, name } => write!(
                formatter,
                "the block ID at {at} ({:?}) is not a readable C string",
                String::from_utf8_lossy(name)
            ),
            Self::NoObject { kind, name } => {
                write!(formatter, "the {kind} object {name} is not on the stage")
            }
            Self::NoDemoArchive { name } if name.is_empty() => {
                write!(formatter, "no demo archive is named for this stage")
            }
            Self::NoDemoArchive { name } => {
                write!(formatter, "the demo archive {name} is not mounted")
            }
            Self::NoDemoModel { archive, id } => {
                write!(
                    formatter,
                    "the demo archive {archive} carries no model {id:#06x}"
                )
            }
            Self::NoDemoAnm { archive, id } => {
                write!(
                    formatter,
                    "the demo archive {archive} carries no readable bck {id:#06x}"
                )
            }
            Self::UnreadablePass { kind, archive, id } => {
                write!(
                    formatter,
                    "the demo archive {archive} carries {id:#06x} but not as a {kind} pass"
                )
            }
        }
    }
}

/// One timeline command the reader could not apply. Every variant is a place the reference does
/// something this port has no side for: a value that comes out of the function-value bank, a read
/// the reference answers with undefined memory, or a stage object whose J3D model this port never
/// built. The reader records a gap where the command runs and [`DemoManager::take_gaps`] hands
/// them to the shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemoCommandGap {
    /// `adaptor_setVariableValue_FVR_NAME_` (`jstudio-object.cpp:221-225`) looked the paragraph's
    /// ID up in the FVB bank and `TControl::getFunctionValue` answered NULL (`jstudio-control.h`
    /// `getFunctionValue`), which retail hands straight to `setValue_functionValue` and then
    /// dereferences in `update_functionValue_` (`jstudio-object.cpp:65-68`). Nothing else in the
    /// port can stand in for the missing entry.
    FunctionValueName { name: Vec<u8> },
    /// `adaptor_setVariableValue_FVR_INDEX_` (`jstudio-object.cpp:227-231`) looked the paragraph's
    /// index up in the FVB bank and `getFunctionValue_index` answered NULL, the same bare pointer
    /// retail dereferences.
    FunctionValueIndex { index: u32 },
    /// `TFunctionValue_composite::prepare_data_` (`fvb.cpp:220-239`) takes the operation's row out
    /// of `saCompositeOperation_`; the table's `COMPOSITE_NONE` row is `{NULL, NULL}` (`:202-211`),
    /// so an operation of 0 - or one past the table - is a call through a NULL function pointer in
    /// retail. The port drops the bank entry the block would have become instead of inventing a
    /// value for it.
    FvbComposite { operation: u32 },
    /// `adaptor_do_PARENT_NODE` / `RELATION_NODE`'s name arm (`object-actor.cpp:152-166`) or
    /// `TAdaptor_particle::adaptor_do_PARENT_NODE`'s (`object-particle.cpp:110-127`) call
    /// `JSGFindNodeID(name)`; `dDemo_actor_c`'s answer comes from the demo actor's J3D model
    /// joint-name table (`d_demo.h:84-87`), and the model itself is built by `daDemo00_c`'s
    /// `actStandby` out of the room's demo archive [`RoomDemoArchive`] mounts
    /// (`d_a_demo00.cpp:169`, `:303`). A stage object whose model has no such joint - or no model
    /// at all, because no shape was staged or the archive could not answer - is this gap, which
    /// names the object the lookup ran in.
    NodeName {
        object: DemoObjectRef,
        name: Vec<u8>,
    },
    /// `TJPACallback_::execute`'s parented branch calls
    /// `JStage::TObject::JSGGetNodeTransformation` for the parent node's matrix
    /// (`object-particle.cpp:248-259`); the matrix is the demo actor's J3D animation matrix
    /// (`d_demo.h:88-92`), which needs the model [`RoomDemoArchive`] feeds
    /// (`d_a_demo00.cpp:303`).
    NodeTransform { object: DemoObjectRef, node: u32 },
    /// `JPAEmitterManager::createSimpleEmitterID` resolves the id the command carries through
    /// `pResMgrArray[rmID]->getEmitterResource()->getByUserIndex(userID)`
    /// (`JPAEmitterManager.cpp:52-78`) and creates no emitter when that answers NULL. The
    /// resolution is [`super::jpa::Jpa::create_simple_emitter_packed_id`], which creates nothing
    /// and records the miss as a [`super::jpa::JpaGap`]; this gap is the adaptor's own report of
    /// the same miss, naming the packed id the command carried.
    ParticleId { id: u32 },
    /// A `JMSG` code whose group has no message resource. `JMessage::TControl::setMessageCode`
    /// resolves the group through `mResourceContainer->Get_groupID` (`JMessage/control.cpp:29-39`)
    /// and answers with no entry when the container has none; the port's container is the
    /// per-group table `assets::load_msg_table` fills from the `bmg`s the dump ships - groups 0
    /// and 1, from `bmgres.arc` and `bmgresh.arc` (`d_mesg.cpp:2186-2202`) - so a code naming any
    /// other group has nothing to resolve in.
    MessageGroup { group: u16 },
    /// A read past the end of the demo file. The reference's objects all point into the file
    /// buffer (`d_demo.cpp:720`), so a command whose operand runs past its block still reads the
    /// following block's bytes; only a read past the buffer itself has no value at all, and that
    /// is what this names.
    Truncated { at: usize },
    /// A sequence jump whose target falls outside the demo file (`stb.cpp:179-185`,
    /// `process_sequence_` case 3). A jump that lands outside its block but inside the file is
    /// the reference's own behaviour and runs here.
    JumpOutside { at: usize, target: isize },
}

impl std::fmt::Display for DemoCommandGap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FunctionValueName { name } => write!(
                formatter,
                "function value {} is not in the FVB bank",
                String::from_utf8_lossy(name)
            ),
            Self::FunctionValueIndex { index } => {
                write!(formatter, "function value #{index} is not in the FVB bank")
            }
            Self::FvbComposite { operation } => {
                write!(
                    formatter,
                    "FVB composite operation {operation} has no function"
                )
            }
            Self::NodeName { object, name } => write!(
                formatter,
                "node {} is not in {object}'s J3D joint-name table",
                String::from_utf8_lossy(name)
            ),
            Self::NodeTransform { object, node } => write!(
                formatter,
                "node {node}'s transformation is not in {object}'s J3D model"
            ),
            Self::ParticleId { id } => {
                write!(
                    formatter,
                    "particle id {id} is not confirmed through the JPA bank"
                )
            }
            Self::MessageGroup { group } => {
                write!(formatter, "message group {group} has no message archive")
            }
            Self::Truncated { at } => {
                write!(formatter, "the command at {at} reads past the demo file")
            }
            Self::JumpOutside { at, .. } => {
                write!(formatter, "the jump at {at} lands outside the demo file")
            }
        }
    }
}

/// `DEMO_SELECT(0x10000, 0x10000000)` - the bit the parameter record's keys 6 and 7 OR into the
/// staged `mBtkID` / `mBrkID` (`d_a_demo00.cpp:808`, `:810`), which every arm that reads the id
/// then tests (`:359`, `:437`, `:444`, `:462`, `:474`).
pub const PASS_SELECT_BIT: i32 = 0x1000_0000;

/// Which of the two `mDoExt_*Anm::init` call sites a pass is registered from. `createHeap`'s
/// (`d_a_demo00.cpp:189-207`) and `actPerformance`'s per-key swap (`:346-370`) differ in exactly
/// two things: the attribute, which the swap reads off the staged id's [`PASS_SELECT_BIT`]
/// (`:359`), and the start frame, which the swap takes from the cursor the pass already had
/// (`:365`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassInit {
    /// `createHeap`: `EMode_NULL` with the cursor at zero.
    Heap,
    /// `actPerformance`'s swap: `EMode_LOOP` and a kept cursor when the id carries the bit.
    Swap,
}

/// One `mDoExt_btpAnm` (`m_Do/m_Do_ext.h:131-149`): the parsed texture-pattern key, its frame
/// cursor, and the texture units its material entries update.
#[derive(Debug, Clone)]
struct TexNoPass {
    anm: crate::j3d::btp::TexPattern,
    ctrl: crate::j3d::ctrl::FrameCtrl,
    /// One `setTexNoAnm(texNo, &anmR[i])` pair (`J3DMaterialAttach.cpp:261-270`): the material the
    /// entry updates, the texture unit it writes and the entry it samples.
    units: Vec<TexNoUnit>,
}

#[derive(Debug, Clone, Copy)]
struct TexNoUnit {
    material: usize,
    unit: usize,
    entry: usize,
}

/// One `mDoExt_btkAnm` (`m_Do/m_Do_ext.h:49-70`).
#[derive(Debug, Clone)]
struct TexMtxPass {
    anm: crate::j3d::btk::TextureAnimation,
    ctrl: crate::j3d::ctrl::FrameCtrl,
    /// The staged id's [`PASS_SELECT_BIT`], which the frame arm tests instead of the cursor's
    /// attribute (`d_a_demo00.cpp:437`, `:462`).
    select: bool,
    /// One `setTexMtxAnm(texMtxID, &pTexAnm[no])` pair per animation track the model's material
    /// names answer to (`J3DMaterialAttach.cpp:284-304`). The reference also writes the SRT's
    /// centre and the calc type into the material's tex-matrix info there (`:298-301`); the port's
    /// [`crate::j3d::btk::UvMatrix`] folds the centre in per sample.
    tracks: Vec<TexMtxTrack>,
}

#[derive(Debug, Clone, Copy)]
struct TexMtxTrack {
    material: usize,
    /// The animation's own `getUpdateTexMtxID(no)`, which is the material slot the reference
    /// writes (`setTexMtxAnm(texMtxID, ...)`, `J3DMaterialAttach.cpp:289-303`).
    slot: usize,
    track: u32,
}

/// One `mDoExt_brkAnm` (`m_Do/m_Do_ext.h:79-101`).
#[derive(Debug, Clone)]
struct TevRegPass {
    anm: crate::j3d::brk::TevRegKey,
    ctrl: crate::j3d::ctrl::FrameCtrl,
    /// The staged id's [`PASS_SELECT_BIT`] (`d_a_demo00.cpp:444`, `:474`).
    select: bool,
    /// One `setTevColorAnm(colorId, &pCRegAnmR[i])` pair (`J3DMaterialAttach.cpp:320-329`).
    colors: Vec<TevRegSlot>,
    /// One `setTevKColorAnm(colorId, &pKRegAnmR[i])` pair (`:331-341`).
    k_colors: Vec<TevRegSlot>,
}

#[derive(Debug, Clone, Copy)]
struct TevRegSlot {
    material: usize,
    color: u8,
    entry: usize,
}

/// The node table `dDemo_actor_c` reaches through `mModel`: `JSGFindNodeID` searches
/// `mModel->getModelData()->getJointName()` (`d_demo.h:84-87`) and `JSGGetNodeTransformation`
/// reads `mModel->getAnmMtx(no)` (`:88-92`). A J3D model's two halves, held so the demo reader
/// can answer without a renderer, in the joint order [`RoomMesh`] walks.
///
/// The animation retail splits between `mModel.mpModel` (the `J3DModel` whose `getAnmMtx` answers)
/// and `mModel.mpMorf` (the `mDoExt_McaMorf` that owns the `bck` and the frame cursor,
/// `d_a_demo00.h:51-53`) is one value here: [`DemoNodeTable::set_frame`] reads the morph and
/// rewrites the matrices through it.
///
/// `createHeap`'s other half is the model's material table, which it inits three keys onto: the
/// staged `mBtpID` texture pattern (`:173-187`), `mBtkID` texture SRT (`:189-207`) and `mBrkID`
/// tev registers (`:209-221`). Each pass lands in the slots its entries name
/// (`J3DMaterialAnm::calc`, `J3DMaterialAnm.cpp:51-76`), the way the reference's `calc` does, and
/// `draw` re-applies them at the frame the cursor holds (`d_a_demo00.cpp:530-560`).
///
/// [`RoomMesh`]: crate::j3d::bdl::RoomMesh
#[derive(Debug, Clone, Default)]
pub struct DemoNodeTable {
    /// `J3DModelData::getJointName()`, in joint order.
    names: Vec<String>,
    /// `mModel->getAnmMtx(no)` per joint. Without a morph these are the `JNT1` block's node
    /// transforms, which is what a model built from a `bdl4` has before an animation runs.
    matrices: Vec<Mat4>,
    /// `mModel->getModelData()`'s material table (`J3DMaterialAttach`), in `MAT3` order, with the
    /// three passes' last writes in its slots.
    materials: Vec<crate::j3d::bdl::MaterialSlot>,
    /// `mModel.mpMorf` (`d_a_demo00.h:52`): the morph `daDemo00_c::createHeap` builds from the
    /// staged `mBckID` (`:223-253`). `None` is that block's other arm, where the reference makes
    /// a plain model (`:224-225`) and loops never move its joints off the bind pose.
    morph: Option<Morph>,
    /// The frame the morph has posed the matrices at. `None` while they are still the model's
    /// own bind pose: `createHeap` builds the morph with its cursor at zero and `setBaseMtx`'s
    /// `mpModel->calc()` (`d_a_demo00.cpp:84-91`) has not run over it yet, or no morph exists.
    frame: Option<f32>,
    /// `mModel.mpBtpAnm` (`d_a_demo00.h:52`).
    tex_pattern: Option<TexNoPass>,
    /// `mModel.mpBtkAnm`.
    texture_srt: Option<TexMtxPass>,
    /// `mModel.mpBrkAnm`.
    tev_reg: Option<TevRegPass>,
}

impl PartialEq for DemoNodeTable {
    /// The two fields the node lookups read. The morph is the animation the matrices were posed
    /// from, not part of what `JSGFindNodeID` or `JSGGetNodeTransformation` answers, so two tables
    /// holding the same joint table compare equal whatever animation posed them; the material
    /// table is what the passes write, and no node lookup reads it either.
    fn eq(&self, other: &Self) -> bool {
        self.names == other.names && self.matrices == other.matrices
    }
}

impl DemoNodeTable {
    pub fn new(
        names: Vec<String>,
        matrices: Vec<Mat4>,
        materials: Vec<crate::j3d::bdl::MaterialSlot>,
    ) -> Self {
        Self {
            names,
            matrices,
            materials,
            morph: None,
            frame: None,
            tex_pattern: None,
            texture_srt: None,
            tev_reg: None,
        }
    }

    /// `mModel->getModelData()`'s material table with the passes' last writes in it. The cut
    /// model itself is not drawn by this port - the demo actor draws as its cull box - so this is
    /// where a caller reads what a `btp`, `btk` or `brk` key left on a material.
    pub fn materials(&self) -> &[crate::j3d::bdl::MaterialSlot] {
        &self.materials
    }

    /// The frame the morph was last posed at, which is the cursor `mpMorf` holds
    /// (`mDoExt_McaMorf::getFrame`). `None` before the first pose.
    pub fn frame(&self) -> Option<f32> {
        self.frame
    }

    /// `mModel.mpMorf`'s construction: the cut's morph, built from the `bck` the staged
    /// `mBckID` names (`d_a_demo00.cpp:229-253`). The next [`DemoNodeTable::set_frame`] poses the
    /// matrices from it whatever frame the table was left at.
    pub fn set_morph(&mut self, morph: Morph) {
        self.morph = Some(morph);
        self.frame = None;
    }

    /// `mModel.mpMorf->getEndFrame()` (`d_a_demo00.cpp:304-305`), the frame count `actStandby`
    /// hands `setAnmFrameMax`. `None` is the plain model with no morph, whose frame max
    /// `actStandby` never writes.
    pub fn morph_frame_max(&self) -> Option<f32> {
        self.morph.as_ref().map(Morph::frame_max)
    }

    /// `mModel.mpMorf->setFrame(f)` (`d_a_demo00.cpp:412-452`) over `mpModel->calc()`: pose every
    /// joint at `frame`. A table with no morph keeps its bind pose, which is `mpMorf == NULL`.
    ///
    /// Re-posing a frame already posed is skipped: `setFrame` rewrites the cursor retail's
    /// `getAnmMtx` reads lazily, and the port's matrices are the same for the same frame.
    pub fn set_frame(&mut self, frame: f32) {
        let Some(morph) = &self.morph else {
            return;
        };
        if self.frame == Some(frame) {
            return;
        }
        self.frame = Some(frame);
        self.matrices = morph.joint_matrices(frame);
    }

    /// The btp arm (`d_a_demo00.cpp:173-187`, re-inited at `:336-343`): `mDoExt_btpAnm::init`
    /// (`m_Do_ext.cpp:195-228`) over the archive's object, whose material entries
    /// `J3DAnmTexPattern::searchUpdateMaterialID` resolves by name (`J3DAnimation.cpp:862-871`),
    /// with the frame cursor `initPlay` leaves at zero. `createHeap` inits this key with
    /// `J3DFrameCtrl::EMode_NULL` and so does every later arm, which never tests the staged id for
    /// it; only the btk and brk keys carry [`PASS_SELECT_BIT`].
    ///
    /// `None` is the archive's miss, which retail's own `anm == NULL` arm answers with an early
    /// `return TRUE` (`:180-186`): the cut is built without the pass.
    pub fn set_tex_pattern(&mut self, pass: Option<crate::j3d::btp::TexPattern>) {
        let registered = pass.map(|mut anm| {
            anm.resolve(|name| self.material_index(name));
            let units = anm
                .material_ids
                .iter()
                .enumerate()
                .filter_map(|(entry, id)| {
                    let material = material_slot(*id, self.materials.len())?;
                    // `setTexNoAnm(pAnm->getAnmTable()[i].mTexNo, &pAnmR[i])` - and
                    // `J3DMaterialAnm::calc` then writes that unit of the material's `TevBlock`.
                    let unit = usize::from(anm.entries.get(entry)?.tex_no);
                    (unit < crate::j3d::bdl::TEX_UNITS).then_some(TexNoUnit {
                        material,
                        unit,
                        entry,
                    })
                })
                .collect();
            let ctrl =
                crate::j3d::ctrl::FrameCtrl::init_play(anm.frame_max, crate::j3d::ctrl::MODE_NULL);
            TexNoPass { anm, ctrl, units }
        });
        self.tex_pattern = registered;
    }

    /// The btk arm (`d_a_demo00.cpp:189-207`, re-inited at `:346-370`): `mDoExt_btkAnm::init`
    /// (`m_Do_ext.cpp:251-296`) over the archive's object. One entry per track, the way
    /// `setTexMtxAnimator` walks them (`J3DMaterialAttach.cpp:284-306`): the track's material name
    /// resolved against the model's material table, the track's own `getUpdateTexMtxID` as the
    /// slot, and an `0xFF` slot or an unresolved name skipped. Each registered slot also
    /// re-points that material's matching coord at its own matrix (`setTexGenMtx(GX_TEXMTX0 +
    /// texMtxID * 3)`, `:293-294`), which is where a coord parked on `GX_IDENTITY` starts reading
    /// the animated matrix.
    pub fn set_texture_srt(
        &mut self,
        pass: Option<crate::j3d::btk::TextureAnimation>,
        id: i32,
        init: PassInit,
    ) {
        let select = id & PASS_SELECT_BIT != 0;
        let frame = self
            .texture_srt
            .as_ref()
            .map_or(0.0, |pass| pass.ctrl.frame());
        let registered = pass.map(|anm| {
            let tracks: Vec<TexMtxTrack> = anm
                .names
                .iter()
                .enumerate()
                .filter_map(|(track, name)| {
                    let slot = usize::from(*anm.texture_matrix_ids.get(track)?);
                    if slot == 0xFF || slot >= crate::j3d::bdl::TEX_MTX_SLOTS {
                        return None;
                    }
                    let material = usize::from(self.material_index(name)?);
                    Some(TexMtxTrack {
                        material,
                        slot,
                        track: u32::try_from(track).ok()?,
                    })
                })
                .collect();
            for track in &tracks {
                if let Some(coord) = self
                    .materials
                    .get_mut(track.material)
                    .and_then(|material| material.tex_gen.get_mut(track.slot))
                {
                    *coord = Some(track.slot as u8);
                }
            }
            let ctrl = pass_init(anm.frame_max, select, init, frame);
            TexMtxPass {
                anm,
                ctrl,
                select,
                tracks,
            }
        });
        self.texture_srt = registered;
    }

    /// The brk arm (`d_a_demo00.cpp:209-221`, re-inited at `:373-397`): `mDoExt_brkAnm::init`
    /// (`m_Do_ext.cpp:324-382`) over the archive's object, whose entries
    /// `J3DAnmTevRegKey::searchUpdateMaterialID` resolves by name.
    ///
    /// The register a C-reg entry names is kept as the reference registers it. Its `calc` only
    /// applies the first three (`J3DMaterialAnm.cpp:63`), so a `colorId` of three is stored and
    /// never written back, exactly as in the reference.
    pub fn set_tev_reg(
        &mut self,
        pass: Option<crate::j3d::brk::TevRegKey>,
        id: i32,
        init: PassInit,
    ) {
        let select = id & PASS_SELECT_BIT != 0;
        let frame = self.tev_reg.as_ref().map_or(0.0, |pass| pass.ctrl.frame());
        let registered = pass.map(|mut anm| {
            anm.resolve(|name| self.material_index(name));
            let colors = register_slots(&anm.c_ids, &anm.c_reg, self.materials.len());
            let k_colors = register_slots(&anm.k_ids, &anm.k_reg, self.materials.len());
            let ctrl = pass_init(anm.frame_max, select, init, frame);
            TevRegPass {
                anm,
                ctrl,
                select,
                colors,
                k_colors,
            }
        });
        self.tev_reg = registered;
    }

    /// The btp, btk and brk half of `actPerformance`'s frame arm (`d_a_demo00.cpp:412-452`), which
    /// sets each pass a frame short of `fVar1` and plays it back. With a rate of one and the
    /// `EMode_NULL` attribute those inits use, `setFrame(fVar1 - 1.0f)` followed by `play` leaves
    /// the cursor exactly on `fVar1`, which is what this sets. A pass whose staged id carried
    /// [`PASS_SELECT_BIT`] only plays (`:436-447`), advancing on its own cursor instead.
    ///
    /// `draw` re-applies what each cursor holds to the material table
    /// (`mDoExt_*Anm::entry`, `d_a_demo00.cpp:530-560`) and `J3DMaterialAnm::calc` writes the
    /// slots; the port does both after the step.
    pub fn set_pass_frames(&mut self, frame: f32) {
        if let Some(pass) = &mut self.tex_pattern {
            pass.ctrl.set_frame(frame);
        }
        if let Some(pass) = &mut self.texture_srt {
            if pass.select {
                pass.ctrl.update();
            } else {
                pass.ctrl.set_frame(frame);
            }
        }
        if let Some(pass) = &mut self.tev_reg {
            if pass.select {
                pass.ctrl.update();
            } else {
                pass.ctrl.set_frame(frame);
            }
        }
        self.apply_passes();
    }

    /// The `else if` chain `actPerformance` ends on (`d_a_demo00.cpp:481-490`): with
    /// `ENABLE_ANM_FRAME_e` clear, the morph plays, and a cut with no morph plays its first
    /// staged pass instead - btp, else btk, else brk.
    pub fn play_pass(&mut self) {
        if self.morph.is_none() {
            if let Some(pass) = &mut self.tex_pattern {
                pass.ctrl.update();
            } else if let Some(pass) = &mut self.texture_srt {
                pass.ctrl.update();
            } else if let Some(pass) = &mut self.tev_reg {
                pass.ctrl.update();
            }
        }
        self.apply_passes();
    }

    /// `JUTNameTab::getIndex(name)` - `JUTNameTab.cpp:368-378`: the entry index, which is the
    /// node id `JSGGetNodeTransformation` is handed. `None` is retail's `-1`.
    pub fn find_node_id(&self, name: &[u8]) -> Option<u32> {
        let index = self
            .names
            .iter()
            .position(|joint| joint.as_bytes() == name)?;
        u32::try_from(index).ok()
    }

    /// `mModel->getAnmMtx(no)` for a node the table holds.
    pub fn node_transform(&self, node: u32) -> Option<Mat4> {
        self.matrices.get(usize::try_from(node).ok()?).copied()
    }

    /// The material name table's lookup, the one every `searchUpdateMaterialID` runs
    /// (`J3DMaterialAttach.cpp:265`, `getMaterialName()->getIndex`).
    fn material_index(&self, name: &str) -> Option<u16> {
        let index = self
            .materials
            .iter()
            .position(|material| material.name.as_deref() == Some(name))?;
        u16::try_from(index).ok()
    }

    /// `J3DMaterialAnm::calc` (`J3DMaterialAnm.cpp:51-76`) over the values the cursors hold, then
    /// the draw-time `entry` (`d_a_demo00.cpp:530-560`) that re-registers each pass on the
    /// material table: for the port the two are one write per slot.
    fn apply_passes(&mut self) {
        let Self {
            materials,
            tex_pattern,
            texture_srt,
            tev_reg,
            ..
        } = self;
        if let Some(pass) = tex_pattern {
            let frame = pass.ctrl.frame();
            for unit in &pass.units {
                if let Some(tex_no) = pass.anm.tex_no(unit.entry, frame)
                    && let Some(slot) = materials
                        .get_mut(unit.material)
                        .and_then(|material| material.tex_nos.get_mut(unit.unit))
                {
                    // `0xFFFF` is `J3DMaterialFactory`'s own "no texture" value, which the port's
                    // unit holds as `None`.
                    *slot = (tex_no != crate::j3d::bdl::MATERIAL_UNTEXTURED).then_some(tex_no);
                }
            }
        }
        if let Some(pass) = texture_srt {
            let frame = pass.ctrl.frame();
            for track in &pass.tracks {
                if let Some(matrix) = pass.anm.uv_transform(track.track, frame)
                    && let Some(slot) = materials
                        .get_mut(track.material)
                        .and_then(|material| material.tex_mtxs.get_mut(track.slot))
                {
                    *slot = Some(matrix);
                }
            }
        }
        if let Some(pass) = tev_reg {
            let frame = pass.ctrl.frame();
            for slot in &pass.colors {
                // `J3DMaterialAnm::calc`'s C-reg loop stops at three (`J3DMaterialAnm.cpp:63`).
                if usize::from(slot.color) >= TEV_COLOR_APPLIED {
                    continue;
                }
                if let Some(rgba) = pass.anm.tev_color(slot.entry, frame)
                    && let Some(target) = materials
                        .get_mut(slot.material)
                        .and_then(|material| material.tev_colors.get_mut(usize::from(slot.color)))
                {
                    *target = Some(rgba);
                }
            }
            for slot in &pass.k_colors {
                if let Some(rgba) = pass.anm.tev_k_color(slot.entry, frame)
                    && let Some(target) = materials
                        .get_mut(slot.material)
                        .and_then(|material| material.tev_k_colors.get_mut(usize::from(slot.color)))
                {
                    *target = Some(rgba);
                }
            }
        }
    }
}

/// `J3DMaterialAnm::calc` applies the C registers below this index only
/// (`J3DMaterialAnm.cpp:63`), though the slot array it fills holds four.
const TEV_COLOR_APPLIED: usize = 3;

/// The attribute and start frame one registration gives a pass, which is the only pair that
/// separates `createHeap`'s `mDoExt_*Anm::init` (`d_a_demo00.cpp:198`, `:218`) from
/// `actPerformance`'s per-key swap (`:352-367`, `:379-394`): the swap takes `EMode_LOOP` and the
/// cursor the pass already had when the staged id carries [`PASS_SELECT_BIT`], and
/// `EMode_NULL` with a zero start otherwise (`m_Do_ext.cpp:85-114`).
fn pass_init(
    frame_max: u16,
    select: bool,
    init: PassInit,
    frame: f32,
) -> crate::j3d::ctrl::FrameCtrl {
    let attribute = match init {
        PassInit::Heap => crate::j3d::ctrl::MODE_NULL,
        PassInit::Swap if select => crate::j3d::ctrl::MODE_LOOP,
        PassInit::Swap => crate::j3d::ctrl::MODE_NULL,
    };
    let mut ctrl = crate::j3d::ctrl::FrameCtrl::init_play(frame_max, attribute);
    if init == PassInit::Swap && select {
        ctrl.set_frame(frame);
    }
    ctrl
}

/// A resolved material id as an index. `searchUpdateMaterialID` writes `0xFFFF` for a name the
/// model's material table does not carry, and every `set*Animator` loop skips those entries
/// (`J3DMaterialAttach.cpp:263`, `:286`, `:321`), so a miss leaves that material alone.
fn material_slot(id: u16, materials: usize) -> Option<usize> {
    let index = usize::from(id);
    (index < materials).then_some(index)
}

/// `setTevRegAnimator`'s per-family loop (`J3DMaterialAttach.cpp:320-341`): one slot per entry
/// whose material id resolved, holding the register index that entry's table names.
fn register_slots(
    ids: &[u16],
    entries: &[crate::j3d::brk::Entry],
    materials: usize,
) -> Vec<TevRegSlot> {
    ids.iter()
        .enumerate()
        .filter_map(|(entry, id)| {
            let material = material_slot(*id, materials)?;
            let color = entries.get(entry)?.color_id;
            Some(TevRegSlot {
                material,
                color,
                entry,
            })
        })
        .collect()
}

/// `dStage_roomControl_c::mDemoArcName` (`d_stage.h:933`), the room's demo archive, and the cut
/// models read out of it.
///
/// `d_s_room.cpp:220-231` names it: the room file's `LBNK` banks are indexed by
/// `dComIfG_play_c::getLayerNo(roomNo)`, the first room whose bank is not `0xFF` writes
/// `sprintf(mDemoArcName, "Demo%02d", bank)` and mounts the archive with `dComIfG_setObjectRes`.
/// `daDemo00_c::createHeap` is the reader: `dComIfG_getObjectIDRes(mDemoArcName, mShapeID)`
/// (`d_a_demo00.cpp:169`) is what the cut's J3D model is built from, and its node table is what
/// `act->setModel` hands the demo actor (`:303`).
///
/// The reference clears the name when the mount fails (`:231`), which lets a later room set name
/// another archive; the port keeps the name instead, so a miss can report the archive it wanted.
/// The objects read out of it are kept as the node tables [`DemoActor::set_model`] takes, one
/// lookup per object id, and each lookup keeps the model's `JNT1` parent list beside the table so
/// a morph can be paired with the joint order it drives.
#[derive(Resource, Default)]
pub struct RoomDemoArchive {
    name: String,
    mounted: Option<Archive>,
    models: HashMap<u16, Result<DemoModel, DemoGap>>,
}

/// One shape id's build: the node table `dComIfG_getObjectIDRes` answered with, and the joint
/// parents the same object parsed to. The mesh's geometry and textures are dropped - nothing
/// reads them - and the parents stay for `d_a_demo00_c::createHeap`'s morph.
#[derive(Debug, Clone)]
struct DemoModel {
    table: DemoNodeTable,
    parents: Vec<u16>,
}

impl RoomDemoArchive {
    /// The `if (dStage_roomControl_c::mDemoArcName[0] == '\0')` guard the naming sits behind
    /// (`d_s_room.cpp:220`) and the mount that follows it (`:230`): the first room set that names
    /// a bank names the archive, and later sets leave it where it is.
    pub fn name_from_room(&mut self, name: Option<&str>) {
        if !self.name.is_empty() {
            return;
        }
        let Some(name) = name else {
            return;
        };
        self.name = name.to_owned();
        self.mounted = crate::assets::load_object_archive(name);
    }

    /// `dComIfG_getObjectRes(dStage_roomControl_c::getDemoArcName(), resName)`
    /// (`d_com_inf_game.h:4110-4112`) for the demo file `specialProcPackage` mounts
    /// (`d_event_data.cpp:797-799`): the archive entry under the name the cut's `FileName` key
    /// carries. `None` is the reference's NULL, which is what makes it fall through to the stage
    /// archive (`:800-801`).
    pub fn data(&self, name: &str) -> Option<Vec<u8>> {
        let data = self.mounted.as_ref()?.read(name).ok()?;
        Some(data.to_vec())
    }

    /// `dComIfG_getObjectIDRes(dStage_roomControl_c::getDemoArcName(), id)` for the cut model
    /// `daDemo00_c::createHeap` builds (`d_a_demo00.cpp:169`): the object's joint-name table and
    /// bind-pose matrices, which is the pair `JSGFindNodeID` and `JSGGetNodeTransformation` read
    /// (`d_demo.h:84-92`). A miss - nothing mounted, or no such object - is a typed [`DemoGap`],
    /// where the reference dereferences the NULL.
    pub fn model(&mut self, id: u16) -> Result<DemoNodeTable, DemoGap> {
        Ok(self.resolve(id)?.table.clone())
    }

    /// `dComIfG_getObjectIDRes(dStage_roomControl_c::getDemoArcName(), (u16)mBckID)`
    /// (`d_com_inf_game.h:4122-4124`) for the morph `daDemo00_c::createHeap` builds
    /// (`d_a_demo00.cpp:229-253`): the `bck` under the staged id, parsed over the `JNT1` joint
    /// order `shape` resolved to, which is the order an `ANK1` table's joint indices count in.
    ///
    /// The keys are parsed per staged pair rather than cached: a morph carries every joint local
    /// of every disk frame, and the one the actor holds is the one that is read.
    pub fn morph(&mut self, shape: u16, id: u16) -> Result<Morph, DemoGap> {
        let parents = self.resolve(shape)?.parents.clone();
        let Some(mounted) = &self.mounted else {
            return Err(DemoGap::NoDemoArchive {
                name: self.name.clone(),
            });
        };
        let data = match mounted.read_id(id) {
            Ok(data) => data,
            Err(_) => return Err(self.no_anm(id)),
        };
        Morph::parse(data, &parents).map_err(|error| {
            log::warn!(
                "clearsleeper: demo archive {} bck {id:#06x} unreadable ({})",
                self.name,
                error.as_str()
            );
            self.no_anm(id)
        })
    }

    /// One `dComIfG_getObjectIDRes(dStage_roomControl_c::getDemoArcName(), (u16)id)` lookup
    /// (`d_com_inf_game.h:4122-4124`) over the mounted archive, which is the read `createHeap`
    /// makes for each staged key. `Ok(None)` is the reference's NULL, which every btp, btk and
    /// brk arm tolerates with its own `return TRUE` (`d_a_demo00.cpp:180-186`, `:199-205`,
    /// `:218`): the cut is built without that pass. The unreadable case is the port's own, where
    /// the object is there and its bytes are not a shape this port can follow.
    fn pass(&self, id: u16) -> Result<Option<&[u8]>, DemoGap> {
        let Some(mounted) = &self.mounted else {
            return Err(DemoGap::NoDemoArchive {
                name: self.name.clone(),
            });
        };
        match mounted.read_id(id) {
            Ok(data) => Ok(Some(data)),
            Err(_) => Ok(None),
        }
    }

    /// `J3DAnmTexPattern` under the staged `mBtpID` (`d_a_demo00.cpp:173-187`).
    pub fn tex_pattern(&mut self, id: u16) -> Result<Option<crate::j3d::btp::TexPattern>, DemoGap> {
        let Some(data) = self.pass(id)?.map(<[u8]>::to_vec) else {
            return Ok(None);
        };
        crate::j3d::btp::parse(&data).map(Some).map_err(|error| {
            log::warn!(
                "clearsleeper: demo archive {} btp {id:#06x} unreadable ({})",
                self.name,
                error.as_str()
            );
            self.unreadable_pass("btp", id)
        })
    }

    /// `J3DAnmTextureSRTKey` under the staged `mBtkID` (`d_a_demo00.cpp:189-207`).
    pub fn texture_srt(
        &mut self,
        id: u16,
    ) -> Result<Option<crate::j3d::btk::TextureAnimation>, DemoGap> {
        let Some(data) = self.pass(id)?.map(<[u8]>::to_vec) else {
            return Ok(None);
        };
        crate::j3d::btk::parse(&data).map(Some).map_err(|error| {
            log::warn!(
                "clearsleeper: demo archive {} btk {id:#06x} unreadable ({})",
                self.name,
                error.as_str()
            );
            self.unreadable_pass("btk", id)
        })
    }

    /// `J3DAnmTevRegKey` under the staged `mBrkID` (`d_a_demo00.cpp:209-221`).
    pub fn tev_reg(&mut self, id: u16) -> Result<Option<crate::j3d::brk::TevRegKey>, DemoGap> {
        let Some(data) = self.pass(id)?.map(<[u8]>::to_vec) else {
            return Ok(None);
        };
        crate::j3d::brk::parse(&data).map(Some).map_err(|error| {
            log::warn!(
                "clearsleeper: demo archive {} brk {id:#06x} unreadable ({})",
                self.name,
                error.as_str()
            );
            self.unreadable_pass("brk", id)
        })
    }

    /// The miss of [`RoomDemoArchive::tex_pattern`] and its two siblings: the archive carries the
    /// staged object, but not as the pass that key names.
    fn unreadable_pass(&self, kind: &'static str, id: u16) -> DemoGap {
        DemoGap::UnreadablePass {
            kind,
            archive: self.name.clone(),
            id,
        }
    }

    /// The miss both of [`RoomDemoArchive::morph`]'s arms answer with: the object the staged
    /// `mBckID` names is not there, or is not a `J3D1bck1` this port can read.
    fn no_anm(&self, id: u16) -> DemoGap {
        DemoGap::NoDemoAnm {
            archive: self.name.clone(),
            id,
        }
    }

    fn resolve(&mut self, id: u16) -> Result<&DemoModel, DemoGap> {
        if !self.models.contains_key(&id) {
            let resolved = match &self.mounted {
                Some(archive) => match crate::assets::object_model(archive, id) {
                    Some(mesh) => Ok(DemoModel {
                        table: DemoNodeTable::new(
                            mesh.joint_names,
                            mesh.joint_matrices,
                            mesh.materials,
                        ),
                        parents: mesh.joint_parents,
                    }),
                    None => Err(DemoGap::NoDemoModel {
                        archive: self.name.clone(),
                        id,
                    }),
                },
                None => Err(DemoGap::NoDemoArchive {
                    name: self.name.clone(),
                }),
            };
            self.models.insert(id, resolved);
        }
        self.models
            .get(&id)
            .expect("just inserted")
            .as_ref()
            .map_err(Clone::clone)
    }
}

/// `dDemo_actor_c` - `d_demo.h:52-153`. Every field is one a `JSGSet*` overload writes and the
/// enable mask it raises; a reader tests the mask first.
#[derive(Debug, Clone, PartialEq)]
pub struct DemoActor {
    flags: u16,
    trans: Vec3,
    scale: Vec3,
    rotate: [i16; 3],
    shape_id: u32,
    anm_id: u32,
    anm_frame: f32,
    anm_transition: f32,
    anm_frame_max: f32,
    tex_animation: u32,
    tex_animation_frame: f32,
    tex_animation_frame_max: f32,
    prm: Prm,
    /// `mActorId`, held as the entity a `fopAcM_SearchByID` would find. `None` is the reference's
    /// `fpcM_ERROR_PROCESS_ID_e`, the state `fopAc_Delete` leaves through `setActor(NULL)`
    /// (`f_op_actor.cpp:245-247`).
    actor: Option<Entity>,
    old_anm_id: i32,
    btp_id: i32,
    btk_id: i32,
    brk_id: i32,
    /// `mModel` - `d_demo.h:148`. `setModel` writes it from the model the demo cut built
    /// (`d_demo.h:119`, `d_demo.cpp:207`, `d_a_demo00.cpp:303`); it starts NULL, which the
    /// reference's `JSGFindNodeID` asserts against (`d_demo.h:85`).
    model: Option<DemoNodeTable>,
}

impl DemoActor {
    /// The constructor (`d_demo.cpp:14-22`) followed by `setActor` (`:43-52`). The constructor
    /// leaves `mShapeId`, `mAnmId`, `mAnmFrame`, `mAnmTransition` and `mTexAnimation` alone, which
    /// is safe only because their enable bits start clear; `setActor` is what writes `mOldAnmId`
    /// and the three animation caches.
    pub fn new(actor: Option<Entity>) -> Self {
        Self {
            flags: 0,
            trans: Vec3::ZERO,
            scale: Vec3::ONE,
            rotate: [0; 3],
            shape_id: 0,
            anm_id: 0,
            anm_frame: 0.0,
            anm_transition: 0.0,
            anm_frame_max: f32::MAX,
            tex_animation: 0,
            tex_animation_frame: 0.0,
            tex_animation_frame_max: f32::MAX,
            prm: Prm::default(),
            actor,
            old_anm_id: -1,
            btp_id: -1,
            btk_id: -1,
            brk_id: -1,
            model: None,
        }
    }

    /// `checkEnable(u16 mask)` - `d_demo.h:109`.
    pub fn check(&self, mask: u16) -> u16 {
        self.flags & mask
    }

    pub fn flags(&self) -> u16 {
        self.flags
    }

    /// `getActor()` (`d_demo.cpp:38-40`), resolved to the sim entity.
    pub fn actor(&self) -> Option<Entity> {
        self.actor
    }

    /// `setActor(fopAc_ac_c*)` - `d_demo.cpp:43-52`, including the four cache resets.
    pub fn set_actor(&mut self, actor: Option<Entity>) {
        self.actor = actor;
        self.old_anm_id = -1;
        self.btp_id = -1;
        self.btk_id = -1;
        self.brk_id = -1;
    }

    /// `setModel(J3DModel*)` - `d_demo.cpp:207` and `d_a_demo00.cpp:303`, the write that gives
    /// this stage object a node table. The demo-cut shell is what builds the model and hands its
    /// table over; a cut whose `daDemo00_c` never reached `actStandby`, or whose ids name no
    /// model any more, leaves this NULL.
    pub fn set_model(&mut self, model: Option<DemoNodeTable>) {
        self.model = model;
    }

    pub fn model(&self) -> Option<&DemoNodeTable> {
        self.model.as_ref()
    }

    /// The same model mutably: where [`DemoNodeTable::set_frame`] takes `actPerformance`'s frame
    /// (`d_a_demo00.cpp:412-452`).
    pub fn model_mut(&mut self) -> Option<&mut DemoNodeTable> {
        self.model.as_mut()
    }

    /// `JSGFindNodeID(const char*)` - `d_demo.h:84-87`: the model's joint-name table.
    pub fn find_node_id(&self, name: &[u8]) -> Option<u32> {
        self.model.as_ref()?.find_node_id(name)
    }

    /// `JSGGetNodeTransformation(u32, Mtx)` - `d_demo.h:88-92`: the node's matrix, `None` where
    /// the reference's read would answer `false` or run off the node array.
    pub fn node_transform(&self, node: u32) -> Option<Mat4> {
        self.model.as_ref()?.node_transform(node)
    }

    pub fn trans(&self) -> Vec3 {
        self.trans
    }

    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    /// `getRatate()` - `d_demo.h:113`, the raw `csXyz` triple `dDemo_setDemoData` copies into
    /// `current.angle` (`d_demo.cpp:200`), *not* the degree readback `JSGGetRotation` makes.
    pub fn rotate(&self) -> [i16; 3] {
        self.rotate
    }

    pub fn shape_id(&self) -> u32 {
        self.shape_id
    }

    pub fn anm_id(&self) -> u32 {
        self.anm_id
    }

    pub fn anm_frame(&self) -> f32 {
        self.anm_frame
    }

    pub fn anm_transition(&self) -> f32 {
        self.anm_transition
    }

    pub fn anm_frame_max(&self) -> f32 {
        self.anm_frame_max
    }

    pub fn old_anm_id(&self) -> i32 {
        self.old_anm_id
    }

    pub fn tex_animation(&self) -> u32 {
        self.tex_animation
    }

    pub fn tex_animation_frame(&self) -> f32 {
        self.tex_animation_frame
    }

    /// `JSGGetTextureAnimationFrameMax()` - `d_demo.h:94`, the max `TVVOutput_ANIMATION_FRAME_`
    /// reads for the texture-frame output (`object-actor.cpp:268-271`).
    pub fn tex_animation_frame_max(&self) -> f32 {
        self.tex_animation_frame_max
    }

    pub fn prm(&self) -> &Prm {
        &self.prm
    }

    /// `JSGSetData(u32, const void*, u32)` - `d_demo.cpp:259-264`.
    pub fn set_data(&mut self, id: u32, data: Vec<u8>, size: u32) {
        self.prm = Prm { id, data, size };
        self.flags |= ACTOR_ENABLE_UNK;
    }

    /// `JSGSetTranslation(const Vec&)` - `d_demo.cpp:267-270`.
    pub fn set_translation(&mut self, value: Vec3) {
        self.trans = value;
        self.flags |= ACTOR_ENABLE_TRANS;
    }

    /// `JSGSetScaling(const Vec&)` - `d_demo.cpp:273-276`.
    pub fn set_scaling(&mut self, value: Vec3) {
        self.scale = value;
        self.flags |= ACTOR_ENABLE_SCALE;
    }

    /// `JSGSetRotation(const Vec&)` - `d_demo.cpp:279-290`: a **degree** triple in, `cM_deg2s` out.
    pub fn set_rotation(&mut self, value: Vec3) {
        self.rotate = [deg2s(value.x), deg2s(value.y), deg2s(value.z)];
        self.flags |= ACTOR_ENABLE_ROTATE;
    }

    /// `JSGGetRotation(Vec*)` - `d_demo.h:97-107`, the readback that hands degrees back out.
    pub fn get_rotation(&self) -> Vec3 {
        Vec3::new(
            sh2d(f32::from(self.rotate[0])),
            sh2d(f32::from(self.rotate[1])),
            sh2d(f32::from(self.rotate[2])),
        )
    }

    /// `JSGSetShape(u32)` - `d_demo.cpp:293-296`.
    pub fn set_shape(&mut self, id: u32) {
        self.shape_id = id;
        self.flags |= ACTOR_ENABLE_SHAPE;
    }

    /// `JSGSetAnimation(u32)` - `d_demo.cpp:299-303`, including the frame-max reset that tells a
    /// reader no `setAnmFrameMax` has answered for the new animation yet.
    pub fn set_animation(&mut self, id: u32) {
        self.anm_id = id;
        self.anm_frame_max = f32::MAX;
        self.flags |= ACTOR_ENABLE_ANM;
    }

    /// `JSGSetAnimationFrame(f32)` - `d_demo.cpp:306-309`.
    pub fn set_animation_frame(&mut self, value: f32) {
        self.anm_frame = value;
        self.flags |= ACTOR_ENABLE_ANM_FRAME;
    }

    /// `JSGSetAnimationTransition(f32)` - `d_demo.cpp:312-315`, which raises the frame bit too,
    /// because `getPrm_Morf` reads the transition as the morph rate (`:127-129`).
    pub fn set_animation_transition(&mut self, value: f32) {
        self.anm_transition = value;
        self.flags |= ACTOR_ENABLE_ANM_FRAME;
    }

    /// `JSGSetTextureAnimation(u32)` - `d_demo.cpp:318-321`.
    pub fn set_texture_animation(&mut self, id: u32) {
        self.tex_animation = id;
        self.flags |= ACTOR_ENABLE_TEX_ANM;
    }

    /// `JSGSetTextureAnimationFrame(f32)` - `d_demo.cpp:324-327`.
    pub fn set_texture_animation_frame(&mut self, value: f32) {
        self.tex_animation_frame = value;
        self.flags |= ACTOR_ENABLE_TEX_ANM_FRAME;
    }

    /// `setAnmFrameMax(f32)` - `d_demo.h:118`, written by `dDemo_setDemoData` from the morph
    /// (`d_demo.cpp:230`) and by `daDemo00_c` when its model is built (`d_a_demo00.cpp:296`).
    pub fn set_anm_frame_max(&mut self, value: f32) {
        self.anm_frame_max = value;
    }

    /// `setOldAnmId(u32)` - `d_demo.h:122`.
    pub fn set_old_anm_id(&mut self, id: u32) {
        self.old_anm_id = id as i32;
    }

    /// `getPrm_Morf` - `d_demo.cpp:126-166`. The transition wins while the frame bit is up; the
    /// rest reads one field of the parameter record, each gated on the size the record was handed.
    pub fn prm_morf(&self) -> f32 {
        if self.check(ACTOR_ENABLE_ANM_FRAME) != 0 {
            return self.anm_transition;
        }
        if self.check(ACTOR_ENABLE_UNK) == 0 {
            return 0.0;
        }
        let prm = &self.prm;
        match prm.id {
            PRM_ID_UNK_1 if prm.size >= 4 => f32::from(prm.s16_at(1).unwrap_or(0) as u8),
            PRM_ID_UNK_2 if prm.size >= 7 => f32::from(prm.s16_at(2).unwrap_or(0) as u8),
            PRM_ID_UNK_4 if prm.size >= 6 => f32::from(prm.s16_at(1).unwrap_or(0) as u8),
            PRM_ID_UNK_5 if prm.size >= 0xB => f32::from(prm.s16_at(2).unwrap_or(0) as u8),
            PRM_ID_UNK_6 if prm.size >= 0xF => f32::from(prm.s16_at(2).unwrap_or(0) as u8),
            _ => 0.0,
        }
    }

    /// `getP_BtpData(const char*)` - `d_demo.cpp:55-85`, the first of the three the fork matched.
    ///
    /// The `ENABLE_TEX_ANM` path bypasses the parameter record; the parameter path reads a halfword
    /// at `+1` (`ID_UNK_1`), `+2` (`ID_UNK_2`), or a word at `+1` (`ID_UNK_4`) / `+2` (`ID_UNK_5`,
    /// `ID_UNK_6`). The id's `0x10000` bit swaps the archive to the room's demo arc, and the id is
    /// cached, so the second call for the same animation reports [`DemoGap::Unchanged`].
    pub fn p_btp_data(&mut self, name: &str) -> Result<AnmRef, DemoGap> {
        let mut archive;
        let id: u32;
        if self.check(ACTOR_ENABLE_TEX_ANM) != 0 {
            id = self.tex_animation;
            archive = DemoArchive::Uninitialized;
        } else {
            if self.check(ACTOR_ENABLE_UNK) == 0 {
                return Err(DemoGap::NoParameterRecord);
            }
            archive = DemoArchive::Named(name.to_owned());
            id = match self.prm.id {
                PRM_ID_UNK_1 => self
                    .prm
                    .s16_at(1)
                    .ok_or(DemoGap::UnknownParameter { id: PRM_ID_UNK_1 })?
                    as u16 as u32,
                PRM_ID_UNK_2 => self
                    .prm
                    .s16_at(2)
                    .ok_or(DemoGap::UnknownParameter { id: PRM_ID_UNK_2 })?
                    as u16 as u32,
                PRM_ID_UNK_4 => self
                    .prm
                    .u32_at(1)
                    .ok_or(DemoGap::UnknownParameter { id: PRM_ID_UNK_4 })?,
                PRM_ID_UNK_5 | PRM_ID_UNK_6 => self
                    .prm
                    .u32_at(2)
                    .ok_or(DemoGap::UnknownParameter { id: self.prm.id })?,
                other => return Err(DemoGap::UnknownParameter { id: other }),
            };
        }
        if id as i32 == self.btp_id {
            return Err(DemoGap::Unchanged);
        }
        self.btp_id = id as i32;
        if id & 0x10000 != 0 {
            archive = DemoArchive::Room;
        }
        Ok(AnmRef {
            archive,
            id: (id & 0xFFFF) as u16,
            frame_max: None,
        })
    }

    /// `getP_BrkData(const char*)` - `d_demo.cpp:88-103`, the second of the fork's three: its one
    /// arm is `ID_UNK_6` and a word at `+10`.
    pub fn p_brk_data(&mut self, name: &str) -> Result<AnmRef, DemoGap> {
        if self.check(ACTOR_ENABLE_UNK) == 0 {
            return Err(DemoGap::NoParameterRecord);
        }
        let id = match self.prm.id {
            PRM_ID_UNK_6 => self
                .prm
                .u32_at(10)
                .ok_or(DemoGap::UnknownParameter { id: PRM_ID_UNK_6 })?,
            other => return Err(DemoGap::UnknownParameter { id: other }),
        };
        if id as i32 == self.brk_id {
            return Err(DemoGap::Unchanged);
        }
        self.brk_id = id as i32;
        let archive = if id & 0x10000 != 0 {
            DemoArchive::Room
        } else {
            DemoArchive::Named(name.to_owned())
        };
        Ok(AnmRef {
            archive,
            id: (id & 0xFFFF) as u16,
            frame_max: None,
        })
    }

    /// `getP_BtkData(const char*)` - `d_demo.cpp:106-123`: a halfword at `+4` (`ID_UNK_2`) or a
    /// word at `+6` (`ID_UNK_5`, `ID_UNK_6`).
    pub fn p_btk_data(&mut self, name: &str) -> Result<AnmRef, DemoGap> {
        if self.check(ACTOR_ENABLE_UNK) == 0 {
            return Err(DemoGap::NoParameterRecord);
        }
        let id = match self.prm.id {
            PRM_ID_UNK_2 => self
                .prm
                .s16_at(4)
                .ok_or(DemoGap::UnknownParameter { id: PRM_ID_UNK_2 })?
                as u16 as u32,
            PRM_ID_UNK_5 | PRM_ID_UNK_6 => self
                .prm
                .u32_at(6)
                .ok_or(DemoGap::UnknownParameter { id: self.prm.id })?,
            other => return Err(DemoGap::UnknownParameter { id: other }),
        };
        if id as i32 == self.btk_id {
            return Err(DemoGap::Unchanged);
        }
        self.btk_id = id as i32;
        let archive = if id & 0x10000 != 0 {
            DemoArchive::Room
        } else {
            DemoArchive::Named(name.to_owned())
        };
        Ok(AnmRef {
            archive,
            id: (id & 0xFFFF) as u16,
            frame_max: None,
        })
    }
}

/// The numbers and vectors `dDemo_camera_c`'s getters read back out of the game's live view when
/// the demo has not overridden them (`d_demo.cpp:337-460`). The sim has no render view, so the
/// layer that owns one hands it in, and `None` is the reference's own `getView() == NULL` case
/// with its per-getter fallbacks (`:342`, `:356`, `:369`, `:384`, `:447`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewState {
    pub near: f32,
    pub far: f32,
    pub fovy: f32,
    pub aspect: f32,
    pub eye: Vec3,
    pub up: Vec3,
    pub target: Vec3,
    pub bank: f32,
}

/// `dDemo_camera_c` - `d_demo.h:155-206`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DemoCamera {
    flags: u8,
    proj_near: f32,
    proj_far: f32,
    fovy: f32,
    aspect: f32,
    view_position: Vec3,
    up_vector: Vec3,
    target_position: Vec3,
    roll: f32,
}

impl Default for DemoCamera {
    /// The constructor writes `mFlags = 0` and nothing else (`d_demo.h:168`); every other member
    /// is read only while its flag is up, so a zeroed member stands in for an untouched one.
    fn default() -> Self {
        Self {
            flags: 0,
            proj_near: 0.0,
            proj_far: 0.0,
            fovy: 0.0,
            aspect: 0.0,
            view_position: Vec3::ZERO,
            up_vector: Vec3::ZERO,
            target_position: Vec3::ZERO,
            roll: 0.0,
        }
    }
}

impl DemoCamera {
    pub fn check(&self, mask: u8) -> bool {
        self.flags & mask != 0
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    pub fn fovy(&self) -> f32 {
        self.fovy
    }

    pub fn roll(&self) -> f32 {
        self.roll
    }

    /// `getTarget()` - `d_demo.h:192`.
    pub fn target(&self) -> Vec3 {
        self.target_position
    }

    /// `getTrans()` - `d_demo.h:193`.
    pub fn trans(&self) -> Vec3 {
        self.view_position
    }

    /// `getUp()` - `d_demo.h:194`.
    pub fn up(&self) -> Vec3 {
        self.up_vector
    }

    /// `JSGSetProjectionNear(f32)` - `d_demo.cpp:346-349`.
    pub fn set_projection_near(&mut self, value: f32) {
        self.proj_near = value;
        self.flags |= CAMERA_ENABLE_PROJ_NEAR;
    }

    /// `JSGSetProjectionFar(f32)` - `d_demo.cpp:360-363`.
    pub fn set_projection_far(&mut self, value: f32) {
        self.proj_far = value;
        self.flags |= CAMERA_ENABLE_PROJ_FAR;
    }

    /// `JSGSetProjectionFovy(f32)` - `d_demo.cpp:374-377`.
    pub fn set_projection_fovy(&mut self, value: f32) {
        self.fovy = value;
        self.flags |= CAMERA_ENABLE_PROJ_FOVY;
    }

    /// `JSGSetProjectionAspect(f32)` - `d_demo.cpp:388-391`.
    pub fn set_projection_aspect(&mut self, value: f32) {
        self.aspect = value;
        self.flags |= CAMERA_ENABLE_PROJ_ASPECT;
    }

    /// `JSGSetViewPosition(const Vec&)` - `d_demo.cpp:405-408`.
    pub fn set_view_position(&mut self, value: Vec3) {
        self.view_position = value;
        self.flags |= CAMERA_ENABLE_VIEW_POS;
    }

    /// `JSGSetViewUpVector(const Vec&)` - `d_demo.cpp:422-425`.
    pub fn set_view_up_vector(&mut self, value: Vec3) {
        self.up_vector = value;
        self.flags |= CAMERA_ENABLE_VIEW_UP_VEC;
    }

    /// `JSGSetViewTargetPosition(const Vec&)` - `d_demo.cpp:439-442`.
    pub fn set_view_target_position(&mut self, value: Vec3) {
        self.target_position = value;
        self.flags |= CAMERA_ENABLE_VIEW_TARG_POS;
    }

    /// `JSGSetViewRoll(f32)` - `d_demo.cpp:457-460`.
    pub fn set_view_roll(&mut self, value: f32) {
        self.roll = value;
        self.flags |= CAMERA_ENABLE_VIEW_ROLL;
    }

    /// `JSGGetProjectionNear` - `d_demo.cpp:338-343`, `0.0f` for no view.
    pub fn projection_near(&self, view: Option<&ViewState>) -> f32 {
        view.map_or(0.0, |view| view.near)
    }

    /// `JSGGetProjectionFar` - `d_demo.cpp:352-357`, `1.0f` for no view.
    pub fn projection_far(&self, view: Option<&ViewState>) -> f32 {
        view.map_or(1.0, |view| view.far)
    }

    /// `JSGGetProjectionFovy` - `d_demo.cpp:366-371`, `60.0f` for no view.
    pub fn projection_fovy(&self, view: Option<&ViewState>) -> f32 {
        view.map_or(60.0, |view| view.fovy)
    }

    /// `JSGGetProjectionAspect` - `d_demo.cpp:380-385`, `4/3` for no view. The reference writes the
    /// fallback and then overwrites it when the view exists; `map_or` is that same read.
    pub fn projection_aspect(&self, view: Option<&ViewState>) -> f32 {
        view.map_or(4.0 / 3.0, |view| view.aspect)
    }

    /// `JSGGetViewPosition` - `d_demo.cpp:394-402`.
    pub fn view_position(&self, view: Option<&ViewState>) -> Vec3 {
        view.map_or(Vec3::ZERO, |view| view.eye)
    }

    /// `JSGGetViewUpVector` - `d_demo.cpp:411-419`.
    pub fn view_up_vector(&self, view: Option<&ViewState>) -> Vec3 {
        view.map_or(Vec3::Y, |view| view.up)
    }

    /// `JSGGetViewTargetPosition` - `d_demo.cpp:428-436`.
    pub fn view_target_position(&self, view: Option<&ViewState>) -> Vec3 {
        view.map_or(Vec3::Z, |view| view.target)
    }

    /// `JSGGetViewRoll` - `d_demo.cpp:445-454`, `0.0f` for no view.
    pub fn view_roll(&self, view: Option<&ViewState>) -> f32 {
        view.map_or(0.0, |view| sh2d(view.bank))
    }

    /// `store`'s demo camera block - `d_camera.cpp:7884-7901`, the one place the five view enables
    /// are read: target position, eye, up, roll (negated and converted) and fovy. The near and far
    /// bits are set by timelines but read nowhere in the shipped build, so they stay unused here
    /// rather than being applied somewhere they do not belong.
    pub fn apply_store(&self, view: &mut ViewState) {
        if self.check(CAMERA_ENABLE_VIEW_TARG_POS) {
            view.target = self.target_position;
        }
        if self.check(CAMERA_ENABLE_VIEW_POS) {
            view.eye = self.view_position;
        }
        if self.check(CAMERA_ENABLE_VIEW_UP_VEC) {
            view.up = self.up_vector;
        }
        if self.check(CAMERA_ENABLE_VIEW_ROLL) {
            view.bank = f32::from(-deg2s(self.roll));
        }
        if self.check(CAMERA_ENABLE_PROJ_FOVY) {
            view.fovy = self.fovy;
        }
    }
}

/// `dDemo_ambient_c` - `d_demo.h:208-223`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DemoAmbient {
    flags: u8,
    color: [u8; 4],
}

impl DemoAmbient {
    pub fn check(&self, mask: u8) -> bool {
        self.flags & mask != 0
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    pub fn color(&self) -> [u8; 4] {
        self.color
    }

    /// `JSGSetColor(GXColor)` - `d_demo.cpp:463-466`.
    pub fn set_color(&mut self, color: [u8; 4]) {
        self.color = color;
        self.flags |= AMBIENT_ENABLE_COLOR;
    }
}

/// `dDemo_light_c` - `d_demo.h:225-258`. `mLightType`, `mAttnFn` and `mSpotFn` keep the reference's
/// storage types; their values are consumed by a lighting layer the shell owns.
///
/// `mPosition` and `mDirection` are `Option`s because the reference never initialises them: the
/// constructor writes `mFlags = 0` and nothing else (`d_demo.h:238`), and `JSGSetPosition` /
/// `JSGSetDirection` (`d_demo.cpp:475-478`, `:502-505`) are the only writers, while the matching
/// getters are the empty `JStage::TLight` defaults (`JSGLight.cpp` `JSGGetPosition`,
/// `JSGGetDirection`). `None` is the port's state for a field nothing has written.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DemoLight {
    flags: u8,
    light_type: i32,
    attn_fn: i32,
    spot_fn: i32,
    position: Option<Vec3>,
    color: [u8; 4],
    dist_attn0: f32,
    dist_attn1: f32,
    angle_attn: f32,
    direction: Option<Vec3>,
}

impl DemoLight {
    pub fn check(&self, mask: u8) -> bool {
        self.flags & mask != 0
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    /// The read `TAdaptor_light::adaptor_do_begin` makes (`object-light.cpp:52`). Retail's is the
    /// empty `JStage::TLight` default, so the value the adaptor's triple ends up holding is an
    /// uninitialised stack slot; the port's light answers its own store, `None` until a
    /// `JSGSet*` has written it.
    pub fn jsg_get_position(&self) -> Option<Vec3> {
        self.position
    }

    /// `JSGGetDirection` - `object-light.cpp:53`, the same read as
    /// [`DemoLight::jsg_get_position`].
    pub fn jsg_get_direction(&self) -> Option<Vec3> {
        self.direction
    }

    pub fn position(&self) -> Option<Vec3> {
        self.position
    }

    pub fn color(&self) -> [u8; 4] {
        self.color
    }

    pub fn direction(&self) -> Option<Vec3> {
        self.direction
    }

    pub fn light_type(&self) -> i32 {
        self.light_type
    }

    pub fn attn_fn(&self) -> i32 {
        self.attn_fn
    }

    pub fn spot_fn(&self) -> i32 {
        self.spot_fn
    }

    pub fn dist_attn(&self) -> (f32, f32) {
        (self.dist_attn0, self.dist_attn1)
    }

    pub fn angle_attn(&self) -> f32 {
        self.angle_attn
    }

    /// `JSGSetLightType(TELight)` - `d_demo.cpp:469-472`.
    pub fn set_light_type(&mut self, value: i32) {
        self.light_type = value;
        self.flags |= LIGHT_ENABLE_LIGHT_TYPE;
    }

    /// `JSGSetPosition(const Vec&)` - `d_demo.cpp:475-478`. `None` is a write whose value is the
    /// adaptor's uninitialised read: the reference stores the triple it was handed either way.
    pub fn set_position(&mut self, value: Option<Vec3>) {
        self.position = value;
        self.flags |= LIGHT_ENABLE_POSITION;
    }

    /// `JSGSetColor(GXColor)` - `d_demo.cpp:481-484`.
    pub fn set_color(&mut self, value: [u8; 4]) {
        self.color = value;
        self.flags |= LIGHT_ENABLE_COLOR;
    }

    /// `JSGSetDistanceAttenuation(f32, f32, GXDistAttnFn)` - `d_demo.cpp:487-492`.
    pub fn set_distance_attenuation(&mut self, first: f32, second: f32, function: i32) {
        self.dist_attn0 = first;
        self.dist_attn1 = second;
        self.attn_fn = function;
        self.flags |= LIGHT_ENABLE_DIST_ATTEN;
    }

    /// `JSGSetAngleAttenuation(f32, GXSpotFn)` - `d_demo.cpp:495-499`.
    pub fn set_angle_attenuation(&mut self, angle: f32, function: i32) {
        self.angle_attn = angle;
        self.spot_fn = function;
        self.flags |= LIGHT_ENABLE_ANGLE_ATTEN;
    }

    /// `JSGSetDirection(const Vec&)` - `d_demo.cpp:502-505`, the same `None` as
    /// [`DemoLight::set_position`].
    pub fn set_direction(&mut self, value: Option<Vec3>) {
        self.direction = value;
        self.flags |= LIGHT_ENABLE_DIRECTION;
    }
}

/// `dDemo_fog_c` - `d_demo.h:260-284`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DemoFog {
    flags: u8,
    fog_type: u8,
    start_z: f32,
    end_z: f32,
    color: [u8; 4],
}

impl DemoFog {
    pub fn check(&self, mask: u8) -> bool {
        self.flags & mask != 0
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    pub fn fog_type(&self) -> u8 {
        self.fog_type
    }

    pub fn start_z(&self) -> f32 {
        self.start_z
    }

    pub fn end_z(&self) -> f32 {
        self.end_z
    }

    pub fn color(&self) -> [u8; 4] {
        self.color
    }

    /// `JSGSetFogFunction(GXFogType)` - `d_demo.cpp:508-511`.
    pub fn set_fog_function(&mut self, function: u8) {
        self.fog_type = function;
        self.flags |= FOG_ENABLE_FOG_FN;
    }

    /// `JSGSetStartZ(f32)` - `d_demo.cpp:514-517`.
    pub fn set_start_z(&mut self, value: f32) {
        self.start_z = value;
        self.flags |= FOG_ENABLE_START_Z;
    }

    /// `JSGSetEndZ(f32)` - `d_demo.cpp:520-523`.
    pub fn set_end_z(&mut self, value: f32) {
        self.end_z = value;
        self.flags |= FOG_ENABLE_END_Z;
    }

    /// `JSGSetColor(GXColor)` - `d_demo.cpp:526-529`.
    pub fn set_color(&mut self, color: [u8; 4]) {
        self.color = color;
        self.flags |= FOG_ENABLE_COLOR;
    }
}

/// `dDemo_object_c` - `d_demo.h:286-313`: the actor, camera, ambient, light and fog tables.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DemoObject {
    actors: Vec<DemoActor>,
    camera: Option<DemoCamera>,
    ambient: Option<DemoAmbient>,
    lights: Vec<DemoLight>,
    fog: Option<DemoFog>,
}

impl DemoObject {
    pub fn actor_count(&self) -> usize {
        self.actors.len()
    }

    pub fn light_count(&self) -> usize {
        self.lights.len()
    }

    /// `getActor(u8)` - `d_demo.cpp:564-569`: the id is the **one-based** `demoActorID`, so `0`
    /// and anything past the array answer NULL.
    pub fn get_actor(&self, id: u8) -> Option<&DemoActor> {
        if id == 0 || usize::from(id) > ACTOR_MAX {
            return None;
        }
        self.actors.get(usize::from(id) - 1)
    }

    pub fn get_actor_mut(&mut self, id: u8) -> Option<&mut DemoActor> {
        if id == 0 || usize::from(id) > ACTOR_MAX {
            return None;
        }
        self.actors.get_mut(usize::from(id) - 1)
    }

    /// `JSGFindNodeID` on the stage object a lookup addressed: `dDemo_actor_c` answers from its
    /// model (`d_demo.h:84-87`) and every other stage object from `JStage::TObject`'s `-1`
    /// (`JSGObject.cpp:50-53`).
    pub fn find_node_id(&self, object: DemoObjectRef, name: &[u8]) -> Option<u32> {
        match object {
            DemoObjectRef::Actor(id) => self.get_actor(id)?.find_node_id(name),
            _ => None,
        }
    }

    /// `JSGGetNodeTransformation` on the same stage object: only the actor's model carries a
    /// node array (`d_demo.h:88-92`); the base class answers `false` (`JSGObject.cpp:55-62`).
    pub fn node_transform(&self, object: DemoObjectRef, node: u32) -> Option<Mat4> {
        match object {
            DemoObjectRef::Actor(id) => self.get_actor(id)?.node_transform(node),
            _ => None,
        }
    }

    /// A slot with no actor bound yet, for the fast-create path where the entity is reserved by
    /// `Commands` in the same breath: the reference has the actor at once (`fopAcM_fastCreate`,
    /// `d_demo.cpp:651`) and the port has its id at once.
    pub fn push_actor(&mut self, entity: Option<Entity>) -> Result<u8, DemoGap> {
        if self.actors.len() >= ACTOR_MAX {
            return Err(DemoGap::ActorTableFull);
        }
        self.actors.push(DemoActor::new(entity));
        Ok(self.actors.len() as u8)
    }

    /// Binds a slot `push_actor` reserved to the entity the spawn produced, the second half of the
    /// reference's `appendActor` where `fAcM_fastCreate` already had the actor in hand
    /// (`d_demo.cpp:551-561`).
    pub fn bind_actor(&mut self, id: u8, entity: Entity) -> bool {
        match self.get_actor_mut(id) {
            Some(demo) => {
                demo.set_actor(Some(entity));
                true
            }
            None => false,
        }
    }

    /// `fopAc_Delete`'s `pDemoActor->setActor(NULL)` (`f_op_actor.cpp:245-247`), swept over the
    /// entities still present. A slot whose actor is gone keeps its place: retail's does too, and
    /// `daDemo00_c::execute` still finds it by id (`d_a_demo00.cpp:636`).
    pub fn forget_actors(&mut self, live: &[Entity]) {
        for demo in &mut self.actors {
            if let Some(entity) = demo.actor
                && !live.contains(&entity)
            {
                demo.set_actor(None);
            }
        }
    }

    /// `createCamera()` - `d_demo.cpp:572-578`: one camera, created on the first ask.
    pub fn create_camera(&mut self) -> &mut DemoCamera {
        self.camera.get_or_insert_with(DemoCamera::default)
    }

    /// `getCamera()` - `d_demo.h:300`.
    pub fn camera(&self) -> Option<&DemoCamera> {
        self.camera.as_ref()
    }

    /// `getActiveCamera()` - `d_demo.cpp:581-585`, which exists to answer the game's
    /// `dComIfGp_demo_getCamera()`; `store` is its only caller (`d_camera.cpp:7879`).
    pub fn active_camera(&self) -> Option<&DemoCamera> {
        self.camera.as_ref()
    }

    /// `createAmbient()` - `d_demo.cpp:588-594`.
    pub fn create_ambient(&mut self) -> &mut DemoAmbient {
        self.ambient.get_or_insert_with(DemoAmbient::default)
    }

    pub fn ambient(&self) -> Option<&DemoAmbient> {
        self.ambient.as_ref()
    }

    /// `appendLight()` - `d_demo.cpp:597-607`.
    pub fn append_light(&mut self) -> Result<&mut DemoLight, DemoGap> {
        if self.lights.len() >= LIGHT_MAX {
            return Err(DemoGap::ActorTableFull);
        }
        self.lights.push(DemoLight::default());
        Ok(self.lights.last_mut().expect("just pushed"))
    }

    pub fn lights(&self) -> &[DemoLight] {
        &self.lights
    }

    /// The mutable light table row `JSGFindObject(TOBJ_LIGHT)`'s `appendLight` index names
    /// (`d_demo.cpp:597-607`).
    pub fn light_mut(&mut self, index: usize) -> Option<&mut DemoLight> {
        self.lights.get_mut(index)
    }

    /// `createFog()` - `d_demo.cpp:610-616`.
    pub fn create_fog(&mut self) -> &mut DemoFog {
        self.fog.get_or_insert_with(DemoFog::default)
    }

    pub fn fog(&self) -> Option<&DemoFog> {
        self.fog.as_ref()
    }

    /// `remove()` - `d_demo.cpp:619-640`, which drops every table entry. Each demo actor's
    /// destructor clears its actor's `demoActorID` (`d_demo.cpp:26-29`), which is the sweep below.
    pub fn remove(&mut self, actors: &mut Query<(Entity, &mut Actor)>) {
        for demo in self.actors.drain(..) {
            if let Some(entity) = demo.actor
                && let Ok((_, mut actor)) = actors.get_mut(entity)
            {
                actor.demo_actor_id = 0;
            }
        }
        self.camera = None;
        self.ambient = None;
        self.lights.clear();
        self.fog = None;
    }
}

/// `JStage::TEObject`, the type argument of `JSGFindObject` (`d_demo.cpp:643`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    /// `TOBJ_ACTOR`, the arm that may fast-create.
    Actor,
    /// `TOBJ_ACTOR_UNK`, the arm that may not.
    ActorUnk,
    Camera,
    Ambient,
    Light,
    Fog,
}

/// A lookup's answer: the table slot a timeline writes into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoObjectRef {
    /// The one-based `demoActorID`.
    Actor(u8),
    Camera,
    Ambient,
    Light(usize),
    Fog,
}

impl std::fmt::Display for DemoObjectRef {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Actor(id) => write!(formatter, "actor #{id}"),
            Self::Camera => write!(formatter, "the camera"),
            Self::Ambient => write!(formatter, "the ambient light"),
            Self::Light(index) => write!(formatter, "light #{index}"),
            Self::Fog => write!(formatter, "the fog"),
        }
    }
}

/// One actor the shell has to build because its `spawn_actor` branch needs the whole world. The
/// `CREATER` staff's `fopAcM_create` (`d_event_data.cpp:560`) is the only source; `JSGFindObject`'s
/// own fast-create is the 32 `d_act*` shells and goes through `Commands` instead.
#[derive(Debug, Clone, PartialEq)]
pub struct DemoCreate {
    /// The stage-table spelling the port's spawn branch is keyed on (`dStage_objectNameInf::name`,
    /// which is what `dStage_searchName` looked up).
    pub name: String,
    pub parameters: u32,
    pub pos: Vec3,
    pub angle: [i16; 3],
    pub scale: Vec3,
    pub room: i8,
    pub set_id: u16,
    /// `dStage_objectNameInf::argument`, which `fopAcM_create` copies onto the actor
    /// (`d_stage.cpp:1308`).
    pub argument: u8,
    /// The demo actor slot the spawn has to bind, for the `JSGFindObject` fast-create path; `None`
    /// for the `CREATER` staff, whose `fopAcM_create` appends nothing to the demo table.
    pub demo_actor_id: Option<u8>,
}

/// The queue [`DemoCreate`]s wait in until the shell drains it, the same shape `kalle_demos` uses
/// for its runtime spawns.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct DemoCreates {
    pub list: Vec<DemoCreate>,
}

/// An item the demo's own parameter table asked for, which this port has no per-item function
/// table to run. `execItemGet(itemNo)` is `item_func_ptr[itemNo]()` (`d_item.cpp:534-536`) and
/// `d_a_demo00.cpp:715-736` is the table that names the ten ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DemoItemGet {
    pub item: u8,
}

/// Micro-requests the demo layer works out but cannot run: the shell drains this each step, the way
/// it drains `SimEvents`.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct DemoNotices {
    pub item_gets: Vec<DemoItemGet>,
}

/// One actor a lookup can address, resolved at step start: `fopAcM_searchFromName`'s table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageActorRow {
    pub entity: Entity,
    pub name: String,
    pub demo_actor_id: u8,
}

/// The stage one timeline step addresses: `dDemo_system_c` plus the sim handles its lookups need.
///
/// The reference's `JSGFindObject` reaches straight into the actor registry -
/// `fopAcM_searchFromName` reads it, `fopAcM_fastCreate` creates into it, and `appendActor` writes
/// the actor's `demoActorID`. A port system already holds the `Query` of its own, and a struct
/// cannot borrow one across the two lifetimes such a query carries, so this step carries the
/// *resolved* rows instead: `appendActor`'s write goes into `assigned`, which [`control`] applies
/// through its query once the timeline has returned, and a fast-create goes into [`DemoCreates`]
/// for the shell to spawn - the same two-phase shape the `CREATER` staff's `fopAcM_create` takes
/// (`d_event_data.cpp:560`).
pub struct DemoStage<'a> {
    pub object: &'a mut DemoObject,
    actors: &'a mut Vec<StageActorRow>,
    assigned: &'a mut Vec<(Entity, u8)>,
    pub table: &'a ActorTable,
    pub creates: &'a mut DemoCreates,
    /// `dComIfGp_roomControl_getStayNo()`, the room a fast-created actor is placed in.
    pub stay_room: i8,
    /// `d_demo_manager_c::create`'s `transform_setOrigin(offsetPos, offsetAngY)` (`d_demo.cpp:729-734`):
    /// the position and yaw the timeline's demo-coordinate writes are offset by, `None` for the
    /// reference's `transform_enable(false)`. A timeline is what applies them.
    pub origin: Option<(Vec3, f32)>,
    /// `dDemo_manager_c`'s sound factory's channel - `JStudio_JAudio::TCreateObject(mDoAud_zelAudio_c::getInterface())`
    /// (`d_demo.cpp:686`). A JSND command's fade-in starts a JAISoundID here and its fade-out stops
    /// one.
    pub events: &'a mut SimEvents,
    /// `dDemo_manager_c`'s `mMesgControl` as the port's message box sees it - the
    /// `JStudio_JMessage::TCreateObject(mMesgControl)` factory (`d_demo.cpp:690`) and the
    /// `dMesg_c` screen that polls the control (`d_mesg.cpp:2361-2445`).
    pub messages: &'a mut MsgBox,
    /// `dComIfG_play_c`'s message archive, the table a JMSG code is looked up in.
    pub msg_table: &'a MsgTable,
    /// `dPa_control_c::getEmitterManager()`, the JPA manager
    /// `JStudio_JParticle::TCreateObject` was handed (`d_demo.cpp:688`).
    pub particles: &'a mut Jpa,
}

impl DemoStage<'_> {
    /// `fopAcM_searchFromName(name, 0, 0)` - `f_op_actor_mng.cpp:4081-4103`: the first actor whose
    /// name-table row resolves to the same profile name and argument.
    ///
    /// The port's name table decodes the argument column but not the profile-name column, so
    /// "same profile name" is checked as "same name-table spelling". That is strictly stronger: an
    /// actor sharing a profile under a different spelling is missed, never mismatched.
    fn search_from_name(&mut self, name: &str) -> Option<usize> {
        let wanted = self.table.record_for_name(name)?;
        let spelling = wanted.name().to_owned();
        let argument = wanted.argument;
        self.actors.iter().position(|row| {
            let Some(other) = self.table.record_for_name(&row.name) else {
                return false;
            };
            other.name() == spelling && other.argument == argument
        })
    }

    /// `dDemo_object_c::appendActor` - `d_demo.cpp:545-562`: an actor that already has a live slot
    /// is handed back, otherwise a slot is added and the actor's `demoActorID` becomes the new
    /// one-based index. The write lands in `assigned` for [`control`] to apply.
    fn append_actor(&mut self, row: usize) -> Result<u8, DemoGap> {
        if self
            .object
            .get_actor(self.actors[row].demo_actor_id)
            .is_some()
        {
            return Ok(self.actors[row].demo_actor_id);
        }
        let entity = self.actors[row].entity;
        let id = self.object.push_actor(Some(entity))?;
        self.actors[row].demo_actor_id = id;
        self.assigned.push((entity, id));
        Ok(id)
    }

    /// `JSGFindObject(const char*, JStage::TEObject)` - `d_demo.cpp:642-671`.
    pub fn find_object(&mut self, name: &str, kind: ObjectKind) -> Result<DemoObjectRef, DemoGap> {
        match kind {
            ObjectKind::Actor | ObjectKind::ActorUnk => {
                if let Some(row) = self.search_from_name(name) {
                    return self.append_actor(row).map(DemoObjectRef::Actor);
                }
                if kind == ObjectKind::Actor && name.starts_with(DEMO_ACTOR_PREFIX) {
                    return self.fast_create_demo_actor(name);
                }
                Err(DemoGap::NoNameInfo {
                    name: name.to_owned(),
                })
            }
            ObjectKind::Camera => {
                self.object.create_camera();
                Ok(DemoObjectRef::Camera)
            }
            ObjectKind::Ambient => {
                self.object.create_ambient();
                Ok(DemoObjectRef::Ambient)
            }
            ObjectKind::Light => {
                let index = self.object.light_count();
                self.object.append_light()?;
                Ok(DemoObjectRef::Light(index))
            }
            ObjectKind::Fog => {
                self.object.create_fog();
                Ok(DemoObjectRef::Fog)
            }
        }
    }

    /// The `JStage::TObject` half a timeline's node lookups reach: `JSGFindNodeID(name)` on the
    /// object a `PARENT`/`RELATION` command resolved (`object-actor.cpp:152-166`,
    /// `object-particle.cpp:110-127`). `None` is the reference's `-1`, whether because the
    /// object's model has no such joint or because it has no model at all.
    pub fn find_node_id(&self, object: DemoObjectRef, name: &[u8]) -> Option<u32> {
        self.object.find_node_id(object, name)
    }

    /// `JSGGetNodeTransformation(no, mtx)` on the same object
    /// (`object-particle.cpp:248-259`); `None` is the reference's `false`.
    pub fn node_transform(&self, object: DemoObjectRef, node: u32) -> Option<Mat4> {
        self.object.node_transform(object, node)
    }

    /// The `strncmp(name, "d_act", 5)` arm - `d_demo.cpp:650-658`: `fopAcM_fastCreate(name, 0)`
    /// followed by `fopAcM_setStageLayer`. The name table's `d_act*` rows are the ones bound to
    /// `fpcNm_DEMO00_e`, which is why `demo00::is_demo00` is the gate. Position, angle and scale
    /// are the reference's defaults for a NULL triple: zero, zero and one
    /// (`f_op_actor_mng.cpp:105-127`), the last divided back out of `fopAcM_prm_class`'s times-ten
    /// scale (`:127-133`).
    ///
    /// `fopAcM_setStageLayer` (`f_op_actor_mng.cpp:35-39`) moves the new process onto the stage
    /// scene's layer; the port's registry has no per-layer queue and registers every fresh actor
    /// into the actor group on its create pass (`registry::create_pass`), so there is no second
    /// call to make here.
    fn fast_create_demo_actor(&mut self, name: &str) -> Result<DemoObjectRef, DemoGap> {
        let Some(row) = self.table.record_for_name(name) else {
            return Err(DemoGap::NoNameInfo {
                name: name.to_owned(),
            });
        };
        if !demo00::is_demo00(row.name()) && !demo00::is_demo00(name) {
            return Err(DemoGap::NoNameInfo {
                name: name.to_owned(),
            });
        }
        let id = self.object.push_actor(None)?;
        self.creates.list.push(DemoCreate {
            name: name.to_owned(),
            parameters: 0,
            pos: Vec3::ZERO,
            angle: [0; 3],
            scale: Vec3::ONE,
            room: self.stay_room,
            set_id: u16::MAX,
            argument: row.argument,
            demo_actor_id: Some(id),
        });
        Ok(DemoObjectRef::Actor(id))
    }

    /// `fopAcM_create`'s request, queued for the shell (`d_event_data.cpp:560`). A timeline's own
    /// create command reaches here too, which is `JStudio_JStage::TCreateObject`'s actor arm.
    pub fn request_create(&mut self, create: DemoCreate) {
        self.creates.list.push(create);
    }
}

/// `JStudio::TControl`, as far as this port can hold it: `forward(1)` and `isSuspended()`
/// (`d_demo.cpp:760-763`). A layer with an STB reader and a timeline player implements this and
/// installs it through [`DemoReaders`]; its `forward` is what `dDemo_manager_c::update` forwards,
/// and its `DemoStage` writes are what put demo actors and the demo camera together.
pub trait DemoTimeline: Send + Sync {
    /// The mount step: `JSGFindObject` for every block the file carries, then `forward(0)`.
    /// Retail resolves each block's ID *while parsing* (`JStudio_JStage/control.cpp:54-57`) and a
    /// lookup that answers NULL fails the whole file (`stb.cpp:403-421`), so the demo never
    /// mounts (`d_demo.cpp:723-727` leaves `mMode == 0`). The port's reader parses without a
    /// stage, so its resolution runs here, at the mount site, and an `Err` is that failed
    /// `create`.
    fn mount(&mut self, stage: &mut DemoStage<'_>) -> Result<(), DemoGap> {
        self.begin(stage);
        Ok(())
    }

    /// `mControl->forward(0)`: the create-time step (`d_demo.cpp:730`), which applies every
    /// paragraph up to the first wait. `create` has no stage, so the layer that mounts the file
    /// runs this at the mount site through [`DemoManager::start`]; like the reference, the answer
    /// is ignored.
    fn begin(&mut self, stage: &mut DemoStage<'_>) -> bool {
        self.forward(stage)
    }

    /// `mControl->forward(1)`. `true` while the timeline still has frames; `false` ends the demo
    /// (`d_demo.cpp:760-767`).
    fn forward(&mut self, stage: &mut DemoStage<'_>) -> bool;

    /// `mControl->isSuspended()` - `d_demo.cpp:762`, the gate on `mFrameNoMsg`.
    fn suspended(&self) -> bool {
        false
    }

    /// The per-command gaps recorded since the last call. Retail has no equivalent; this is the
    /// port's reporting surface where a command's value is unrepresentable.
    fn take_gaps(&mut self) -> Vec<DemoCommandGap> {
        Vec::new()
    }
}

/// The reader a mount needs: the file the stage archive handed over in, the timeline out. `None`
/// is a file this layer cannot read, which leaves the manager at [`MODE_NONE`].
pub type DemoReader = fn(&DemoFile) -> Option<Box<dyn DemoTimeline>>;

/// A demo file one of the two archives answered with - `dComIfG_getObjectRes(demo_arc, filename)`
/// or `dComIfG_getStageRes("Stage", filename)`, `d_event_data.cpp:796-802`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoFile {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// The fillable hook: whichever layer owns an STB reader installs one here and [`DemoManager::create`]
/// uses it. With no reader every mount is [`DemoGap::NoReader`] and the state machine stays at
/// [`MODE_NONE`], which is where retail's own `create` leaves a file whose `parse_next` failed
/// (`d_demo.cpp:723-727`).
#[derive(Resource, Default)]
pub struct DemoReaders {
    pub reader: Option<DemoReader>,
}

/// What the last mount did, for the shell to read and log. The manager's silent fields (mode,
/// frames) are left where the reference leaves them either way.
#[derive(Debug, Clone, PartialEq)]
pub struct DemoMount {
    pub name: String,
    pub outcome: Result<(), DemoGap>,
}

/// `dDemo_manager_c` - `d_demo.h:327-358`, `d_demo.cpp:673-769`.
///
/// `update` is the whole per-step surface: the reference calls it once a frame from
/// `dScnPly_Execute` *before* the event manager runs (`d_s_play.cpp:1100-1101`), which is the order
/// [`control`] keeps.
#[derive(Resource, Default)]
pub struct DemoManager {
    object: DemoObject,
    cur_file: Option<String>,
    frame: i32,
    frame_no_msg: i32,
    mode: i32,
    timeline: Option<Box<dyn DemoTimeline>>,
    /// `transform_enable(true)` / `transform_setOrigin(offsetPos, offsetAngY)` (`d_demo.cpp:729-734`):
    /// the position and yaw a timeline's demo-coordinate writes are offset by. The port carries the
    /// values; a timeline is what applies them.
    origin: Option<(Vec3, f32)>,
    /// `create` runs `mControl->forward(0)` itself (`d_demo.cpp:730`); the reader has no stage, so
    /// [`DemoManager::start`] runs that step at the mount site and marks it here so neither
    /// [`DemoManager::update`] nor [`control`] steps the timeline twice on the mount tick.
    started: bool,
    /// [`DemoCommandGap`]s the timeline recorded since the shell last read them.
    command_gaps: Vec<DemoCommandGap>,
    last_mount: Option<DemoMount>,
}

impl DemoManager {
    pub fn object(&self) -> &DemoObject {
        &self.object
    }

    /// `getObject()` - `d_demo.h:339`, the mutable side the shell's create drain binds a
    /// fast-created `d_act` slot through.
    pub fn object_mut(&mut self) -> &mut DemoObject {
        &mut self.object
    }

    /// `getMode()` - `d_demo.h:342`, the number `specialProcPackage` polls
    /// (`d_event_data.cpp:820-830`).
    pub fn mode(&self) -> i32 {
        self.mode
    }

    /// `getFrame()` - `d_demo.h:340`.
    pub fn frame(&self) -> i32 {
        self.frame
    }

    /// `getFrameNoMsg()` - `d_demo.h:341`, the count the `SOUND` staff's `NOMSG_WAIT` and
    /// `kytag05`/`kytag07` read (`d_event_data.cpp:480`, `d_a_kytag05.cpp:52`).
    pub fn frame_no_msg(&self) -> i32 {
        self.frame_no_msg
    }

    pub fn cur_file(&self) -> Option<&str> {
        self.cur_file.as_deref()
    }

    pub fn origin(&self) -> Option<(Vec3, f32)> {
        self.origin
    }

    /// The record of the last mount attempt, where a typed [`DemoGap`] reaches the layer that can
    /// fill it in.
    pub fn last_mount(&self) -> Option<&DemoMount> {
        self.last_mount.as_ref()
    }

    /// `dComIfGp_demo_getActor(id)` - `d_com_inf_game.h:3807-3809` over
    /// `dDemo_object_c::getActor` (`d_demo.cpp:564-569`). `demoActorID == 0` answers `None`, which
    /// is the test `daDemo00_c::execute` opens with (`d_a_demo00.cpp:636-639`).
    pub fn actor(&self, id: u8) -> Option<&DemoActor> {
        self.object.get_actor(id)
    }

    /// The same slot, mutably: where `daDemo00_c::actStandby`'s `act->setModel(mModel.mpModel)`
    /// (`d_a_demo00.cpp:296`) lands.
    pub fn actor_mut(&mut self, id: u8) -> Option<&mut DemoActor> {
        self.object.get_actor_mut(id)
    }

    /// `dComIfGp_demo_getCamera()` - `d_com_inf_game.h:3811-3813`.
    pub fn camera(&self) -> Option<&DemoCamera> {
        self.object.active_camera()
    }

    /// `create(const u8* demo_data, cXyz* offsetPos, f32 offsetAngleY)` - `d_demo.cpp:719-740`.
    ///
    /// The file's bytes go to [`DemoReaders`]. With no reader, or a reader that answers `None`,
    /// nothing is parsed: the manager keeps `mCurFile == NULL` and `mMode == 0` - the state the
    /// reference's own early return leaves (`:723-727`) - and the attempt is recorded for the
    /// layer. The reference's own `forward(0)` runs at the mount site, through
    /// [`DemoManager::start`], because the reader has no stage while parsing.
    pub fn create(
        &mut self,
        file: DemoFile,
        offset: Option<(Vec3, f32)>,
        readers: &DemoReaders,
    ) -> Result<(), DemoGap> {
        let parsed = match readers.reader {
            Some(reader) => reader(&file),
            None => {
                self.last_mount = Some(DemoMount {
                    name: file.name,
                    outcome: Err(DemoGap::NoReader),
                });
                return Err(DemoGap::NoReader);
            }
        };
        let outcome = match parsed {
            Some(timeline) => {
                self.timeline = Some(timeline);
                self.origin = offset;
                self.cur_file = Some(file.name.clone());
                self.frame = 0;
                self.frame_no_msg = 0;
                self.mode = MODE_PLAY;
                self.started = false;
                // The parse-time gaps - a malformed JFVB paragraph, a read past the file's
                // end - belong to this mount, not to a step, so they are drained before any
                // forward can add more.
                if let Some(timeline) = self.timeline.as_mut() {
                    self.command_gaps.extend(timeline.take_gaps());
                }
                Ok(())
            }
            None => Err(DemoGap::UnreadableFile),
        };
        self.last_mount = Some(DemoMount {
            name: file.name,
            outcome: outcome.clone(),
        });
        outcome
    }

    /// `create`'s `mControl->forward(0)` (`d_demo.cpp:730`): the mount site's half of `create`,
    /// run where a [`DemoStage`] exists. Retail runs the zero step inside `create`, before the
    /// mount tick's actor pass, so the mount site - the event staff that called
    /// [`DemoManager::create`] - is where it belongs.
    ///
    /// A resolution failure at this point is the reference's own failed `create` (`:723-727`):
    /// `mCurFile == NULL` and `mMode == 0`, with the demo object table left as the partial parse
    /// left it.
    pub fn start(
        &mut self,
        actors: &mut Query<(Entity, &mut Actor)>,
        table: Option<&ActorTable>,
        stay_room: i8,
        creates: &mut DemoCreates,
        events: &mut SimEvents,
        messages: &mut MsgBox,
        msg_table: &MsgTable,
        particles: &mut Jpa,
    ) -> Result<(), DemoGap> {
        if self.started || self.cur_file.is_none() {
            return Ok(());
        }
        let mut rows = actor_rows(actors);
        let empty = ActorTable::default();
        let table = table.unwrap_or(&empty);
        let mut assigned: Vec<(Entity, u8)> = Vec::new();
        let outcome;
        let gaps;
        {
            let Some(timeline) = self.timeline.as_mut() else {
                return Ok(());
            };
            let mut stage = DemoStage {
                object: &mut self.object,
                actors: &mut rows,
                assigned: &mut assigned,
                table,
                creates,
                stay_room,
                origin: self.origin,
                events,
                messages,
                msg_table,
                particles,
            };
            // `create` ignores `forward(0)`'s answer (`d_demo.cpp:730-732`) and keeps only the
            // parse's.
            outcome = timeline.mount(&mut stage);
            gaps = timeline.take_gaps();
        }
        self.started = true;
        self.command_gaps.extend(gaps);
        apply_assigned(actors, assigned);
        match outcome {
            Ok(()) => Ok(()),
            Err(gap) => {
                let name = self.cur_file.take().unwrap_or_default();
                self.timeline = None;
                self.origin = None;
                self.started = false;
                self.mode = MODE_NONE;
                self.last_mount = Some(DemoMount {
                    name,
                    outcome: Err(gap.clone()),
                });
                Err(gap)
            }
        }
    }

    /// The [`DemoCommandGap`]s recorded since the last call, for the shell to log.
    pub fn take_gaps(&mut self) -> Vec<DemoCommandGap> {
        std::mem::take(&mut self.command_gaps)
    }

    /// The `demo_data == NULL` leg of `specialProcPackage` (`d_event_data.cpp:796-803`), which the
    /// reference asserts on and the port records instead of mounting.
    pub fn mount_failed(&mut self, name: String, gap: DemoGap) {
        self.last_mount = Some(DemoMount {
            name,
            outcome: Err(gap),
        });
    }

    /// `remove()` - `d_demo.cpp:748-753`.
    pub fn remove(&mut self, actors: &mut Query<(Entity, &mut Actor)>) {
        self.object.remove(actors);
        self.cur_file = None;
        self.timeline = None;
        self.origin = None;
        self.started = false;
        self.mode = MODE_NONE;
    }

    /// `update()` - `d_demo.cpp:756-769`. `false` is the reference's "no file mounted" answer and
    /// nothing else; a mounted file whose timeline stops forwarding moves the mode to [`MODE_END`]
    /// and still answers `true`. A timeline the mount site did not get to start through
    /// [`DemoManager::start`] runs its mount step here, compacted with the first update the way
    /// retail's first `update` follows `create`.
    pub fn update(&mut self, stage: &mut DemoStage<'_>) -> bool {
        if self.cur_file.is_none() {
            return false;
        }
        let mut advanced = false;
        let mut failed = None;
        {
            let Some(timeline) = self.timeline.as_mut() else {
                return true;
            };
            if self.started {
                advanced = timeline.forward(stage);
            } else {
                self.started = true;
                match timeline.mount(stage) {
                    Ok(()) => advanced = timeline.forward(stage),
                    // The reference's failed `create` (`d_demo.cpp:723-727`).
                    Err(gap) => failed = Some(gap),
                }
            }
            if advanced {
                self.frame += 1;
                if !timeline.suspended() {
                    self.frame_no_msg += 1;
                }
            }
            self.command_gaps.extend(timeline.take_gaps());
        }
        if let Some(gap) = failed {
            let name = self.cur_file.take().unwrap_or_default();
            self.timeline = None;
            self.origin = None;
            self.started = false;
            self.mode = MODE_NONE;
            self.last_mount = Some(DemoMount {
                name,
                outcome: Err(gap),
            });
            return true;
        }
        if !advanced {
            self.mode = MODE_END;
        }
        true
    }
}

/// The per-step actor snapshot [`DemoStage`] works against - `fopAcM_searchFromName` reads the
/// same fields, resolved once per step start.
fn actor_rows(actors: &mut Query<(Entity, &mut Actor)>) -> Vec<StageActorRow> {
    actors
        .iter_mut()
        .map(|(entity, actor)| StageActorRow {
            entity,
            name: actor.name.clone(),
            demo_actor_id: actor.demo_actor_id,
        })
        .collect()
}

/// Applies the `demoActorID` writes a fast-create or `appendActor` left behind, through the query
/// the step borrowed.
fn apply_assigned(actors: &mut Query<(Entity, &mut Actor)>, assigned: Vec<(Entity, u8)>) {
    for (entity, id) in assigned {
        if let Ok((_, mut actor)) = actors.get_mut(entity) {
            actor.demo_actor_id = id;
        }
    }
}

/// The transform half of `dDemo_setDemoData` - `d_demo.cpp:195-204`, the part that runs before the
/// `i_morf == NULL` early return (`:206-207`).
///
/// `mask` is the caller's `i_flags`; the reference intersects it with the demo actor's own enables
/// and *truncates the result to a byte* (`u8 flags = demo_actor->checkEnable(i_flags)`, `:195`), so
/// a caller asking for a bit above `0xff` gets nothing from it. Returns the truncated mask.
pub fn apply_transform(actor: &mut Actor, demo: &DemoActor, mask: u16) -> u8 {
    let flags = demo.check(mask) as u8;
    if flags & ACTOR_ENABLE_TRANS as u8 != 0 {
        actor.old.pos = demo.trans();
        actor.pos = demo.trans();
    }
    if flags & ACTOR_ENABLE_ROTATE as u8 != 0 {
        actor.shape_angle = demo.rotate();
        actor.angle = demo.rotate();
    }
    if flags & ACTOR_ENABLE_SCALE as u8 != 0 {
        actor.scale = demo.scale();
    }
    flags
}

/// `dDemo_setDemoData(...)` - `d_demo.cpp:186-256`.
///
/// The transform writes land straight on the actor through [`apply_transform`]. The animation half
/// needs the caller's morph, which this port's actors own themselves, so it comes back as
/// [`DemoAnm`] for whoever owns the animation step instead of being written here; `morf` is the
/// reference's `i_morf == NULL` early return (`:206-207`), its second statement.
///
/// The animation id's `0x10000` bit selects the room's demo archive and the reference reads it as
/// `anmID & 0xFFFF` either way (`:215-224`); `arc_name` is the caller's own archive, `"Ba"` in
/// `daNpc_Ba1_c::demo` (`d_a_npc_ba1.cpp:1795`).
pub fn set_demo_data(
    actor: &mut Actor,
    demo: &mut DemoActor,
    mask: u16,
    morf: bool,
    arc_name: &str,
) -> DemoApplied {
    let flags = apply_transform(actor, demo, mask);
    if !morf {
        return DemoApplied::Transform;
    }
    let _ = arc_name;
    // `ENABLE_ANM_e` selects a new animation only when the id differs from the last one applied
    // (`:211-213`), and `setAnm`'s third argument is `getPrm_Morf()` (`:228-229`).
    let frame = (flags & ACTOR_ENABLE_ANM_FRAME as u8 != 0).then_some(demo.anm_frame());
    if flags & ACTOR_ENABLE_ANM as u8 != 0 && demo.anm_id() as i32 != demo.old_anm_id() {
        demo.set_old_anm_id(demo.anm_id());
        return DemoApplied::TransformAnm(DemoAnm {
            id: demo.anm_id(),
            transition: demo.prm_morf(),
            frame: frame.map(play_from),
        });
    }
    if flags & ACTOR_ENABLE_ANM_FRAME as u8 != 0 {
        // The frame half alone still plays the current animation (`:244-253`).
        return DemoApplied::TransformAnm(DemoAnm {
            id: demo.anm_id(),
            transition: demo.prm_morf(),
            frame: frame.map(play_from),
        });
    }
    DemoApplied::Transform
}

/// `if (anm_frame > 1.0f) anm_frame -= 1.0f;` - `d_demo.cpp:237-238`.
fn play_from(frame: f32) -> f32 {
    if frame > 1.0 { frame - 1.0 } else { frame }
}

/// What [`set_demo_data`] could do: the transform writes are the actor's own fields, the animation
/// is whoever owns the morph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DemoApplied {
    Transform,
    TransformAnm(DemoAnm),
}

/// The animation half of `dDemo_setDemoData`'s application. The key is
/// `setAnm(i_key, -1, f1, 1.0f, 0.0f, -1.0f, i_sound)`'s (`d_demo.cpp:229`): the
/// `dComIfG_getObjectIDRes(a_name, anmID & 0xFFFF)` the id names, the `getPrm_Morf()` transition,
/// and the optional frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DemoAnm {
    pub id: u32,
    pub transition: f32,
    pub frame: Option<f32>,
}

/// `dDemo_manager_c::update` once per fixed step, ahead of `event::control` the way
/// `dScnPly_Execute` runs `dComIfGp_demo_update` ahead of `dComIfGp_evmng_execute`
/// (`d_s_play.cpp:1100-1101`).
pub fn control(
    mut demo: ResMut<DemoManager>,
    mut actors: Query<(Entity, &mut Actor)>,
    table: Option<Res<ActorTable>>,
    stay: Res<StayRoom>,
    mut creates: ResMut<DemoCreates>,
    mut events: ResMut<SimEvents>,
    mut messages: ResMut<MsgBox>,
    msg_table: Res<MsgTable>,
    mut particles: ResMut<Jpa>,
) {
    // `update()`'s own first test (`d_demo.cpp:757-759`): with no file mounted the manager has
    // nothing to step, which also keeps the per-step actor sweep below off the frame.
    if demo.cur_file.is_none() {
        return;
    }
    let mut rows = actor_rows(&mut actors);
    let live: Vec<Entity> = rows.iter().map(|row| row.entity).collect();
    demo.object.forget_actors(&live);
    let empty = ActorTable::default();
    let table = table.as_deref().unwrap_or(&empty);
    let mut assigned: Vec<(Entity, u8)> = Vec::new();
    {
        let DemoManager {
            object,
            timeline,
            cur_file,
            frame,
            frame_no_msg,
            mode,
            origin,
            started,
            command_gaps,
            ..
        } = &mut *demo;
        if cur_file.is_some()
            && let Some(timeline) = timeline.as_mut()
        {
            let mut stage = DemoStage {
                object,
                actors: &mut rows,
                assigned: &mut assigned,
                table,
                creates: &mut creates,
                stay_room: stay.0,
                origin: *origin,
                events: &mut events,
                messages: &mut messages,
                msg_table: &msg_table,
                particles: &mut particles,
            };
            let advanced = if *started {
                timeline.forward(&mut stage)
            } else {
                // The mount site runs the create step through `DemoManager::start`; a mount that
                // reached here without one gets it now, compacted with the first update the way
                // retail's first `update` follows `create`'s own `forward(0)`.
                *started = true;
                let _ = timeline.begin(&mut stage);
                timeline.forward(&mut stage)
            };
            if advanced {
                *frame += 1;
                if !timeline.suspended() {
                    *frame_no_msg += 1;
                }
            } else {
                *mode = MODE_END;
            }
            command_gaps.extend(timeline.take_gaps());
        }
    }
    apply_assigned(&mut actors, assigned);
}

pub fn init(world: &mut World) {
    world.init_resource::<DemoManager>();
    world.init_resource::<DemoReaders>();
    world.init_resource::<DemoNotices>();
    world.init_resource::<RoomDemoArchive>();
    world.insert_resource(DemoCreates::default());
    world.resource_mut::<DemoReaders>().reader = Some(super::jstudio::read_timeline);
}
