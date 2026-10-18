//! The STB reader and `stb::TControl` timeline player - `JStudio::TParse`
//! (`JSystem/JStudio/JStudio/stb.cpp`, `stb-data-parse.cpp`, `jstudio-control.cpp`) plus the four
//! adaptor factories `dDemo_manager_c` mounts: the stage one (`JStudio_JStage/control.cpp`), the
//! message one (`JStudio_JMessage/control.cpp`), the sound one (`JStudio_JAudio/control.cpp`) and
//! the particle one (`JStudio_JParticle/control.cpp`).
//!
//! `dDemo_manager_c::create` parses the mounted file into a `JStudio::TControl` timeline and
//! forwards it zero frames (`d_demo.cpp:719-740`); `update` forwards it one frame a step
//! (`:756-769`). This module is the port's stack: [`Timeline::mount`] is `parse_next` plus the
//! create-time `forward(0)`, and [`Timeline::forward`] is `forward(1)`. The reader is installed by
//! `super::demo::init` and reaches a mount through `DemoReaders`.
//!
//! # The file
//!
//! A STB file is a 0x20-byte `data::THeader` - `'STB\0'`, byte order `0xFEFF`, version 1..3, the
//! `jstudio` target name at `0x10` and target version 2..3 (`stb.cpp:334-359`,
//! `jstudio-control.cpp:68-89`) - followed by `block_number` blocks. A block is a `data::TBlock`
//! (`size`, 32-bit type) in its object form, with a `flag` word and an ID (`stb-data.h:47-57`);
//! the factory resolves the ID through `JSGFindObject` and builds the matching adaptor
//! (`JStudio_JStage/control.cpp:26-60`). Block type `-1` is the control track: not an object but
//! the master sequence (`stb.cpp:386-390`), laid out with the same object header because
//! `parseBlock_object` reads it as one.
//!
//! Every pointer the reference's objects hold points into this one file buffer, so a read that
//! runs past a block, a sequence or a paragraph reads the bytes that follow it, and a jump may
//! land anywhere in the file (`stb.cpp:179-185` sets the pointer and never bounds it). The port
//! keeps the file and reads through absolute offsets for the same reason; only a read past the
//! buffer itself has no bytes at all.
//!
//! # The sequences
//!
//! Each block's content is a `TParse_TSequence` stream: a 32-bit head `[type:8][param:24]`
//! (`stb-data-parse.cpp:16-33`). Type 0 ends the stream, 1..0x7F are 4-byte operations - 1
//! raises or lowers the flag bits, 2 sets a wait, 3 jumps by a signed 24-bit offset, 4 suspends -
//! and `0x80` opens a paragraph region of `param` bytes. Paragraphs carry a `[size][type]` header
//! in either a 16-bit or 32-bit variable encoding (`stb-data-parse.cpp:36-48`); the reserved
//! types 1, 2 and 3 repeat flag/wait/jump, `0x80` carries an anonymous data record, `0x81` a
//! named one, and every type above `0xFF` is an adaptor command whose group is `type >> 5` and
//! operation `type & 0x1F` (`jstudio-object.cpp:289-415`).
//!
//! # The function-value bank
//!
//! A `JFVB` block is an FVB container (`jstudio-control.cpp:91-115`) parsed into the control's
//! `fvb::TControl` (`fvb.cpp:409-486`). Each FVB block is one function value: a composite, a
//! constant, a transition, a list, a list of parameters or a hermite curve (`fvb.cpp:376-396`),
//! whose paragraphs carry its data, its references to other bank entries and its range,
//! adjustment, outside and interpolation attributes (`fvb.cpp:22-163`). The STB side reaches the
//! bank through `adaptor_setVariableValue_FVR_NAME_`/`_INDEX_` (`jstudio-object.cpp:221-231`); the
//! variable keeps the bank index and evaluates it each frame the way `update_functionValue_`
//! does (`:65-68`).
//!
//! # The message, sound and particle factories
//!
//! `dDemo_manager_c` mounts all four factories side by side (`d_demo.cpp:686-702`). Their blocks
//! are ordinary stb objects, but their IDs are never looked up: the message factory hands back
//! its control, and the sound and particle factories create from the manager they were built
//! with (`JStudio_JMessage/control.cpp:31-49`, `JStudio_JAudio/control.cpp:30-42`,
//! `JStudio_JParticle/control.cpp:30-42`). The port routes their commands into the systems the
//! reference's adaptors write: `JMSG` into the message box (`super::msg`), `JSND` into
//! `SimEvents`' `Se`/`SeAt`/`Bgm`/`SeStop`, and `JPTC` into the JPA manager (`super::jpa`), where
//! a `PARTICLE` command's packed id resolves through the bank the way `createSimpleEmitterID`
//! resolves it (`JPAEmitterManager.cpp:52-78`) and a miss is reported as
//! [`DemoCommandGap::ParticleId`] over the manager's own [`super::jpa::JpaGap`].
//!
//! # What is not here
//!
//! * `JSND`'s five mix variables - `setDemoVolume`, `setDemoPan`, `setDemoPitch`,
//!   `setTempoProportion`, `setDemoFxmix` (`JStudio_JAudio/object-sound.cpp:50-57`, `:144-150`).
//!   The values are held in the adaptor's variables like retail's, but the port's sound channel
//!   is an id-keyed cue list with no `JAISound` to mix.
//! * The demo-cut model's animation half beyond the morph and its material passes: a node lookup
//!   reaches [`super::demo::DemoActor::find_node_id`] / [`super::demo::DemoActor::node_transform`],
//!   which answer from the node table `daDemo00_c::actStandby`'s model gives
//!   (`d_a_demo00.cpp:296-313`) as [`super::demo::RoomDemoArchive`] reads it out of the room's
//!   demo archive, with the morph `createHeap` builds around that model (`:223-253`) posing the
//!   joints at the frame `actPerformance` sets (`:412-452`) and the btp, btk and brk keys the same
//!   `createHeap` inits (`:173-221`) writing the material slots their entries name. A staged key
//!   the archive does not carry is retail's own early return, and one it carries but the port
//!   cannot read as that key is [`super::demo::DemoGap::UnreadablePass`]. A lookup in a cut whose
//!   actor has no model is reported as [`DemoCommandGap::NodeName`] / [`DemoCommandGap::NodeTransform`]
//!   against the object it ran in.
//! * The adaptor destructors' releases - `~TAdaptor_sound`'s `mpSound->stop(0)`
//!   (`JStudio_JAudio/object-sound.cpp:25-29`) and `~TAdaptor_particle`'s
//!   `forceDeleteEmitter` (`JStudio_JParticle/object-particle.cpp:31-36`). `d_demo.cpp`'s
//!   `remove` runs them through `mControl->destroyObject_all()` (`:748-753`), but the port's
//!   [`super::demo::DemoManager::remove`] drops the timeline without a [`DemoStage`], so a sound
//!   or emitter still alive at removal is left to the shell's own channels.

use glam::{Mat4, Quat, Vec3};

use super::demo::{
    DemoCamera, DemoCommandGap, DemoFile, DemoFog, DemoGap, DemoObjectRef, DemoStage, DemoTimeline,
    ObjectKind, SECONDS_PER_FRAME,
};
use super::events::SimEvent;
use super::jpa::Jpa;

/// `data::THeader`'s size - `stb-data.h:31-45`: the next byte is the first block.
const HEADER_SIZE: usize = 0x20;

/// `data::BLOCK_NONE` - `stb-data.h:20`: the control track's block type, `-1` in the header.
const BLOCK_NONE: u32 = 0xFFFF_FFFF;
const BLOCK_FVB: u32 = u32::from_be_bytes(*b"JFVB");
const BLOCK_ACTOR: u32 = u32::from_be_bytes(*b"JACT");
const BLOCK_AMBIENT: u32 = u32::from_be_bytes(*b"JABL");
const BLOCK_CAMERA: u32 = u32::from_be_bytes(*b"JCMR");
const BLOCK_FOG: u32 = u32::from_be_bytes(*b"JFOG");
const BLOCK_LIGHT: u32 = u32::from_be_bytes(*b"JLIT");
const BLOCK_MESSAGE: u32 = u32::from_be_bytes(*b"JMSG");
const BLOCK_PARTICLE: u32 = u32::from_be_bytes(*b"JPTC");
const BLOCK_SOUND: u32 = u32::from_be_bytes(*b"JSND");

/// `JAISoundID_TypeMask` - `JAISound.h`: the top two bits of a sound id, non-zero for the
/// sequence (`0x80000000`) and stream (`0xC0000000`) families. `JAIBasic::checkEnablePrepare` is
/// exactly this mask (`JAIBasic.h`).
const SOUND_ID_TYPE_MASK: u32 = 0xC000_0000;
/// The bits `TVVOOn_BEGIN_FADE_IN_` tests before it stops a previous SE handle
/// (`JStudio_JAudio/object-sound.cpp:122`).
const SOUND_ID_STOP_MASK: u32 = 0x0000_0C00;

/// The reader [`super::demo::DemoReaders`] wants: a file in, a timeline out, `None` for a file
/// this layer cannot read.
pub fn read_timeline(file: &DemoFile) -> Option<Box<dyn DemoTimeline>> {
    Timeline::parse(&file.bytes).map(|timeline| Box::new(timeline) as Box<dyn DemoTimeline>)
}

/// What a block's type says its adaptor is. The stb factory's five stage types resolve a stage
/// object by ID; the three channel types do not (`JStudio_JMessage/control.cpp:31-49`,
/// `JStudio_JAudio/control.cpp:30-42`, `JStudio_JParticle/control.cpp:30-42`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Actor,
    Camera,
    Ambient,
    Light,
    Fog,
    Message,
    Sound,
    Particle,
}

impl Kind {
    /// The `JSGFindObject` type a block's ID is resolved with, `None` for the channel blocks.
    fn stage(self) -> Option<ObjectKind> {
        match self {
            Self::Actor => Some(ObjectKind::Actor),
            Self::Camera => Some(ObjectKind::Camera),
            Self::Ambient => Some(ObjectKind::Ambient),
            Self::Light => Some(ObjectKind::Light),
            Self::Fog => Some(ObjectKind::Fog),
            Self::Message | Self::Sound | Self::Particle => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Actor => "actor",
            Self::Camera => "camera",
            Self::Ambient => "ambient",
            Self::Light => "light",
            Self::Fog => "fog",
            Self::Message => "message",
            Self::Sound => "sound",
            Self::Particle => "particle",
        }
    }
}

/// `stb::TObject::TEStatus` - `stb.h:33-39`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Still,
    End,
    Wait,
    Suspend,
    Inactive,
}

/// The `stb::TObject` machinery every block content runs: its flag word, its sequence cursor, wait
/// and suspend count (`stb.h:96-108`, `stb.cpp:49-136`).
///
/// `sequence` and `sequence_next` are absolute offsets into the demo file, which is what the
/// reference's pointers are: `pSequence_next` starts at the block content and every jump is a
/// signed offset added to the current head (`stb.h:75-78`).
#[derive(Debug, Clone, PartialEq)]
struct SequenceState {
    flag: u16,
    b_sequence: bool,
    status: Status,
    sequence: Option<usize>,
    sequence_next: Option<usize>,
    wait: u32,
    suspend: u32,
}

impl SequenceState {
    fn new(content: usize) -> Self {
        Self {
            flag: 0,
            b_sequence: false,
            status: Status::Still,
            sequence: None,
            sequence_next: Some(content),
            wait: 0,
            suspend: 0,
        }
    }
}

/// One variable of a `TVariableValue` array (`jstudio-object.h:42-122`). `frames` is
/// `field_0x4`, the frame counter `forward` feeds; `update` is the pending `field_0x8` updater,
/// which the immediate form clears once it fires and the time and function forms keep.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Variable {
    value: f32,
    frames: u32,
    update: Option<ValueUpdate>,
    /// Whether anything has put a value here. A read that answers with nothing - the light
    /// adaptor's `JSGGetPosition`/`JSGGetDirection` against the empty `JStage` defaults
    /// (`object-light.cpp:52-53`) - leaves it clear, and a write built on such a read is the
    /// absent value rather than a stand-in.
    defined: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ValueUpdate {
    /// `update_immediate_` - `jstudio-object.cpp:55-58`: one shot.
    Immediate(f32),
    /// `update_time_` - `:61-63`: `value * (frames * secondPerFrame)` every step.
    Time(f32),
    /// `update_functionValue_` - `:65-68`: `fv->getValue(frames * secondPerFrame)` every step.
    Function(usize),
}

impl Variable {
    const NONE: Self = Self {
        value: 0.0,
        frames: 0,
        update: None,
        defined: false,
    };

    /// `setValue_none` - `jstudio-object.h:70-72`: drops the updater, keeps value and counter.
    fn set_none(&mut self) {
        self.update = None;
    }

    /// `setValue_immediate` - `jstudio-object.h:64-68`.
    fn set_immediate(&mut self, value: f32) {
        self.frames = 0;
        self.update = Some(ValueUpdate::Immediate(value));
        self.defined = true;
    }

    /// `setValue_time` - `jstudio-object.h:74-78`.
    fn set_time(&mut self, value: f32) {
        self.frames = 0;
        self.update = Some(ValueUpdate::Time(value));
        self.defined = true;
    }

    /// `setValue_functionValue` - `jstudio-object.h:80-84`.
    fn set_function_value(&mut self, index: usize) {
        self.frames = 0;
        self.update = Some(ValueUpdate::Function(index));
        self.defined = true;
    }

    /// `TVariableValue::forward` - `jstudio-object.h:100-106`, the saturating frame counter.
    fn forward(&mut self, frames: u32) {
        self.frames = self.frames.saturating_add(frames);
    }

    /// `TVariableValue::update` - `jstudio-object.cpp:47-52`: the updater runs, then the output
    /// callback is called with the new value. The caller needs to know whether it fired. A
    /// function value only fails to evaluate when the bank entry disappeared, which the resolve
    /// in [`set_variable`] already refuses to install.
    fn update(&mut self, fvb: &[FvbObject], second_per_frame: f64) -> bool {
        match self.update {
            Some(ValueUpdate::Immediate(value)) => {
                self.value = value;
                self.update = None;
                true
            }
            Some(ValueUpdate::Time(value)) => {
                self.value =
                    (f64::from(value) * (f64::from(self.frames) * second_per_frame)) as f32;
                true
            }
            Some(ValueUpdate::Function(index)) => {
                let argument = f64::from(self.frames) * second_per_frame;
                if let Some(object) = fvb.get(index) {
                    self.value = object.value.eval(fvb, argument) as f32;
                }
                true
            }
            None => false,
        }
    }

    /// The value as one component of a triple, absent while nothing has written it.
    fn component(&self) -> Option<f32> {
        self.defined.then_some(self.value)
    }
}

/// One function value in the FVB bank.
#[derive(Debug, Clone, PartialEq)]
struct FvbObject {
    /// `object::TObject_ID` - `fvb.h:26-47`: the bytes and size `TControl::getObject` compares
    /// (`object-id.cpp:12-55`), and the key `getObject_index` walks by position.
    id: Vec<u8>,
    value: FunctionValue,
}

/// `TFunctionValue`'s six concrete forms - `functionvalue.h:158-519`, built by
/// `fvb::TFactory::create` from the block type (`fvb.cpp:376-396`).
#[derive(Debug, Clone, PartialEq)]
enum FunctionValue {
    /// Type 1 - `TFunctionValue_composite` (`functionvalue.h:174-231`). `operation` is the
    /// factory's `pfn_`: `None` is the NULL a block with no data paragraph leaves, and retail
    /// would call it as a function pointer.
    Composite {
        operation: Option<CompositeOp>,
        data: CompositeData,
        refer: Vec<usize>,
    },
    /// Type 2 - `TFunctionValue_constant` (`functionvalue.h:158-172`).
    Constant(f64),
    /// Type 3 - `TFunctionValue_transition` (`functionvalue.h:233-255`).
    Transition {
        begin_value: f64,
        end_value: f64,
        range: Range,
        interpolate: u32,
    },
    /// Type 4 - `TFunctionValue_list` (`functionvalue.h:257-305`).
    List {
        data: Vec<f32>,
        interval: f64,
        range: Range,
        interpolate: u32,
        update: Interpolation,
    },
    /// Type 5 - `TFunctionValue_list_parameter` (`functionvalue.h:307-411`).
    ListParameter {
        data: Vec<f32>,
        range: Range,
        interpolate: u32,
        update: Interpolation,
    },
    /// Type 6 - `TFunctionValue_hermite` (`functionvalue.h:413-519`).
    Hermite {
        data: Vec<f32>,
        count: u32,
        size: u32,
        range: Range,
    },
}

/// `TFunctionValue_composite`'s operation, `data::TEComposite` minus the table's `COMPOSITE_NONE`
/// row, which is a NULL call in retail (`fvb.cpp:202-216`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompositeOp {
    Raw,
    Index,
    Parameter,
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// `TFunctionValue_composite::TData` (`functionvalue.h:176-194`): the union the operation's data
/// word is read into. The getter each operation's row uses decides which arm it is.
#[derive(Debug, Clone, Copy, PartialEq)]
enum CompositeData {
    /// `getCompositeData_raw_` / `getCompositeData_index_`: the 32-bit word, read as an index.
    Word(u32),
    /// `getCompositeData_parameter_` and the four arithmetic getters: `*(f32*)` promoted to f64.
    Value(f64),
}

/// `TFunctionValueAttribute_range` - `functionvalue.h:93-143`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Range {
    begin: f64,
    end: f64,
    difference: f64,
    progress: u32,
    adjust: u32,
    outside_begin: u32,
    outside_end: u32,
    /// `_20` and `_28`, the progress map `range_prepare` fills in (`functionvalue.cpp:159-188`).
    base: f64,
    scale: f64,
}

impl Range {
    /// The constructor's `NAN` sweep (`functionvalue.cpp:230-234`): every numeric field starts
    /// undefined, the four enums at 0.
    const NAN: Self = Self {
        begin: f64::NAN,
        end: f64::NAN,
        difference: f64::NAN,
        progress: 0,
        adjust: 0,
        outside_begin: 0,
        outside_end: 0,
        base: f64::NAN,
        scale: f64::NAN,
    };

    /// `range_set` - `functionvalue.cpp:190-197`.
    fn set(&mut self, begin: f64, end: f64) {
        self.begin = begin;
        self.end = end;
        self.difference = end - begin;
    }

    /// `range_prepare` - `functionvalue.cpp:159-188`.
    fn prepare(&mut self) {
        let (base, scale) = match self.progress {
            1 => (0.0, -1.0),
            2 => (self.begin, -1.0),
            3 => (self.end, -1.0),
            4 => (0.5 * (self.begin + self.end), -1.0),
            _ => (0.0, 1.0),
        };
        self.base = base;
        self.scale = scale;
    }

    /// `range_getParameter_progress` - `functionvalue.h:126`.
    fn parameter_progress(&self, value: f64) -> f64 {
        self.base + self.scale * (value - self.base)
    }

    /// `range_getParameter_outside` - `functionvalue.h:115-125`.
    fn parameter_outside(&self, value: f64) -> f64 {
        let mut result = value - self.begin;
        if result < 0.0 {
            result = outside_function(self.outside_begin)(result, self.difference);
        } else if result >= self.difference {
            result = outside_function(self.outside_end)(result, self.difference);
        }
        result + self.begin
    }

    /// `range_getParameter` - `functionvalue.cpp:199-228`, whose `default` arm is
    /// `ADJ_INIT`'s body rather than a fallthrough.
    fn parameter(&self, value: f64, low: f64, high: f64) -> f64 {
        let progress = self.parameter_progress(value);
        match self.adjust {
            1 => self.parameter_outside(progress + self.begin),
            2 => self.parameter_outside(progress + self.end),
            3 => self.parameter_outside(progress + 0.5 * (self.begin + self.end)),
            4 => {
                let outside = self.parameter_outside(progress);
                low + ((outside - self.begin) * (high - low)) / self.difference
            }
            _ => self.parameter_outside(progress),
        }
    }
}

/// The interpolation each list-like value runs between two keys - `pfnUpdate_`
/// (`functionvalue.cpp:532-557`, `:684-710`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Interpolation {
    None,
    Linear,
    Plateau,
    BSpline,
}

impl FunctionValue {
    fn composite() -> Self {
        Self::Composite {
            operation: None,
            data: CompositeData::Word(0),
            refer: Vec::new(),
        }
    }

    fn transition() -> Self {
        Self::Transition {
            begin_value: f64::NAN,
            end_value: f64::NAN,
            range: Range::NAN,
            interpolate: 0,
        }
    }

    fn list() -> Self {
        Self::List {
            data: Vec::new(),
            interval: f64::NAN,
            range: Range::NAN,
            interpolate: 0,
            update: Interpolation::None,
        }
    }

    fn list_parameter() -> Self {
        Self::ListParameter {
            data: Vec::new(),
            range: Range::NAN,
            interpolate: 0,
            update: Interpolation::None,
        }
    }

    fn hermite() -> Self {
        Self::Hermite {
            data: Vec::new(),
            count: 0,
            size: 0,
            range: Range::NAN,
        }
    }

    fn range_mut(&mut self) -> Option<&mut Range> {
        match self {
            Self::Transition { range, .. }
            | Self::List { range, .. }
            | Self::ListParameter { range, .. }
            | Self::Hermite { range, .. } => Some(range),
            Self::Composite { .. } | Self::Constant(_) => None,
        }
    }

    fn refer_mut(&mut self) -> Option<&mut Vec<usize>> {
        match self {
            Self::Composite { refer, .. } => Some(refer),
            _ => None,
        }
    }

    /// `TFunctionValue_*::prepare` - `functionvalue.cpp:257-258`, `:429-431`, `:460-464`,
    /// `:531-557`, `:684-710`, `:868-871`.
    fn prepare(&mut self) {
        match self {
            Self::Transition { range, .. } | Self::Hermite { range, .. } => range.prepare(),
            Self::List {
                data,
                range,
                interpolate,
                update,
                ..
            } => {
                range.prepare();
                *update = match *interpolate {
                    1 => Interpolation::Linear,
                    2 => Interpolation::Plateau,
                    // A three-point spline needs three points; two fall back to a line.
                    3 if data.len() == 2 => Interpolation::Linear,
                    3 => Interpolation::BSpline,
                    _ => Interpolation::None,
                };
            }
            Self::ListParameter {
                data,
                range,
                interpolate,
                update,
                ..
            } => {
                range.prepare();
                *update = match *interpolate {
                    1 => Interpolation::Linear,
                    2 => Interpolation::Plateau,
                    3 => Interpolation::BSpline,
                    _ => Interpolation::None,
                };
                if *interpolate == 3 && data.len() == 4 {
                    *update = Interpolation::Linear;
                }
            }
            Self::Composite { .. } | Self::Constant(_) => {}
        }
    }

    /// `TFunctionValue_*::getValue` - `functionvalue.cpp:260-267`, `:433-436`, `:466-505`,
    /// `:559-601`, `:712-731`, `:873-893`.
    fn eval(&self, bank: &[FvbObject], argument: f64) -> f64 {
        match self {
            Self::Constant(value) => *value,
            Self::Composite {
                operation,
                data,
                refer,
            } => {
                let Some(operation) = *operation else {
                    // `pfn_` is NULL - a composite block whose data paragraph never ran.
                    // Retail calls it as a function pointer; the port has no value to give.
                    return 0.0;
                };
                let values: Vec<&FunctionValue> = refer
                    .iter()
                    .filter_map(|index| bank.get(*index).map(|object| &object.value))
                    .collect();
                eval_composite(operation, *data, &values, bank, argument)
            }
            Self::Transition {
                begin_value,
                end_value,
                range,
                interpolate,
            } => eval_transition(*begin_value, *end_value, range, *interpolate, argument),
            Self::List {
                data,
                interval,
                range,
                update,
                ..
            } => eval_list(data, *interval, range, *update, argument),
            Self::ListParameter {
                data,
                range,
                update,
                ..
            } => eval_list_parameter(data, range, *update, argument),
            Self::Hermite {
                data,
                count,
                size,
                range,
            } => eval_hermite(data, *count, *size, range, argument),
        }
    }
}

/// `TFunctionValue_transition::getValue` - `functionvalue.cpp:466-505`.
fn eval_transition(
    begin_value: f64,
    end_value: f64,
    range: &Range,
    interpolate: u32,
    argument: f64,
) -> f64 {
    let progress = range.parameter_progress(argument);
    let outside = range.parameter_outside(progress);
    let split = || {
        if outside < 0.5 * (range.begin + range.end) {
            begin_value
        } else {
            end_value
        }
    };
    match range.adjust {
        2 => {
            if outside < range.end {
                begin_value
            } else {
                end_value
            }
        }
        3 => split(),
        4 => {
            if outside < range.begin {
                return begin_value;
            }
            if outside >= range.end {
                return end_value;
            }
            match interpolate {
                1 | 3 => {
                    begin_value
                        + ((outside - range.begin) * (end_value - begin_value)) / range.difference
                }
                2 => interpolate_value_plateau(
                    outside,
                    range.begin,
                    begin_value,
                    range.end,
                    end_value,
                ),
                _ => split(),
            }
        }
        _ => {
            if outside < range.begin {
                begin_value
            } else {
                end_value
            }
        }
    }
}

/// `TFunctionValue_composite`'s seven operations - `functionvalue.cpp:269-409`.
fn eval_composite(
    operation: CompositeOp,
    data: CompositeData,
    values: &[&FunctionValue],
    bank: &[FvbObject],
    argument: f64,
) -> f64 {
    let word = match data {
        CompositeData::Word(word) => word,
        CompositeData::Value(_) => 0,
    };
    let number = match data {
        CompositeData::Value(value) => value,
        CompositeData::Word(_) => 0.0,
    };
    match operation {
        // `composite_raw` reads the same 32-bit word as an index.
        CompositeOp::Raw => match values.get(word as usize) {
            Some(value) => value.eval(bank, argument),
            None => 0.0,
        },
        CompositeOp::Index => composite_index(values, word, bank, argument),
        CompositeOp::Parameter => {
            let mut result = argument - number;
            for value in values {
                result = value.eval(bank, result);
            }
            result
        }
        CompositeOp::Add => {
            let mut result = number;
            for value in values {
                result += value.eval(bank, argument);
            }
            result
        }
        CompositeOp::Subtract => {
            let Some(first) = values.first() else {
                return 0.0;
            };
            let mut result = first.eval(bank, argument);
            for value in values {
                result -= value.eval(bank, argument);
            }
            result - number
        }
        CompositeOp::Multiply => {
            let mut result = number;
            for value in values {
                result *= value.eval(bank, argument);
            }
            result
        }
        CompositeOp::Divide => {
            let Some(first) = values.first() else {
                return 0.0;
            };
            let mut result = first.eval(bank, argument);
            for value in values {
                result /= value.eval(bank, argument);
            }
            result / number
        }
    }
}

/// `composite_index` - `functionvalue.cpp:283-333`.
fn composite_index(values: &[&FunctionValue], word: u32, bank: &[FvbObject], argument: f64) -> f64 {
    let size = values.len() as i32;
    if size <= 1 {
        return 0.0;
    }
    let mut index = values[0].eval(bank, argument).floor() as i32;
    match word {
        1 => {
            index %= size - 1;
            if index < 0 {
                index = size + index - 1;
            }
        }
        2 => {
            if size - 1 == 1 {
                index = 0;
            } else {
                let twice = (size - 2) * 2;
                index %= twice;
                if index < 0 {
                    index += twice;
                }
                if index >= size - 1 {
                    index = twice - index;
                }
            }
        }
        _ => {
            if index < 0 {
                index = 0;
            } else if index >= size - 1 {
                index = size - 2;
            }
        }
    }
    match values.get((index + 1) as usize) {
        Some(value) => value.eval(bank, argument),
        None => 0.0,
    }
}

/// `TFunctionValue_list::getValue` - `functionvalue.cpp:559-601`.
fn eval_list(
    data: &[f32],
    interval: f64,
    range: &Range,
    update: Interpolation,
    argument: f64,
) -> f64 {
    if data.is_empty() {
        // `getValue` reads `_44[0]` straight; a value with no data is retail's NULL dereference.
        return 0.0;
    }
    let last = (data.len() - 1) as f64;
    let progress = range.parameter_progress(argument);
    let scaled = match range.adjust {
        1 => range.parameter_outside(progress + range.begin) / interval,
        2 => range.parameter_outside(progress + range.end) / interval,
        3 => range.parameter_outside(progress + 0.5 * (range.begin + range.end)) / interval,
        4 => {
            let outside = range.parameter_outside(progress);
            (last * (outside - range.begin)) / range.difference
        }
        _ => range.parameter_outside(progress) / interval,
    };
    if scaled < 0.0 {
        return f64::from(data[0]);
    }
    if scaled >= last {
        return f64::from(data[data.len() - 1]);
    }
    let base = scaled.floor();
    let index = base as usize;
    let value = |index: usize| data.get(index).copied().map_or(f64::NAN, f64::from);
    match update {
        Interpolation::None => value(index),
        Interpolation::Linear => {
            interpolate_value_linear_1(scaled, base, value(index), value(index + 1))
        }
        Interpolation::Plateau => {
            interpolate_value_plateau(scaled, base, value(index), 1.0 + base, value(index + 1))
        }
        Interpolation::BSpline => {
            let current = value(index);
            let next = value(index + 1);
            let (left, right) = if index == 0 {
                (2.0 * current - next, value(index + 2))
            } else if index == data.len() - 2 {
                (value(index - 1), 2.0 * next - current)
            } else {
                (value(index - 1), value(index + 2))
            };
            interpolate_value_bspline_uniform(scaled - base, left, current, next, right)
        }
    }
}

/// `TFunctionValue_list_parameter::getValue` - `functionvalue.cpp:712-731`, over the
/// `(time, value)` pairs `data_set` splits the array into (`functionvalue.cpp:656-666`).
fn eval_list_parameter(data: &[f32], range: &Range, update: Interpolation, argument: f64) -> f64 {
    let pairs = data.len() / 2;
    if pairs == 0 {
        return 0.0;
    }
    let front = f64::from(data[0]);
    let back = f64::from(data[(pairs - 1) * 2]);
    let scaled = range.parameter(argument, front, back);
    let times: Vec<f64> = (0..pairs).map(|index| f64::from(data[index * 2])).collect();
    let at = upper_bound(&times, scaled);
    if at == 0 {
        return f64::from(data[1]);
    }
    if at == pairs {
        return f64::from(data[(pairs - 1) * 2 + 1]);
    }
    let value = |index: usize| data.get(index * 2 + 1).copied().map_or(f64::NAN, f64::from);
    let time = |index: usize| data.get(index * 2).copied().map_or(f64::NAN, f64::from);
    match update {
        Interpolation::None => value(at - 1),
        Interpolation::Linear => {
            interpolate_value_linear(scaled, time(at - 1), value(at - 1), time(at), value(at))
        }
        Interpolation::Plateau => {
            interpolate_value_plateau(scaled, time(at - 1), value(at - 1), time(at), value(at))
        }
        Interpolation::BSpline => bspline_nonuniform(scaled, at, pairs, &value, &time),
    }
}

/// `update_INTERPOLATE_BSPLINE_dataMore3_` - `functionvalue.cpp:751-825`, whose pointer
/// differences count the keys in front of `dat3` (`iVar5`/2) and behind it (`iVar3`/2). The
/// control points and knots an end reads past the array are the reference's out-of-bounds reads;
/// the port answers NaN for them.
fn bspline_nonuniform(
    scaled: f64,
    at: usize,
    count: usize,
    value: &impl Fn(usize) -> f64,
    time: &impl Fn(usize) -> f64,
) -> f64 {
    let before = at;
    let after = count - at;
    let mut points = [0.0; 4];
    let mut knots = [0.0; 6];
    points[1] = value(at - 1);
    points[2] = value(at);
    knots[2] = time(at - 1);
    knots[3] = time(at);
    match before {
        1 => {
            points[0] = 2.0 * points[1] - points[2];
            points[3] = value(at + 1);
            knots[4] = time(at + 1);
            knots[1] = 2.0 * knots[2] - knots[3];
            knots[0] = 2.0 * knots[2] - knots[4];
            knots[5] = if after == 1 || after == 2 {
                2.0 * knots[4] - knots[3]
            } else {
                value(at + 2)
            };
        }
        2 => {
            points[0] = value(at - 2);
            knots[1] = time(at - 2);
            knots[0] = 2.0 * knots[1] - knots[2];
            end_points(after, at, &mut points, &mut knots, value, time);
        }
        _ => {
            points[0] = value(at - 2);
            knots[1] = time(at - 2);
            knots[0] = time(at - 3);
            end_points(after, at, &mut points, &mut knots, value, time);
        }
    }
    interpolate_value_bspline_nonuniform(scaled, &points, &knots)
}

/// The tail the reference shares between its last two branches - `functionvalue.cpp:779-822`.
fn end_points(
    after: usize,
    at: usize,
    points: &mut [f64; 4],
    knots: &mut [f64; 6],
    value: &impl Fn(usize) -> f64,
    time: &impl Fn(usize) -> f64,
) {
    match after {
        1 => {
            points[3] = 2.0 * points[2] - points[1];
            knots[4] = 2.0 * knots[3] - knots[2];
            knots[5] = 2.0 * knots[3] - knots[1];
        }
        2 => {
            points[3] = value(at + 1);
            knots[4] = time(at + 1);
            knots[5] = 2.0 * knots[4] - knots[3];
        }
        _ => {
            points[3] = value(at + 1);
            knots[4] = time(at + 1);
            knots[5] = value(at + 2);
        }
    }
}

/// `TFunctionValue_hermite::getValue` - `functionvalue.cpp:873-893`.
fn eval_hermite(data: &[f32], count: u32, size: u32, range: &Range, argument: f64) -> f64 {
    if size < 2 {
        return 0.0;
    }
    let stride = size as usize;
    let entries = (count as usize).min(data.len() / stride);
    if entries == 0 {
        return 0.0;
    }
    let front = f64::from(data[0]);
    let back = f64::from(data[(entries - 1) * stride]);
    let scaled = range.parameter(argument, front, back);
    let times: Vec<f64> = (0..entries)
        .map(|index| f64::from(data[index * stride]))
        .collect();
    let at = upper_bound(&times, scaled);
    if at == 0 {
        return f64::from(data[1]);
    }
    if at == entries {
        return f64::from(data[(entries - 1) * stride + 1]);
    }
    let previous = (at - 1) * stride;
    let current = at * stride;
    let sample = |index: usize| data.get(index).copied().map_or(f64::NAN, f64::from);
    interpolate_value_hermite(
        scaled,
        sample(previous),
        sample(previous + 1),
        sample(previous + stride - 1),
        sample(current),
        sample(current + 1),
        sample(current + 2),
    )
}

/// `JGadget::findUpperBound_binary_current` (`search.h:108-117`) over one keyed array: the first
/// entry whose key is greater than `value`. For the sorted keys every value here is built on,
/// that is `std::upper_bound`, which the reference's galloping searches narrow to.
fn upper_bound(times: &[f64], value: f64) -> usize {
    times.partition_point(|time| *time <= value)
}

/// `interpolateValue_linear` - `functionvalue.cpp:124-126`.
fn interpolate_value_linear(a1: f64, a2: f64, a3: f64, a4: f64, a5: f64) -> f64 {
    a3 + ((a5 - a3) * (a1 - a2)) / (a4 - a2)
}

/// `interpolateValue_linear_1` - `functionvalue.cpp:128-130`.
fn interpolate_value_linear_1(a1: f64, a2: f64, a3: f64, a4: f64) -> f64 {
    a3 + (a4 - a3) * (a1 - a2)
}

/// `interpolateValue_plateau` - `functionvalue.cpp:132-134`.
fn interpolate_value_plateau(a1: f64, a2: f64, a3: f64, a4: f64, a5: f64) -> f64 {
    interpolate_value_hermite(a1, a2, a3, 0.0, a4, a5, 0.0)
}

/// `interpolateValue_hermite` - `functionvalue.cpp:57-72`.
fn interpolate_value_hermite(c0: f64, c1: f64, x: f64, c2: f64, x2: f64, c3: f64, x3: f64) -> f64 {
    let a = c0 - c1;
    let b = a * (1.0 / (x2 - c1));
    let c = b - 1.0;
    let d = (3.0 + -2.0 * b) * (b * b);
    let cab = c * a * b;
    let coeffx3 = cab * x3;
    let cca = c * c * a;
    let coeffc2 = cca * c2;
    ((1.0 - d) * x + (d * c3)) + coeffc2 + coeffx3
}

/// `interpolateValue_BSpline_uniform` - `functionvalue.cpp:75-94`.
fn interpolate_value_bspline_uniform(
    factor: f64,
    point2: f64,
    point3: f64,
    point4: f64,
    point5: f64,
) -> f64 {
    let inverse = 1.0 - factor;
    let inverse2 = inverse * inverse;
    let inverse3 = inverse2 * inverse;
    let factor2 = factor * factor;
    let factor3 = factor2 * factor;
    let coefficient1 = inverse3;
    let blend3 = 0.5 * factor3 - factor2 + (2.0 / 3.0);
    let blend4 = 0.5 * (factor + factor2 - factor3) + (1.0 / 6.0);
    let coefficient2 = factor3;
    ((coefficient1 * point2) + (coefficient2 * point5)) * (1.0 / 6.0)
        + (blend3 * point3)
        + (blend4 * point4)
}

/// `interpolateValue_BSpline_nonuniform` - `functionvalue.cpp:97-122`.
fn interpolate_value_bspline_nonuniform(factor: f64, points: &[f64; 4], knots: &[f64; 6]) -> f64 {
    let knot3 = knots[3];
    let inverse_delta = 1.0 / (knot3 - knots[2]);
    let blend3 = ((knot3 - factor) * inverse_delta) / (knot3 - knots[1]);
    let blend2 = ((factor - knots[2]) * inverse_delta) / (knots[4] - knots[2]);
    let blend1 = ((knot3 - factor) * blend3) / (knot3 - knots[0]);
    let blend4 =
        (((factor - knots[1]) * blend3) + ((knots[4] - factor) * blend2)) / (knots[4] - knots[1]);
    let blend5 = ((factor - knots[2]) * blend2) / (knots[5] - knots[2]);
    let term1 = (knot3 - factor) * blend1;
    let term2 = ((factor - knots[0]) * blend1) + ((knots[4] - factor) * blend4);
    let term3 = ((factor - knots[1]) * blend4) + ((knots[5] - factor) * blend5);
    let term4 = (factor - knots[2]) * blend5;
    (term1 * points[0]) + (term2 * points[1]) + (term3 * points[2]) + (term4 * points[3])
}

/// The per-kind adaptor state - `TAdaptor_actor`, `TAdaptor_camera`, `TAdaptor_ambientLight`,
/// `TAdaptor_light`, `TAdaptor_fog`, `TAdaptor_message`, `TAdaptor_sound` and
/// `TAdaptor_particle` in their factory forms (`JStudio_JStage/control.h:37-220`,
/// `JStudio_JMessage/control.h`, `JStudio_JAudio/control.h`, `JStudio_JParticle/control.h`).
#[derive(Debug, Clone, PartialEq)]
enum Body {
    Actor(ActorBody),
    Camera(CameraBody),
    Ambient(AmbientBody),
    Light(LightBody),
    Fog(FogBody),
    Message(MessageBody),
    Sound(SoundBody),
    Particle(ParticleBody),
}

/// `JStudio::TAdaptor_actor`'s 14 variables (`jstudio-object.h:248`): 0/1/2 are the animation
/// frame, transition and texture-animation frame, 3-5 translation, 6-8 rotation, 9-11 scaling,
/// 12/13 the parent and relation enables.
#[derive(Debug, Clone, PartialEq)]
struct ActorBody {
    vars: [Variable; 14],
    /// `m13C` - `object-actor.cpp:120-126`: the animation mode, which the frame output reads as
    /// its max-mirror flag and outside-function index (`:236-257`).
    anim_mode: u32,
    /// `m140` - `:133-139`, the texture-animation mode.
    tex_anim_mode: u32,
    /// `m12C`/`m130` and `m134`/`m138` - `object-actor.cpp:141-222`.
    parent: Option<DemoObjectRef>,
    parent_node: u32,
    relation: Option<DemoObjectRef>,
    relation_node: u32,
}

/// `JStudio::TAdaptor_camera`'s 11 variables (`jstudio-object.h:302`): 0-2 view position, 3-5
/// target position, 6 fovy, 7 roll, 8/9 near/far, 10 the parent enable.
#[derive(Debug, Clone, PartialEq)]
struct CameraBody {
    vars: [Variable; 11],
    parent: Option<DemoObjectRef>,
    parent_node: u32,
}

/// `JStudio::TAdaptor_ambientLight`'s 4 colour variables (`jstudio-object.h:270`).
#[derive(Debug, Clone, PartialEq)]
struct AmbientBody {
    vars: [Variable; 4],
}

/// `JStudio::TAdaptor_light`'s 13 variables (`jstudio-object.h:367`): 0-3 colour, 4-6 position,
/// 7-9 target position, 10/11 direction theta/phi, 12 the enable. `direction_mode` is `_118`
/// (`object-light.cpp:89-121`): 1 selects theta/phi, 2 the target-minus-position direction.
#[derive(Debug, Clone, PartialEq)]
struct LightBody {
    vars: [Variable; 13],
    direction_mode: i32,
}

/// `JStudio::TAdaptor_fog`'s 6 variables (`jstudio-object.h:329`): 0-3 colour, 4/5 start and end.
#[derive(Debug, Clone, PartialEq)]
struct FogBody {
    vars: [Variable; 6],
}

/// `JStudio::TAdaptor_message` (`jstudio-object.h:461`) carries no state of its own: the one
/// command writes the control's message code, which the port's message box holds.
#[derive(Debug, Clone, PartialEq)]
struct MessageBody;

/// `JStudio::TAdaptor_sound`'s 11 variables (`jstudio-object.h:430-452`): 0-2 position, 3 the
/// fade-in, 4 the fade-out, 5 the located enable, 6-10 the five mix values.
#[derive(Debug, Clone, PartialEq)]
struct SoundBody {
    vars: [Variable; 11],
    /// `mSoundID` - `object-sound.cpp:20`: `-1` until a `SOUND` command names one.
    sound_id: u32,
    /// The id the live handle was started with. Retail stops `mpSound`, whose id is this one, not
    /// whatever `mSoundID` has become since.
    playing_id: u32,
    /// `mpSound != NULL`: whether a sound has been started and not stopped.
    live: bool,
    /// `mPosition != NULL` - `object-sound.cpp:96-108`: whether the adaptor's own position is
    /// handed to `startSoundVec`.
    located: bool,
    /// `_F8` - `:62-68`: the position the `LOCATED` output points the sound at.
    position: Vec3,
}

/// `JStudio::TAdaptor_particle`'s 20 variables (`jstudio-object.h:400-421`): 0-2 translation,
/// 3-5 rotation, 6-8 scaling, 9-12 the primary colour, 13-16 the environment colour, 17 the
/// parent enable, 18/19 the fade-in and fade-out. The remaining fields are the ones
/// `JStudio_JParticle::TAdaptor_particle` adds (`object-particle.h`).
#[derive(Debug, Clone, PartialEq)]
struct ParticleBody {
    vars: [Variable; 20],
    /// `field_0x1B0` - `object-particle.cpp:86-87`: the id the `PARTICLE` command carries.
    effect_id: i32,
    /// `field_0x1B4` - the fade mode, 0 none, 1 fading in, 2 fading out (`:59-81`).
    fade_mode: u32,
    /// `field_0x1B8` - the fade's frame count.
    fade_frames: u32,
    /// `field_0x1BC` - the fade's counter.
    fade_counter: u32,
    /// `field_0x1C8` - `:110-127`, the parent node.
    parent_node: u32,
    /// `field_0x1CC` - `:129-138`, the parent enable.
    parent_enable: u32,
    /// `mObject` - `:95-108`, the parent's stage object.
    parent: Option<DemoObjectRef>,
    /// `mEmitter` - the emitter slot the fade-in output claimed.
    emitter: Option<usize>,
}

impl Body {
    fn new(kind: Kind) -> Self {
        match kind {
            Kind::Actor => Self::Actor(ActorBody {
                vars: [Variable::NONE; 14],
                anim_mode: 0,
                tex_anim_mode: 0,
                parent: None,
                // `TAdaptor_actor`'s constructor - `object-actor.cpp:11-20`: both node slots
                // start at `0xFFFFFFFF`.
                parent_node: u32::MAX,
                relation: None,
                relation_node: u32::MAX,
            }),
            Kind::Camera => Self::Camera(CameraBody {
                vars: [Variable::NONE; 11],
                parent: None,
                parent_node: 0,
            }),
            Kind::Ambient => Self::Ambient(AmbientBody {
                vars: [Variable::NONE; 4],
            }),
            Kind::Light => Self::Light(LightBody {
                vars: [Variable::NONE; 13],
                direction_mode: 0,
            }),
            Kind::Fog => Self::Fog(FogBody {
                vars: [Variable::NONE; 6],
            }),
            Kind::Message => Self::Message(MessageBody),
            Kind::Sound => {
                let mut body = SoundBody {
                    vars: [Variable::NONE; 11],
                    sound_id: u32::MAX,
                    playing_id: u32::MAX,
                    live: false,
                    located: false,
                    position: Vec3::ZERO,
                };
                // `adaptor_do_prepare` - `object-sound.cpp:32-46`: the three position variables'
                // starting values; the fade outputs need no state of their own.
                for variable in &mut body.vars[..3] {
                    variable.set_immediate(0.0);
                }
                Self::Sound(body)
            }
            Kind::Particle => {
                let mut body = ParticleBody {
                    vars: [Variable::NONE; 20],
                    effect_id: -1,
                    fade_mode: 0,
                    fade_frames: 0,
                    fade_counter: 0,
                    parent_node: u32::MAX,
                    parent_enable: 0,
                    parent: None,
                    emitter: None,
                };
                // `adaptor_do_prepare` - `object-particle.cpp:39-52`: the seventeen defaults the
                // block's first wait writes out.
                for variable in &mut body.vars[..6] {
                    variable.set_immediate(0.0);
                }
                for variable in &mut body.vars[6..9] {
                    variable.set_immediate(1.0);
                }
                for variable in &mut body.vars[9..17] {
                    variable.set_immediate(255.0);
                }
                Self::Particle(body)
            }
        }
    }

    fn begin(&mut self, target: Option<DemoObjectRef>, ctx: &mut Ctx<'_, '_>) {
        match self {
            Self::Actor(body) => body.begin(target, ctx),
            Self::Camera(body) => body.begin(target, ctx),
            Self::Ambient(body) => body.begin(target, ctx),
            Self::Light(body) => body.begin(target, ctx),
            Self::Fog(body) => body.begin(target, ctx),
            // `TAdaptor_message`, `TAdaptor_sound` and `TAdaptor_particle` inherit
            // `TAdaptor::adaptor_do_begin`, which is empty (`jstudio-object.cpp:83-84`).
            Self::Message(_) | Self::Sound(_) | Self::Particle(_) => {}
        }
    }

    fn on_wait(&mut self, target: Option<DemoObjectRef>, frames: u32, ctx: &mut Ctx<'_, '_>) {
        match self {
            Self::Actor(body) => body.on_wait(target, frames, ctx),
            Self::Camera(body) => body.on_wait(target, frames, ctx),
            Self::Ambient(body) => body.on_wait(target, frames, ctx),
            Self::Light(body) => body.on_wait(target, frames, ctx),
            Self::Fog(body) => body.on_wait(target, frames, ctx),
            Self::Message(_) => {}
            Self::Sound(body) => body.on_wait(frames, ctx),
            Self::Particle(body) => body.on_wait(frames, ctx),
        }
    }

    fn on_data(
        &mut self,
        target: Option<DemoObjectRef>,
        id: Option<(&[u8], u32)>,
        data: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    ) {
        // `TAdaptor_object_::adaptor_data_` - `JStudio_JStage/object.cpp:12-14`. Only the actor's
        // `JSGSetData` is defined (`d_demo.cpp:259-264`); the other stage objects inherit the
        // empty `JStage::TObject::JSGSetData` (`JSGObject.cpp:35-37`), so there is nothing to
        // write and no value to invent.
        let Some(DemoObjectRef::Actor(actor)) = target else {
            return;
        };
        let id_value = match id {
            None => u32::MAX,
            Some((bytes, id_size)) => {
                if id_size == 0 {
                    u32::MAX
                } else {
                    match be_u32(bytes, 0) {
                        Some(value) => value,
                        None => {
                            ctx.gaps.push(DemoCommandGap::Truncated { at });
                            return;
                        }
                    }
                }
            }
        };
        if let Some(demo) = ctx.stage.object.get_actor_mut(actor) {
            demo.set_data(id_value, data.to_vec(), size);
        }
    }

    fn on_paragraph(
        &mut self,
        target: Option<DemoObjectRef>,
        type_: u32,
        content: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    ) {
        match self {
            Self::Actor(body) => body.on_paragraph(target, type_, content, size, at, ctx),
            Self::Camera(body) => body.on_paragraph(target, type_, content, size, at, ctx),
            Self::Ambient(body) => body.on_paragraph(target, type_, content, size, at, ctx),
            Self::Light(body) => body.on_paragraph(target, type_, content, size, at, ctx),
            Self::Fog(body) => body.on_paragraph(target, type_, content, size, at, ctx),
            Self::Message(body) => body.on_paragraph(type_, content, at, ctx),
            Self::Sound(body) => body.on_paragraph(type_, content, size, at, ctx),
            Self::Particle(body) => body.on_paragraph(type_, content, size, at, ctx),
        }
    }
}

impl ActorBody {
    /// `TAdaptor_actor::adaptor_do_begin` - `object-actor.cpp:43-74`. The translation, rotation
    /// and scaling variables come from the demo actor's own getters; the animation variables come
    /// from `JSGGetAnimationTransition`/`JSGGetAnimationFrame`/`JSGGetTextureAnimationFrame`,
    /// which `dDemo_actor_c` does not override, so all three are `0.0f` (`JSGActor.cpp:63-65`).
    fn begin(&mut self, target: Option<DemoObjectRef>, ctx: &mut Ctx<'_, '_>) {
        let Some(actor) = actor_target(target) else {
            return;
        };
        let (trans, rotate, scale) = match ctx.stage.object.get_actor_mut(actor) {
            Some(demo) => (demo.trans(), demo.get_rotation(), demo.scale()),
            None => return,
        };
        set_vec(
            &mut self.vars,
            [3, 4, 5],
            transform_position(ctx.origin, trans),
        );
        set_vec(
            &mut self.vars,
            [6, 7, 8],
            transform_rotation(ctx.origin, rotate),
        );
        set_vec(&mut self.vars, [9, 10, 11], scale);
        self.vars[0].set_immediate(0.0);
        self.vars[1].set_immediate(0.0);
        self.vars[2].set_immediate(0.0);
    }

    /// `TAdaptor_actor::adaptor_do_update` - `object-actor.cpp:82-103`, plus the variable outputs
    /// `adaptor_updateVariableValue` fires first (`jstudio-object.cpp:193-202`).
    fn on_wait(&mut self, target: Option<DemoObjectRef>, frames: u32, ctx: &mut Ctx<'_, '_>) {
        let second_per_frame = f64::from(SECONDS_PER_FRAME);
        let mut fired = [(0usize, 0.0f32); 14];
        let mut count = 0;
        for (index, variable) in self.vars.iter_mut().enumerate() {
            variable.forward(frames);
            if variable.update(ctx.fvb, second_per_frame) {
                fired[count] = (index, variable.value);
                count += 1;
            }
        }
        let Some(actor) = actor_target(target) else {
            return;
        };
        for &(index, value) in &fired[..count] {
            let Some(demo) = ctx.stage.object.get_actor_mut(actor) else {
                break;
            };
            match index {
                // `saoVVOutput_ANIMATION_FRAME_` row 0 - `object-actor.cpp:269`.
                0 => {
                    let max = demo.anm_frame_max();
                    demo.set_animation_frame(animation_frame_output(max, self.anim_mode, value));
                }
                // `saoVVOutput_` row `TEACTOR_1` - `:264`.
                1 => demo.set_animation_transition(value),
                // `saoVVOutput_ANIMATION_FRAME_` row 2 - `:270`.
                2 => {
                    let max = demo.tex_animation_frame_max();
                    demo.set_texture_animation_frame(animation_frame_output(
                        max,
                        self.tex_anim_mode,
                        value,
                    ));
                }
                // Indices 12/13's BOOL outputs call `JSGSetParent`/`JSGSetRelation`, the empty
                // `JStage::TObject` defaults (`JSGObject.cpp:43-49`); nothing observable.
                _ => {}
            }
        }
        let Some(demo) = ctx.stage.object.get_actor_mut(actor) else {
            return;
        };
        let origin = ctx.origin;
        let trans = transform_position(
            origin,
            Vec3::new(self.vars[3].value, self.vars[4].value, self.vars[5].value),
        );
        let rotate = transform_rotation(
            origin,
            Vec3::new(self.vars[6].value, self.vars[7].value, self.vars[8].value),
        );
        demo.set_translation(trans);
        demo.set_rotation(rotate);
        demo.set_scaling(Vec3::new(
            self.vars[9].value,
            self.vars[10].value,
            self.vars[11].value,
        ));
    }

    /// `JStudio::TObject_actor::do_paragraph` - `jstudio-object.cpp:289-415`. The paragraph type
    /// is `(group << 5) | operation`; the `value` arms set a variable through
    /// `adaptor_setVariableValue`, the `value_n` arms through `adaptor_setVariableValue_n`, and
    /// the named arms call the adaptor's own method.
    fn on_paragraph(
        &mut self,
        target: Option<DemoObjectRef>,
        type_: u32,
        content: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    ) {
        let group = type_ >> 5;
        let operation = (type_ & 0x1F) as u8;
        match group {
            57 => {
                // SHAPE - `setJSG_ID_(&JSGSetShape, ...)`, only `TEOD_Unknown_19` acts.
                if operation == 0x19 {
                    match u32_at(content, 0) {
                        Some(value) => {
                            if let Some(DemoObjectRef::Actor(actor)) = target
                                && let Some(demo) = ctx.stage.object.get_actor_mut(actor)
                            {
                                demo.set_shape(value);
                            }
                        }
                        None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            58 => {
                if operation == 0x19 {
                    match u32_at(content, 0) {
                        Some(value) => {
                            if let Some(DemoObjectRef::Actor(actor)) = target
                                && let Some(demo) = ctx.stage.object.get_actor_mut(actor)
                            {
                                demo.set_animation(value);
                            }
                        }
                        None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            67 => {
                // ANIMATION_MODE - `object-actor.cpp:120-126`.
                if operation == 0x02 {
                    match u32_at(content, 0) {
                        Some(value) => self.anim_mode = value,
                        None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            76 => {
                if operation == 0x19 {
                    match u32_at(content, 0) {
                        Some(value) => {
                            if let Some(DemoObjectRef::Actor(actor)) = target
                                && let Some(demo) = ctx.stage.object.get_actor_mut(actor)
                            {
                                demo.set_texture_animation(value);
                            }
                        }
                        None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            78 => {
                if operation == 0x02 {
                    match u32_at(content, 0) {
                        Some(value) => self.tex_anim_mode = value,
                        None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            48 => self.parent = parent_lookup(content, ctx),
            49 => {
                self.parent_node =
                    node_command(self.parent_node, self.parent, operation, content, at, ctx)
            }
            50 => set_variable(&mut self.vars, 12, operation, content, size, at, ctx),
            51 => self.relation = parent_lookup(content, ctx),
            52 => {
                self.relation_node = node_command(
                    self.relation_node,
                    self.relation,
                    operation,
                    content,
                    at,
                    ctx,
                )
            }
            53 => set_variable(&mut self.vars, 13, operation, content, size, at, ctx),
            59 => set_variable(&mut self.vars, 0, operation, content, size, at, ctx),
            75 => set_variable(&mut self.vars, 1, operation, content, size, at, ctx),
            9 => set_variable(&mut self.vars, 3, operation, content, size, at, ctx),
            10 => set_variable(&mut self.vars, 4, operation, content, size, at, ctx),
            11 => set_variable(&mut self.vars, 5, operation, content, size, at, ctx),
            13 => set_variable(&mut self.vars, 6, operation, content, size, at, ctx),
            14 => set_variable(&mut self.vars, 7, operation, content, size, at, ctx),
            15 => set_variable(&mut self.vars, 8, operation, content, size, at, ctx),
            17 => set_variable(&mut self.vars, 9, operation, content, size, at, ctx),
            18 => set_variable(&mut self.vars, 10, operation, content, size, at, ctx),
            19 => set_variable(&mut self.vars, 11, operation, content, size, at, ctx),
            12 => set_variable_n(&mut self.vars, &[3, 4, 5], operation, content, at, ctx),
            16 => set_variable_n(&mut self.vars, &[6, 7, 8], operation, content, at, ctx),
            20 => set_variable_n(&mut self.vars, &[9, 10, 11], operation, content, at, ctx),
            _ => {}
        }
    }
}

impl CameraBody {
    /// `TAdaptor_camera::adaptor_do_begin` - `object-camera.cpp:67-82`. The two positions are the
    /// demo camera's own getters, which read the live view when the demo flags are clear
    /// (`d_demo.cpp:394-436`); this port has no view layer handing one in, so `None` is the
    /// reference's own `getView() == NULL` fallbacks. The four `saoVVOutput_` getters are
    /// `dDemo_camera_c`'s, with the same no-view fallbacks.
    fn begin(&mut self, target: Option<DemoObjectRef>, ctx: &mut Ctx<'_, '_>) {
        let origin = ctx.origin;
        let Some(demo) = camera_mut(target, ctx) else {
            return;
        };
        let position = transform_position(origin, demo.view_position(None));
        let looked_at = transform_position(origin, demo.view_target_position(None));
        let fovy = demo.projection_fovy(None);
        let roll = demo.view_roll(None);
        let near = demo.projection_near(None);
        let far = demo.projection_far(None);
        set_vec(&mut self.vars, [0, 1, 2], position);
        set_vec(&mut self.vars, [3, 4, 5], looked_at);
        self.vars[6].set_immediate(fovy);
        self.vars[7].set_immediate(roll);
        self.vars[8].set_immediate(near);
        self.vars[9].set_immediate(far);
    }

    /// `TAdaptor_camera::adaptor_do_update` - `object-camera.cpp:90-99`, which writes the two
    /// transformed positions.
    fn on_wait(&mut self, target: Option<DemoObjectRef>, frames: u32, ctx: &mut Ctx<'_, '_>) {
        let second_per_frame = f64::from(SECONDS_PER_FRAME);
        let mut fired = [(0usize, 0.0f32); 11];
        let mut count = 0;
        for (index, variable) in self.vars.iter_mut().enumerate() {
            variable.forward(frames);
            if variable.update(ctx.fvb, second_per_frame) {
                fired[count] = (index, variable.value);
                count += 1;
            }
        }
        let origin = ctx.origin;
        let Some(demo) = camera_mut(target, ctx) else {
            return;
        };
        for &(index, value) in &fired[..count] {
            // `saoVVOutput_` - `object-camera.cpp:13-19`.
            match index {
                6 => demo.set_projection_fovy(value),
                7 => demo.set_view_roll(value),
                8 => demo.set_projection_near(value),
                9 => demo.set_projection_far(value),
                // Index 10's BOOL output calls `JSGSetParent`; the empty default.
                _ => {}
            }
        }
        let position = transform_position(
            origin,
            Vec3::new(self.vars[0].value, self.vars[1].value, self.vars[2].value),
        );
        let target_position = transform_position(
            origin,
            Vec3::new(self.vars[3].value, self.vars[4].value, self.vars[5].value),
        );
        demo.set_view_position(position);
        demo.set_view_target_position(target_position);
    }

    /// `JStudio::TObject_camera::do_paragraph` - `jstudio-object.cpp:482-578`. The 39/38 pair is
    /// the reference's own order: 39 is fovy, 38 is roll.
    fn on_paragraph(
        &mut self,
        _target: Option<DemoObjectRef>,
        type_: u32,
        content: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    ) {
        let group = type_ >> 5;
        let operation = (type_ & 0x1F) as u8;
        match group {
            48 => self.parent = parent_lookup(content, ctx),
            49 => {
                self.parent_node =
                    node_command(self.parent_node, self.parent, operation, content, at, ctx)
            }
            50 => set_variable(&mut self.vars, 10, operation, content, size, at, ctx),
            21 => set_variable(&mut self.vars, 0, operation, content, size, at, ctx),
            22 => set_variable(&mut self.vars, 1, operation, content, size, at, ctx),
            23 => set_variable(&mut self.vars, 2, operation, content, size, at, ctx),
            25 => set_variable(&mut self.vars, 3, operation, content, size, at, ctx),
            26 => set_variable(&mut self.vars, 4, operation, content, size, at, ctx),
            27 => set_variable(&mut self.vars, 5, operation, content, size, at, ctx),
            39 => set_variable(&mut self.vars, 6, operation, content, size, at, ctx),
            38 => set_variable(&mut self.vars, 7, operation, content, size, at, ctx),
            40 => set_variable(&mut self.vars, 8, operation, content, size, at, ctx),
            41 => set_variable(&mut self.vars, 9, operation, content, size, at, ctx),
            24 => set_variable_n(&mut self.vars, &[0, 1, 2], operation, content, at, ctx),
            28 => set_variable_n(&mut self.vars, &[3, 4, 5], operation, content, at, ctx),
            42 => set_variable_n(&mut self.vars, &[8, 9], operation, content, at, ctx),
            _ => {}
        }
    }
}

impl AmbientBody {
    /// `TAdaptor_ambientLight::adaptor_do_begin` - `object-ambientlight.cpp:30-33`. The colour is
    /// `JSGGetColor`, whose `dDemo_ambient_c` does not override, so the `JStage` default's
    /// `{0xFF, 0xFF, 0xFF, 0xFF}` is the defined begin value (`JSGAmbientLight.cpp:21-23`).
    fn begin(&mut self, target: Option<DemoObjectRef>, ctx: &mut Ctx<'_, '_>) {
        if !matches!(target, Some(DemoObjectRef::Ambient)) {
            return;
        }
        let _ = ctx;
        for variable in &mut self.vars {
            variable.set_immediate(255.0);
        }
    }

    /// `TAdaptor_ambientLight::adaptor_do_update` - `object-ambientlight.cpp:41-45`.
    fn on_wait(&mut self, target: Option<DemoObjectRef>, frames: u32, ctx: &mut Ctx<'_, '_>) {
        let second_per_frame = f64::from(SECONDS_PER_FRAME);
        for variable in &mut self.vars {
            variable.forward(frames);
            variable.update(ctx.fvb, second_per_frame);
        }
        if !matches!(target, Some(DemoObjectRef::Ambient)) {
            return;
        }
        let color = [
            clamp_u8(self.vars[0].value),
            clamp_u8(self.vars[1].value),
            clamp_u8(self.vars[2].value),
            clamp_u8(self.vars[3].value),
        ];
        ctx.stage.object.create_ambient().set_color(color);
    }

    /// `JStudio::TObject_ambientLight::do_paragraph` - `jstudio-object.cpp:427-469`.
    fn on_paragraph(
        &mut self,
        _target: Option<DemoObjectRef>,
        type_: u32,
        content: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    ) {
        let group = type_ >> 5;
        let operation = (type_ & 0x1F) as u8;
        match group {
            29 => set_variable(&mut self.vars, 0, operation, content, size, at, ctx),
            30 => set_variable(&mut self.vars, 1, operation, content, size, at, ctx),
            31 => set_variable(&mut self.vars, 2, operation, content, size, at, ctx),
            32 => set_variable(&mut self.vars, 3, operation, content, size, at, ctx),
            33 => set_variable_n(&mut self.vars, &[0, 1, 2], operation, content, at, ctx),
            34 => set_variable_n(&mut self.vars, &[0, 1, 2, 3], operation, content, at, ctx),
            _ => {}
        }
    }
}

impl LightBody {
    /// `TAdaptor_light::adaptor_do_begin` - `object-light.cpp:43-72`. The colour comes from
    /// `JSGGetColor` (the `JStage` default, since `dDemo_light_c` overrides only the setter).
    ///
    /// The position and direction come from `JSGGetPosition`/`JSGGetDirection`
    /// (`object-light.cpp:52-53`), whose `JStage::TLight` implementations are empty
    /// (`JSGLight.cpp`): retail reads two uninitialised stack triples and then does the maths
    /// below with them. The port's light answers the values it holds, `None` until a `JSGSet*`
    /// has written them, and every derived write - the position triple, the two atan2 angles and
    /// the position-plus-direction target - is made from that typed answer. The control
    /// transform is applied when it is enabled; on the create-time step it is not, because
    /// `create` enables it after `forward(0)` (`d_demo.cpp:728-734`).
    fn begin(&mut self, target: Option<DemoObjectRef>, ctx: &mut Ctx<'_, '_>) {
        let Some(index) = light_target(target) else {
            return;
        };
        for variable in &mut self.vars[..4] {
            variable.set_immediate(255.0);
        }
        let (position, direction) = match ctx.stage.object.lights().get(index) {
            Some(light) => (light.jsg_get_position(), light.jsg_get_direction()),
            None => return,
        };
        let origin = ctx.origin;
        let position = position.map(|value| transform_position(origin, value));
        let direction = direction.map(|value| transform_direction(origin, value));
        if let Some(position) = position {
            set_vec(&mut self.vars, [4, 5, 6], position);
        }
        if let Some(direction) = direction {
            let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();
            self.vars[10].set_immediate(direction.x.atan2(direction.z).to_degrees());
            self.vars[11].set_immediate(direction.y.atan2(distance).to_degrees());
        }
        if let (Some(position), Some(direction)) = (position, direction) {
            set_vec(&mut self.vars, [7, 8, 9], position + direction);
        }
    }

    /// `TAdaptor_light::adaptor_do_update` - `object-light.cpp:80-121`. The direction outputs
    /// (`saoVVOutput_direction_`, `:13-20`) only set the direction mode. The position and
    /// direction are written every update, as the reference writes them; a triple the adaptor
    /// has no value for is written as absent, which is what the reference's uninitialised read
    /// leaves behind.
    fn on_wait(&mut self, target: Option<DemoObjectRef>, frames: u32, ctx: &mut Ctx<'_, '_>) {
        let second_per_frame = f64::from(SECONDS_PER_FRAME);
        let mut fired = [(0usize, 0.0f32); 13];
        let mut count = 0;
        for (index, variable) in self.vars.iter_mut().enumerate() {
            variable.forward(frames);
            if variable.update(ctx.fvb, second_per_frame) {
                fired[count] = (index, variable.value);
                count += 1;
            }
        }
        for &(index, _value) in &fired[..count] {
            match index {
                10 | 11 => self.direction_mode = 1,
                7 | 8 | 9 => self.direction_mode = 2,
                _ => {}
            }
        }
        let Some(index) = light_target(target) else {
            return;
        };
        let position = optional_vec3(&self.vars, [4, 5, 6]);
        let direction = match self.direction_mode {
            1 => match (self.vars[10].component(), self.vars[11].component()) {
                (Some(theta), Some(phi)) => {
                    let theta = f64::from(theta).to_radians();
                    let phi = f64::from(phi).to_radians();
                    Some(Vec3::new(
                        (phi.cos() * theta.sin()) as f32,
                        phi.sin() as f32,
                        (phi.cos() * theta.cos()) as f32,
                    ))
                }
                _ => None,
            },
            2 => match (optional_vec3(&self.vars, [7, 8, 9]), position) {
                (Some(target), Some(position)) => Some(target - position),
                _ => None,
            },
            _ => None,
        };
        let position = position.map(|value| transform_position(ctx.origin, value));
        let direction = direction.map(|value| transform_direction(ctx.origin, value));
        let color = [
            clamp_u8(self.vars[0].value),
            clamp_u8(self.vars[1].value),
            clamp_u8(self.vars[2].value),
            clamp_u8(self.vars[3].value),
        ];
        let Some(light) = ctx.stage.object.light_mut(index) else {
            return;
        };
        light.set_color(color);
        light.set_position(position);
        light.set_direction(direction);
    }

    /// `JStudio::TObject_light::do_paragraph` - `jstudio-object.cpp:658-767`. 54 is ENABLE
    /// (whose only write, `JSGFEnableFlag(2)`, lands on the empty `JStage` defaults) and 55 is
    /// FACULTY, the light type.
    fn on_paragraph(
        &mut self,
        target: Option<DemoObjectRef>,
        type_: u32,
        content: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    ) {
        let group = type_ >> 5;
        let operation = (type_ & 0x1F) as u8;
        match group {
            54 => set_variable(&mut self.vars, 12, operation, content, size, at, ctx),
            55 => {
                // `TAdaptor_light::adaptor_do_FACULTY` - `object-light.cpp:129-154`.
                if operation == 0x02 {
                    match u32_at(content, 0) {
                        Some(0x301) => self.set_light_type(target, ctx, 1),
                        Some(0x302) => self.set_light_type(target, ctx, 2),
                        Some(0x303) => self.set_light_type(target, ctx, 3),
                        Some(_) => {}
                        None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            29 => set_variable(&mut self.vars, 0, operation, content, size, at, ctx),
            30 => set_variable(&mut self.vars, 1, operation, content, size, at, ctx),
            31 => set_variable(&mut self.vars, 2, operation, content, size, at, ctx),
            32 => set_variable(&mut self.vars, 3, operation, content, size, at, ctx),
            21 => set_variable(&mut self.vars, 4, operation, content, size, at, ctx),
            22 => set_variable(&mut self.vars, 5, operation, content, size, at, ctx),
            23 => set_variable(&mut self.vars, 6, operation, content, size, at, ctx),
            25 => set_variable(&mut self.vars, 7, operation, content, size, at, ctx),
            26 => set_variable(&mut self.vars, 8, operation, content, size, at, ctx),
            27 => set_variable(&mut self.vars, 9, operation, content, size, at, ctx),
            35 => set_variable(&mut self.vars, 10, operation, content, size, at, ctx),
            36 => set_variable(&mut self.vars, 11, operation, content, size, at, ctx),
            33 => set_variable_n(&mut self.vars, &[0, 1, 2], operation, content, at, ctx),
            34 => set_variable_n(&mut self.vars, &[0, 1, 2, 3], operation, content, at, ctx),
            24 => set_variable_n(&mut self.vars, &[4, 5, 6], operation, content, at, ctx),
            28 => set_variable_n(&mut self.vars, &[7, 8, 9], operation, content, at, ctx),
            37 => set_variable_n(&mut self.vars, &[10, 11], operation, content, at, ctx),
            _ => {}
        }
    }

    fn set_light_type(&mut self, target: Option<DemoObjectRef>, ctx: &mut Ctx<'_, '_>, value: i32) {
        if let Some(index) = light_target(target)
            && let Some(light) = ctx.stage.object.light_mut(index)
        {
            light.set_light_type(value);
        }
    }
}

impl FogBody {
    /// `TAdaptor_fog::adaptor_do_begin` - `object-fog.cpp:40-46`. The colour is the `JStage`
    /// default `{0xFF, 0xFF, 0xFF, 0xFF}` and the start/end the defaults `0.0f`/`FLOAT_MAX`
    /// (`JSGFog.cpp:29-49`); all three are spread into variables, so the first wait writes them
    /// out through the outputs and the colour update.
    fn begin(&mut self, target: Option<DemoObjectRef>, ctx: &mut Ctx<'_, '_>) {
        if !matches!(target, Some(DemoObjectRef::Fog)) {
            return;
        }
        let _ = ctx;
        for variable in &mut self.vars[..4] {
            variable.set_immediate(255.0);
        }
        self.vars[4].set_immediate(0.0);
        self.vars[5].set_immediate(f32::MAX);
    }

    /// `TAdaptor_fog::adaptor_do_update` - `object-fog.cpp:54-58`: only the colour is written
    /// here; the start/end outputs fire from their variables.
    fn on_wait(&mut self, target: Option<DemoObjectRef>, frames: u32, ctx: &mut Ctx<'_, '_>) {
        let second_per_frame = f64::from(SECONDS_PER_FRAME);
        let mut fired = [(0usize, 0.0f32); 6];
        let mut count = 0;
        for (index, variable) in self.vars.iter_mut().enumerate() {
            variable.forward(frames);
            if variable.update(ctx.fvb, second_per_frame) {
                fired[count] = (index, variable.value);
                count += 1;
            }
        }
        let Some(demo) = fog_mut(target, ctx) else {
            return;
        };
        for &(index, value) in &fired[..count] {
            // `saoVVOutput_` - `object-fog.cpp:26-30`.
            match index {
                4 => demo.set_start_z(value),
                5 => demo.set_end_z(value),
                _ => {}
            }
        }
        let color = [
            clamp_u8(self.vars[0].value),
            clamp_u8(self.vars[1].value),
            clamp_u8(self.vars[2].value),
            clamp_u8(self.vars[3].value),
        ];
        demo.set_color(color);
    }

    /// `JStudio::TObject_fog::do_paragraph` - `jstudio-object.cpp:591-643`.
    fn on_paragraph(
        &mut self,
        _target: Option<DemoObjectRef>,
        type_: u32,
        content: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    ) {
        let group = type_ >> 5;
        let operation = (type_ & 0x1F) as u8;
        match group {
            29 => set_variable(&mut self.vars, 0, operation, content, size, at, ctx),
            30 => set_variable(&mut self.vars, 1, operation, content, size, at, ctx),
            31 => set_variable(&mut self.vars, 2, operation, content, size, at, ctx),
            32 => set_variable(&mut self.vars, 3, operation, content, size, at, ctx),
            43 => set_variable(&mut self.vars, 4, operation, content, size, at, ctx),
            44 => set_variable(&mut self.vars, 5, operation, content, size, at, ctx),
            33 => set_variable_n(&mut self.vars, &[0, 1, 2], operation, content, at, ctx),
            34 => set_variable_n(&mut self.vars, &[0, 1, 2, 3], operation, content, at, ctx),
            45 => set_variable_n(&mut self.vars, &[4, 5], operation, content, at, ctx),
            _ => {}
        }
    }
}

impl MessageBody {
    /// `JStudio::TObject_message::do_paragraph` - `jstudio-object.cpp:776-792`: group `0x42` is
    /// `MESSAGE`, and the one operation that acts is `TEOD_Unknown_19`, which hands the
    /// paragraph's word to `adaptor_do_MESSAGE` (`JStudio_JMessage/object-message.cpp:22-30`).
    fn on_paragraph(&mut self, type_: u32, content: &[u8], at: usize, ctx: &mut Ctx<'_, '_>) {
        if type_ >> 5 != 0x42 || (type_ & 0x1F) as u8 != 0x19 {
            return;
        }
        match u32_at(content, 0) {
            // `mControl->setMessageCode(*(u32*)param_2)` - `JStudio_JMessage/object-message.cpp:25`.
            // The control resolves the group through `mResourceContainer->Get_groupID`
            // (`JMessage/control.cpp:29-39`) and loads the entry at the code's index
            // (`resource.h:34-44`); `dMesg_waitProc` opens the box on the changed code
            // (`d_mesg.cpp:2361-2445`), which the port's box does in `MsgBox::message_set_code`.
            // A group no loaded resource carries is `getMessageEntry`'s NULL, which `dMesg` reads
            // as a failed flush, so the box is left where it is and the group is reported.
            Some(code) => {
                let group = (code >> 16) as u16;
                let table = ctx.stage.msg_table;
                if !table.has_group(group) {
                    ctx.gaps.push(DemoCommandGap::MessageGroup { group });
                    return;
                }
                ctx.stage.messages.message_set_code(code, table);
            }
            None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
        }
    }
}

impl SoundBody {
    /// `adaptor_updateVariableValue` fires each variable's output before `adaptor_do_update`
    /// (`jstudio-object.cpp:239-245`). Variables 3 and 4 are the fade outputs
    /// (`object-sound.cpp:40-41`), 5 the located enable, and 6-10 the five `TVVOSetValue_` rows
    /// (`:50-57`).
    fn on_wait(&mut self, frames: u32, ctx: &mut Ctx<'_, '_>) {
        let second_per_frame = f64::from(SECONDS_PER_FRAME);
        let mut fired = [(0usize, 0.0f32); 11];
        let mut count = 0;
        for (index, variable) in self.vars.iter_mut().enumerate() {
            variable.forward(frames);
            if variable.update(ctx.fvb, second_per_frame) {
                fired[count] = (index, variable.value);
                count += 1;
            }
        }
        for &(index, value) in &fired[..count] {
            match index {
                3 => self.begin_fade_in(ctx),
                4 => self.end_fade_out(ctx),
                5 => self.located = value >= 0.5,
                _ => {}
            }
        }
        // `adaptor_do_update` - `object-sound.cpp:59-69`: the position variables are read into
        // `_F8` and transformed when the control's transform is on.
        let position = Vec3::new(self.vars[0].value, self.vars[1].value, self.vars[2].value);
        self.position = transform_position(ctx.origin, position);
    }

    /// `TVVOOn_BEGIN_FADE_IN_::operator()` - `object-sound.cpp:112-134`. `checkEnablePrepare` is
    /// `soundID & JAISoundID_TypeMask`: a sequence or stream id is started on the prepared handle
    /// and is the port's music channel, while an SE is started afresh, first stopping the
    /// previous handle when the id's `0x00000c00` bits say so.
    fn begin_fade_in(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.sound_id & SOUND_ID_TYPE_MASK != 0 {
            if self.live {
                ctx.stage.events.list.push(SimEvent::Bgm(self.sound_id));
                self.playing_id = self.sound_id;
            }
            return;
        }
        if self.live && self.sound_id & SOUND_ID_STOP_MASK != 0 {
            ctx.stage
                .events
                .list
                .push(SimEvent::SeStop(self.playing_id));
        }
        let event = if self.located {
            SimEvent::SeAt(self.sound_id, self.position)
        } else {
            SimEvent::Se(self.sound_id)
        };
        ctx.stage.events.list.push(event);
        self.playing_id = self.sound_id;
        self.live = true;
    }

    /// `TVVOOn_END_FADE_OUT_::operator()` - `object-sound.cpp:137-142`: `mpSound->stop(value)`.
    fn end_fade_out(&mut self, ctx: &mut Ctx<'_, '_>) {
        if self.live {
            ctx.stage
                .events
                .list
                .push(SimEvent::SeStop(self.playing_id));
            self.live = false;
        }
    }

    /// `JStudio::TObject_sound::do_paragraph` - `jstudio-object.cpp:964-1051`. 60 is `SOUND`, 56
    /// `LOCATED` (variable 5 with its output), 21-23 and 24 the position, 46/47 the fade
    /// variables, 61-65 the five mix variables.
    fn on_paragraph(
        &mut self,
        type_: u32,
        content: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    ) {
        let group = type_ >> 5;
        let operation = (type_ & 0x1F) as u8;
        match group {
            60 => {
                // `adaptor_do_SOUND`'s `TEOD_Unknown_19` - `object-sound.cpp:72-93`: the id is
                // `*(s32*)data`; a prepared id stops the handle it replaces and either branch
                // stores it in `mSoundID`.
                if operation == 0x19 {
                    match u32_at(content, 0) {
                        Some(sound_id) => {
                            if sound_id & SOUND_ID_TYPE_MASK != 0 {
                                if self.live {
                                    ctx.stage
                                        .events
                                        .list
                                        .push(SimEvent::SeStop(self.playing_id));
                                }
                                // `prepareSoundVec` leaves `mpSound` on the prepared id
                                // (`object-sound.cpp:81`), which the fade-in then starts.
                                self.playing_id = sound_id;
                                self.live = true;
                            }
                            self.sound_id = sound_id;
                        }
                        None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            56 => set_variable(&mut self.vars, 5, operation, content, size, at, ctx),
            21 => set_variable(&mut self.vars, 0, operation, content, size, at, ctx),
            22 => set_variable(&mut self.vars, 1, operation, content, size, at, ctx),
            23 => set_variable(&mut self.vars, 2, operation, content, size, at, ctx),
            46 => set_variable(&mut self.vars, 3, operation, content, size, at, ctx),
            47 => set_variable(&mut self.vars, 4, operation, content, size, at, ctx),
            61 => set_variable(&mut self.vars, 6, operation, content, size, at, ctx),
            62 => set_variable(&mut self.vars, 7, operation, content, size, at, ctx),
            63 => set_variable(&mut self.vars, 8, operation, content, size, at, ctx),
            64 => set_variable(&mut self.vars, 9, operation, content, size, at, ctx),
            65 => set_variable(&mut self.vars, 10, operation, content, size, at, ctx),
            24 => set_variable_n(&mut self.vars, &[0, 1, 2], operation, content, at, ctx),
            _ => {}
        }
    }
}

impl ParticleBody {
    /// `adaptor_updateVariableValue` fires the outputs first (`jstudio-object.cpp:239-245`), then
    /// `TAdaptor_particle::adaptor_do_update` advances the fade counter
    /// (`object-particle.cpp:60-81`), and the emitter's own callback reads the variables and
    /// writes the emitter (`:183-277`). The port runs that callback here, at the step the demo
    /// owns: the values it reads are the ones the variables above have just produced, and the
    /// fields it writes are the emitter's.
    fn on_wait(&mut self, frames: u32, ctx: &mut Ctx<'_, '_>) {
        let second_per_frame = f64::from(SECONDS_PER_FRAME);
        let mut fired = [(0usize, 0.0f32); 20];
        let mut count = 0;
        for (index, variable) in self.vars.iter_mut().enumerate() {
            variable.forward(frames);
            if variable.update(ctx.fvb, second_per_frame) {
                fired[count] = (index, variable.value);
                count += 1;
            }
        }
        for &(index, value) in &fired[..count] {
            match index {
                18 => self.begin_fade_in(value, ctx),
                19 => self.end_fade_out(value),
                17 => self.parent_enable = u32::from(value >= 0.5),
                _ => {}
            }
        }
        // `adaptor_do_update` - `object-particle.cpp:61-81`.
        if self.fade_frames != 0 && self.fade_counter < self.fade_frames {
            self.fade_counter = self.fade_counter.saturating_add(frames);
            if self.fade_counter >= self.fade_frames {
                if self.fade_mode != 2 {
                    self.fade_mode = 0;
                }
                self.fade_frames = 0;
                self.fade_counter = 0;
            }
        }
        self.callback(ctx);
    }

    /// `TVVOOn_BEGIN_FADE_IN_::operator()` - `object-particle.cpp:141-162`: the previous emitter
    /// is force-deleted, `createSimpleEmitterID` claims the new slot from the packed effect id,
    /// makes it immortal, and the fade starts.
    fn begin_fade_in(&mut self, value: f32, ctx: &mut Ctx<'_, '_>) {
        if let Some(slot) = self.emitter.take() {
            free_emitter(ctx.stage.particles, slot);
        }
        let id = self.effect_id as u32;
        // `adaptor_particle->mEmitterManager->createSimpleEmitterID(pos, temp & 0xFFFF,
        // (temp & 0xFF000000) >> 0x18, (temp & 0x00FF0000) >> 0x10, &mCallback, NULL)` -
        // `object-particle.cpp:148-155`. The bank is what the id resolves through
        // (`JPAEmitterManager.cpp:52-80`): a hit carries its resource - the reference pins
        // `getLinkInfo()[0]` on the emitter (`:71`, `JPAEmitter.cpp:134`) - and a miss creates
        // nothing, which is the slot [`Jpa::create_simple_emitter_packed_id`] refuses here and
        // records as a [`JpaGap`](super::jpa::JpaGap). `pos` is the adaptor's constant
        // `(0, 0, 0)`: `TJPACallback_::execute` writes the emitter's transform every frame.
        let Some(slot) = ctx
            .stage
            .particles
            .create_simple_emitter_packed_id(Vec3::ZERO, id)
        else {
            ctx.gaps.push(DemoCommandGap::ParticleId { id });
            return;
        };
        // `if (adaptor_particle->mEmitter) { ... }` - `:156-161`.
        ctx.stage.particles.emitters[slot].become_immortal();
        self.emitter = Some(slot);
        self.fade_mode = 1;
        self.fade_frames = value as u32;
        self.fade_counter = 0;
    }

    /// `TVVOOn_END_FADE_OUT_::operator()` - `object-particle.cpp:165-181`: the fade's length is
    /// rewritten so the time left is what the variable names.
    fn end_fade_out(&mut self, value: f32) {
        if self.emitter.is_none() {
            return;
        }
        if self.fade_mode == 1 && self.fade_counter != 0 {
            self.fade_mode = 2;
            let ratio = f64::from(self.fade_frames) / f64::from(self.fade_counter);
            self.fade_frames = (f64::from(value) * ratio) as u32;
            self.fade_counter = (f64::from(value) * (ratio - 1.0)) as u32;
        } else {
            self.fade_mode = 2;
            self.fade_frames = value as u32;
            self.fade_counter = 0;
        }
    }

    /// `TJPACallback_::execute` - `object-particle.cpp:183-277`. The parented branch multiplies
    /// the parent node's matrix by the local SRT and hands the product to
    /// `JPABaseEmitter::setGlobalSRTMatrix`; a stage object whose model cannot answer with the
    /// node's matrix is [`DemoCommandGap::NodeTransform`], the port's spelling of the reference's
    /// `false`.
    fn callback(&mut self, ctx: &mut Ctx<'_, '_>) {
        let Some(slot) = self.emitter else {
            return;
        };
        let Some(emitter) = ctx.stage.particles.emitters.get(slot) else {
            self.emitter = None;
            return;
        };
        if emitter.is_enable_delete_emitter() {
            self.fade_mode = 0;
            self.fade_frames = 0;
            self.fade_counter = 0;
            free_emitter(ctx.stage.particles, slot);
            self.emitter = None;
            return;
        }
        let frames = f64::from(self.fade_frames);
        let counter = f64::from(self.fade_counter);
        let mut alpha = 1.0f64;
        match self.fade_mode {
            1 => {
                if self.fade_frames != 0 {
                    alpha = counter / frames;
                }
            }
            2 => {
                if self.fade_frames == 0 {
                    let emitter = &mut ctx.stage.particles.emitters[slot];
                    alpha = 0.0;
                    emitter.stop_create_particle();
                    emitter.max_frame = 1;
                } else {
                    alpha = (frames - counter) / frames;
                }
            }
            _ => {}
        }
        // `emitter->stopDrawParticle()` - `object-particle.cpp:219`, before the matrix and the
        // colours are written. Only the non-parented path plays the draw again (`:276`).
        ctx.stage.particles.emitters[slot].stop_draw_particle();
        let translation = Vec3::new(self.vars[0].value, self.vars[1].value, self.vars[2].value);
        let rotation = Vec3::new(self.vars[3].value, self.vars[4].value, self.vars[5].value);
        let scale = Vec3::new(self.vars[6].value, self.vars[7].value, self.vars[8].value);
        if self.parent_enable != 0 {
            // `if (mAdaptor->mObject == NULL) return;` - `object-particle.cpp:248-250`.
            let Some(parent) = self.parent else {
                return;
            };
            // `mAdaptor->mObject->JSGGetNodeTransformation(mAdaptor->field_0x1C8, spA8)`
            // - `:250-253`. A `false` answer is where retail returns without a matrix, and the
            // port's model-less stage object is what answers it.
            let Some(node) = ctx.stage.node_transform(parent, self.parent_node) else {
                ctx.gaps.push(DemoCommandGap::NodeTransform {
                    object: parent,
                    node: self.parent_node,
                });
                return;
            };
            // `getTransformation_SRxyzT(sp78, sp0C, srts[1], srts[0])` then
            // `MTXConcat(spA8, sp78, sp48)` - `:255-257`. The parented path reads the three
            // variables raw: the control's transform is the other branch's alone.
            let local = srt_matrix(scale, rotation, translation);
            ctx.stage.particles.emitters[slot].set_global_srt_matrix(node * local);
        } else {
            let (translation, rotation) = match ctx.origin {
                Some(origin) => (
                    transform_position(Some(origin), translation),
                    transform_rotation(Some(origin), rotation),
                ),
                None => (translation, rotation),
            };
            let emitter = &mut ctx.stage.particles.emitters[slot];
            emitter.global_translation = translation;
            emitter.global_rotation = [
                (65536.0f32 * (rotation.x / 360.0)) as i16,
                (65536.0f32 * (rotation.y / 360.0)) as i16,
                (65536.0f32 * (rotation.z / 360.0)) as i16,
            ];
            emitter.global_particle_scale = scale;
        }
        // `sauVariableValue_4_COLOR_RGBA` and `sauVariableValue_4_COLOR1_RGBA` are both
        // `{9, 10, 11, 12}` in this build (`jstudio-object.cpp:798-800`), so the environment
        // colour is the primary colour's second read, as the reference's callback makes it.
        let color = [
            clamp_u8(self.vars[9].value),
            clamp_u8(self.vars[10].value),
            clamp_u8(self.vars[11].value),
            clamp_u8(self.vars[12].value),
        ];
        let emitter = &mut ctx.stage.particles.emitters[slot];
        emitter.global_prm_color.r = color[0];
        emitter.global_prm_color.g = color[1];
        emitter.global_prm_color.b = color[2];
        emitter.global_env_color.r = color[0];
        emitter.global_env_color.g = color[1];
        emitter.global_env_color.b = color[2];
        let mut scaled = alpha * f64::from(color[3]);
        if scaled > 255.0 {
            scaled = 255.0;
        }
        let mut emit = 255.0f64;
        if scaled < 255.0 {
            emit = scaled.max(0.0);
        }
        emitter.set_global_alpha(emit as i16);
        emitter.play_draw_particle();
    }

    /// `JStudio::TObject_particle::do_paragraph` - `jstudio-object.cpp:809-953`. 68 is
    /// `PARTICLE`, 48/49/50 the parent, the parent node and the parent enable, 9-32 the
    /// translation, rotation, scaling and primary colour, 69-72 the environment colour, 46/47
    /// the fade variables, and 12/16/20/33/34/73/74 the variable-n forms.
    fn on_paragraph(
        &mut self,
        type_: u32,
        content: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    ) {
        let group = type_ >> 5;
        let operation = (type_ & 0x1F) as u8;
        match group {
            68 => {
                // `adaptor_do_PARTICLE`'s `TEOD_Unknown_19` - `object-particle.cpp:84-92`.
                if operation == 0x19 {
                    match u32_at(content, 0) {
                        Some(id) => self.effect_id = id as i32,
                        None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            48 => self.parent = parent_lookup(content, ctx),
            49 => {
                self.parent_node =
                    node_command(self.parent_node, self.parent, operation, content, at, ctx)
            }
            50 => set_variable(&mut self.vars, 17, operation, content, size, at, ctx),
            9 => set_variable(&mut self.vars, 0, operation, content, size, at, ctx),
            10 => set_variable(&mut self.vars, 1, operation, content, size, at, ctx),
            11 => set_variable(&mut self.vars, 2, operation, content, size, at, ctx),
            13 => set_variable(&mut self.vars, 3, operation, content, size, at, ctx),
            14 => set_variable(&mut self.vars, 4, operation, content, size, at, ctx),
            15 => set_variable(&mut self.vars, 5, operation, content, size, at, ctx),
            17 => set_variable(&mut self.vars, 6, operation, content, size, at, ctx),
            18 => set_variable(&mut self.vars, 7, operation, content, size, at, ctx),
            19 => set_variable(&mut self.vars, 8, operation, content, size, at, ctx),
            29 => set_variable(&mut self.vars, 9, operation, content, size, at, ctx),
            30 => set_variable(&mut self.vars, 10, operation, content, size, at, ctx),
            31 => set_variable(&mut self.vars, 11, operation, content, size, at, ctx),
            32 => set_variable(&mut self.vars, 12, operation, content, size, at, ctx),
            69 => set_variable(&mut self.vars, 13, operation, content, size, at, ctx),
            70 => set_variable(&mut self.vars, 14, operation, content, size, at, ctx),
            71 => set_variable(&mut self.vars, 15, operation, content, size, at, ctx),
            72 => set_variable(&mut self.vars, 16, operation, content, size, at, ctx),
            46 => set_variable(&mut self.vars, 18, operation, content, size, at, ctx),
            47 => set_variable(&mut self.vars, 19, operation, content, size, at, ctx),
            12 => set_variable_n(&mut self.vars, &[0, 1, 2], operation, content, at, ctx),
            16 => set_variable_n(&mut self.vars, &[3, 4, 5], operation, content, at, ctx),
            20 => set_variable_n(&mut self.vars, &[6, 7, 8], operation, content, at, ctx),
            33 => set_variable_n(&mut self.vars, &[9, 10, 11], operation, content, at, ctx),
            34 => set_variable_n(
                &mut self.vars,
                &[9, 10, 11, 12],
                operation,
                content,
                at,
                ctx,
            ),
            73 => set_variable_n(&mut self.vars, &[9, 10, 11], operation, content, at, ctx),
            74 => set_variable_n(
                &mut self.vars,
                &[9, 10, 11, 12],
                operation,
                content,
                at,
                ctx,
            ),
            _ => {}
        }
    }
}

/// The reference's `forceDeleteEmitter`, as the port's pool spells it: the slot goes back to the
/// free list.
fn free_emitter(jpa: &mut Jpa, slot: usize) {
    if let Some(emitter) = jpa.emitters.get_mut(slot) {
        emitter.active = false;
        emitter.callback = None;
        emitter.particle_callback = None;
        emitter.particles.clear();
    }
}

/// `TAdaptor::adaptor_setVariableValue` - `jstudio-object.cpp:95-119`. The two function-value
/// operations resolve the bank entry the paragraph names; a lookup that answers nothing is the
/// NULL retail hands to `setValue_functionValue` and then dereferences in
/// `update_functionValue_`, so the variable keeps its previous value and the miss is reported.
fn set_variable(
    vars: &mut [Variable],
    index: usize,
    operation: u8,
    content: &[u8],
    size: u32,
    at: usize,
    ctx: &mut Ctx<'_, '_>,
) {
    match operation {
        1 => vars[index].set_none(),
        2 => match f32_at(content, 0) {
            Some(value) => vars[index].set_immediate(value),
            None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
        },
        3 => match f32_at(content, 0) {
            Some(value) => vars[index].set_time(value),
            None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
        },
        // `adaptor_setVariableValue_FVR_NAME_` - `jstudio-object.cpp:221-225`: the FVB bank is
        // looked up by the paragraph's bytes and size.
        0x10 => match content.get(..size as usize) {
            Some(name) => match ctx
                .fvb
                .iter()
                .position(|object| object.id.as_slice() == name)
            {
                Some(bank) => vars[index].set_function_value(bank),
                None => ctx.gaps.push(DemoCommandGap::FunctionValueName {
                    name: name.to_vec(),
                }),
            },
            None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
        },
        // `adaptor_setVariableValue_FVR_INDEX_` - `:227-231`.
        0x12 => match u32_at(content, 0) {
            Some(bank) => match ctx.fvb.get(bank as usize) {
                Some(_) => vars[index].set_function_value(bank as usize),
                None => ctx
                    .gaps
                    .push(DemoCommandGap::FunctionValueIndex { index: bank }),
            },
            None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
        },
        _ => {}
    }
}

/// `TAdaptor::adaptor_setVariableValue_n` - `jstudio-object.cpp:121-151`: op 1 strides zero
/// bytes, 2/3/0x12 stride four. The function-value name form has no arm here in retail either.
fn set_variable_n(
    vars: &mut [Variable],
    indices: &[usize],
    operation: u8,
    content: &[u8],
    at: usize,
    ctx: &mut Ctx<'_, '_>,
) {
    let stride = match operation {
        1 => 0,
        2 | 3 | 0x12 => 4,
        _ => return,
    };
    for (step, index) in indices.iter().enumerate() {
        let offset = step * stride;
        match operation {
            1 => vars[*index].set_none(),
            2 => match f32_at(content, offset) {
                Some(value) => vars[*index].set_immediate(value),
                None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
            },
            3 => match f32_at(content, offset) {
                Some(value) => vars[*index].set_time(value),
                None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
            },
            0x12 => match u32_at(content, offset) {
                Some(bank) => match ctx.fvb.get(bank as usize) {
                    Some(_) => vars[*index].set_function_value(bank as usize),
                    None => ctx
                        .gaps
                        .push(DemoCommandGap::FunctionValueIndex { index: bank }),
                },
                None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
            },
            _ => {}
        }
    }
}

/// `adaptor_do_PARENT`/`RELATION`'s `TEOD_Unknown_18` arm - `object-actor.cpp:141-150`: the
/// paragraph content is a C string for `JSGFindObject(name, TOBJ_ACTOR_UNK)`. A miss stores the
/// reference's own NULL rather than a gap, and a name the port cannot read as a C string is the
/// same miss: retail would search with the bytes it found and come back empty.
fn parent_lookup(content: &[u8], ctx: &mut Ctx<'_, '_>) -> Option<DemoObjectRef> {
    let name = c_string_slice(content)?;
    ctx.stage.find_object(name, ObjectKind::ActorUnk).ok()
}

/// `adaptor_do_PARENT_NODE`/`RELATION_NODE` - `object-actor.cpp:152-166`, and the camera's and
/// the particle's same two arms (`object-camera.cpp:120-130`, `object-particle.cpp:110-127`):
/// `TEOD_Unknown_18` is a name lookup in the **parent** stage object's J3D node table
/// (`JSGFindNodeID`), and it only runs while the parent pointer exists; `TEOD_Unknown_19` is a
/// plain index. Retail stores the lookup's answer, so a name the parent's model does not carry -
/// or a parent with no model - lands as `-1`, which is `u32::MAX` here; only a lookup retail
/// never runs (no parent object) leaves the field alone.
fn node_command(
    current: u32,
    parent: Option<DemoObjectRef>,
    operation: u8,
    content: &[u8],
    at: usize,
    ctx: &mut Ctx<'_, '_>,
) -> u32 {
    match operation {
        0x18 => {
            let Some(parent) = parent else {
                return current;
            };
            let name = name_bytes(content);
            match ctx.stage.find_node_id(parent, &name) {
                Some(node) => node,
                None => {
                    ctx.gaps.push(DemoCommandGap::NodeName {
                        object: parent,
                        name,
                    });
                    u32::MAX
                }
            }
        }
        0x19 => match u32_at(content, 0) {
            Some(value) => value,
            None => {
                ctx.gaps.push(DemoCommandGap::Truncated { at });
                current
            }
        },
        _ => current,
    }
}

/// `TVVOutput_ANIMATION_FRAME_::operator()` - `object-actor.cpp:236-257`. The table's `_08` is
/// the byte offset of the adaptor's mode word - `m13C` for the animation row, `m140` for the
/// texture row, both stored `+1` in the table - so the mode is the index: its low byte selects
/// the outside function and its high byte `1` mirrors the value around the frame max.
fn animation_frame_output(max: f32, mode: u32, value: f32) -> f32 {
    let mut result = value;
    if (mode >> 8) as u8 == 1 {
        result = max - result;
    }
    if max > 0.0 {
        result = outside_function((mode & 0xFF) as u32)(f64::from(result), f64::from(max)) as f32;
    }
    result
}

/// `TFunctionValue::toFunction_outside` - `functionvalue.cpp:27-41` - over
/// `gapfnExtrapolateParameter_` (`:18-23`). Anything outside the four-entry table is the raw
/// identity, the reference's own fallback.
fn outside_function(index: u32) -> fn(f64, f64) -> f64 {
    match index {
        1 => extrapolate_repeat,
        2 => extrapolate_turn,
        3 => extrapolate_clamp,
        _ => extrapolate_raw,
    }
}

fn extrapolate_raw(value: f64, _max: f64) -> f64 {
    value
}

fn extrapolate_repeat(value: f64, max: f64) -> f64 {
    let wrapped = value % max;
    if wrapped < 0.0 {
        wrapped + max
    } else {
        wrapped
    }
}

fn extrapolate_turn(value: f64, max: f64) -> f64 {
    let doubled = 2.0 * max;
    let wrapped = extrapolate_repeat(value, doubled);
    if wrapped >= max {
        doubled - wrapped
    } else {
        wrapped
    }
}

fn extrapolate_clamp(value: f64, max: f64) -> f64 {
    if value <= 0.0 {
        0.0
    } else if max <= value {
        max
    } else {
        value
    }
}

/// `getValue_clamp<u8>` - `jstudio-object.h:88-98`.
fn clamp_u8(value: f32) -> u8 {
    if value <= 0.0 {
        0
    } else if value >= 255.0 {
        255
    } else {
        value as u8
    }
}

/// `TControl::TTransform_position_direction`'s write path: `transformOnSet_transformTranslation`
/// is `MTXMultVec` of the `getTransformation_RyT` matrix (`jstudio-control.h:119-137`,
/// `jstudio-math.cpp:64-69`) - a Y rotation then the origin translation.
fn transform_position(origin: Option<(Vec3, f32)>, position: Vec3) -> Vec3 {
    match origin {
        Some((offset, rotation)) => {
            Quat::from_rotation_y(rotation.to_radians()) * position + offset
        }
        None => position,
    }
}

/// The rotation half of `adaptor_do_begin`/`update`: `sp14[1].y = sp2C[1].y + field_0xa8`
/// (`object-actor.cpp:55-63`, `:90-99`).
fn transform_rotation(origin: Option<(Vec3, f32)>, rotate: Vec3) -> Vec3 {
    match origin {
        Some((_, rotation)) => Vec3::new(rotate.x, rotate.y + rotation, rotate.z),
        None => rotate,
    }
}

/// The direction half, `MTXMultVecSR` of the same matrix - rotation without translation
/// (`jstudio-control.h:135-137`).
fn transform_direction(origin: Option<(Vec3, f32)>, direction: Vec3) -> Vec3 {
    match origin {
        Some((_, rotation)) => Quat::from_rotation_y(rotation.to_radians()) * direction,
        None => direction,
    }
}

/// `JStudio::math::getTransformation_SRxyzT` - `jstudio-math.cpp:78-86`: `MTXScale`, then
/// `rotate_xyz`, then `MTXTransApply`, in the row-major `Mtx` layout the reference writes, which
/// is the layout [`crate::j3d::bdl`]'s `joint_matrix` composes for a `JNT1` local. The rotation
/// is [`rotation_xyz`]'s degrees.
fn srt_matrix(scale: Vec3, rotation: Vec3, translation: Vec3) -> Mat4 {
    let mut rows = rotation_xyz(rotation);
    for row in rows.iter_mut().take(3) {
        row[0] *= scale.x;
        row[1] *= scale.y;
        row[2] *= scale.z;
    }
    rows[0][3] = translation.x;
    rows[1][3] = translation.y;
    rows[2][3] = translation.z;
    Mat4::from_cols_array_2d(&rows).transpose()
}

/// `JStudio::math::getRotation_xyz` - `jstudio-math.cpp:14-42`, the degree triple to its matrix.
fn rotation_xyz(rotation: Vec3) -> [[f32; 4]; 4] {
    let (x, y, z) = rotation.into();
    let (sin_x, cos_x) = x.to_radians().sin_cos();
    let (sin_y, cos_y) = y.to_radians().sin_cos();
    let (sin_z, cos_z) = z.to_radians().sin_cos();
    let mut rows = [[0.0_f32; 4]; 4];
    rows[0][0] = cos_y * cos_z;
    rows[1][0] = cos_y * sin_z;
    rows[2][0] = -sin_y;
    rows[2][1] = sin_x * cos_y;
    rows[2][2] = cos_x * cos_y;
    rows[0][1] = sin_x * cos_z * sin_y - cos_x * sin_z;
    rows[0][2] = sin_x * sin_z + cos_x * cos_z * sin_y;
    rows[1][1] = cos_x * cos_z + sin_x * sin_z * sin_y;
    rows[1][2] = cos_x * sin_z * sin_y - sin_x * cos_z;
    rows[3] = [0.0, 0.0, 0.0, 1.0];
    rows
}

/// `adaptor_setVariableValue_Vec` - `jstudio-object.cpp:162-167`.
fn set_vec(vars: &mut [Variable], indices: [usize; 3], value: Vec3) {
    vars[indices[0]].set_immediate(value.x);
    vars[indices[1]].set_immediate(value.y);
    vars[indices[2]].set_immediate(value.z);
}

/// `adaptor_getVariableValue_Vec`'s read with the port's typed answer: a triple any of whose
/// components nothing has written is absent, which is how the reference's uninitialised reads
/// reach the writes built on them.
fn optional_vec3(vars: &[Variable], indices: [usize; 3]) -> Option<Vec3> {
    Some(Vec3::new(
        vars[indices[0]].component()?,
        vars[indices[1]].component()?,
        vars[indices[2]].component()?,
    ))
}

fn actor_target(target: Option<DemoObjectRef>) -> Option<u8> {
    match target {
        Some(DemoObjectRef::Actor(id)) => Some(id),
        _ => None,
    }
}

fn light_target(target: Option<DemoObjectRef>) -> Option<usize> {
    match target {
        Some(DemoObjectRef::Light(index)) => Some(index),
        _ => None,
    }
}

fn camera_mut<'a>(
    target: Option<DemoObjectRef>,
    ctx: &'a mut Ctx<'_, '_>,
) -> Option<&'a mut DemoCamera> {
    match target {
        Some(DemoObjectRef::Camera) => Some(ctx.stage.object.create_camera()),
        _ => None,
    }
}

fn fog_mut<'a>(target: Option<DemoObjectRef>, ctx: &'a mut Ctx<'_, '_>) -> Option<&'a mut DemoFog> {
    match target {
        Some(DemoObjectRef::Fog) => Some(ctx.stage.object.create_fog()),
        _ => None,
    }
}

/// The block type's adaptor - `JStudio_JStage/control.cpp:29-52` and the three channel factories.
fn object_kind(block: u32) -> Kind {
    match block {
        BLOCK_MESSAGE => Kind::Message,
        BLOCK_SOUND => Kind::Sound,
        BLOCK_PARTICLE => Kind::Particle,
        BLOCK_CAMERA => Kind::Camera,
        BLOCK_AMBIENT => Kind::Ambient,
        BLOCK_LIGHT => Kind::Light,
        BLOCK_FOG => Kind::Fog,
        _ => Kind::Actor,
    }
}

/// One block's object - the `JStudio::TObject` the factory built, with the `stb::TObject`
/// machinery and the adaptor behind it.
#[derive(Debug, Clone, PartialEq)]
struct TimelineObject {
    kind: Kind,
    /// The block's ID as the file spells it: `TParse_TBlock_object::get_ID`/`get_IDSize`
    /// (`stb-data-parse.h:82-94`). The offset is kept because the reference's C-string reads
    /// start there and run on into the rest of the file.
    id_at: usize,
    id_size: usize,
    /// The stage slot `JSGFindObject` answered with at mount; `resolved` distinguishes "not yet
    /// asked" from the channel blocks, which never ask.
    target: Option<DemoObjectRef>,
    resolved: bool,
    state: SequenceState,
    body: Body,
}

/// `TControl::forward`'s `forward(0)`/`forward(1)` once the objects exist: the control track,
/// then every object (`stb.cpp:297-312`).
#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    /// The demo file, which every read, jump and name is taken from. The reference's objects
    /// hold pointers into the buffer `dComIfG_getStageRes` handed `create` (`d_demo.cpp:720`);
    /// the port owns a copy of it.
    file: Vec<u8>,
    control: SequenceState,
    objects: Vec<TimelineObject>,
    /// `TControl::fvb_Control`'s object list (`jstudio-control.h:120`): every `JFVB` block's
    /// values, in file order, which is the order `getObject_index` walks.
    fvb: Vec<FvbObject>,
    gaps: Vec<DemoCommandGap>,
    suspended_at_start: bool,
}

impl Timeline {
    /// `TParse::parse_next(&sp08, 0)` - `d_demo.cpp:723`: the header, then every block. Every
    /// failure the reference could answer with is a `None` here, which the manager records as
    /// [`super::demo::DemoGap::UnreadableFile`].
    fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < HEADER_SIZE || bytes.get(0..4)? != b"STB\0" {
            return None;
        }
        if be_u16(bytes, 4)? != 0xFEFF {
            return None;
        }
        let version = be_u16(bytes, 6)?;
        if !(1..=3).contains(&version) {
            return None;
        }
        if bytes.get(0x10..0x18)? != b"jstudio\0" {
            return None;
        }
        let target_version = be_u16(bytes, 0x1E)?;
        if !(2..=3).contains(&target_version) {
            return None;
        }
        let file = bytes.to_vec();
        let block_number = be_u32(&file, 0x0C)?;
        let mut gaps = Vec::new();
        let mut control: Option<SequenceState> = None;
        let mut objects = Vec::new();
        let mut fvb = Vec::new();
        let mut at = HEADER_SIZE;
        for _ in 0..block_number {
            let size = be_u32(&file, at)? as usize;
            let kind = be_u32(&file, at + 4)?;
            if size < 8 {
                return None;
            }
            let end = at.checked_add(size)?;
            if end > file.len() {
                return None;
            }
            match kind {
                BLOCK_NONE => {
                    let header = object_block(&file, at)?;
                    control = Some(SequenceState::new(header.content_at));
                }
                BLOCK_FVB => parse_fvb(&file, at, &mut fvb, &mut gaps)?,
                BLOCK_ACTOR | BLOCK_CAMERA | BLOCK_AMBIENT | BLOCK_LIGHT | BLOCK_FOG
                | BLOCK_MESSAGE | BLOCK_SOUND | BLOCK_PARTICLE => {
                    let header = object_block(&file, at)?;
                    let kind = object_kind(kind);
                    let mut state = SequenceState::new(header.content_at);
                    // `stb::TObject`'s constructor copies the block flag (`stb.cpp:23-26`).
                    state.flag = header.flag;
                    objects.push(TimelineObject {
                        kind,
                        id_at: header.id_at,
                        id_size: header.id_size,
                        target: None,
                        // The three channel factories never look their block's ID up, so their
                        // objects exist as soon as the block does.
                        resolved: kind.stage().is_none(),
                        state,
                        body: Body::new(kind),
                    });
                }
                // A block kind no factory answers is a parse failure in retail
                // (`stb.cpp:403-421`), so it stays a `None` here too.
                _ => return None,
            }
            at = end;
        }
        Some(Self {
            file,
            control: control.unwrap_or_else(|| SequenceState::new(HEADER_SIZE)),
            objects,
            fvb,
            gaps,
            suspended_at_start: false,
        })
    }

    /// The factory's `JSGFindObject` pass - `JStudio_JStage/control.cpp:63-73` and
    /// `d_demo.cpp:642-671` - which retail runs while parsing. The reader here has no stage, so
    /// the mount site runs it, and a lookup that answers nothing is retail's failed parse: the
    /// whole mount fails (`stb.cpp:403-421`, `d_demo.cpp:723-727`).
    fn resolve(&mut self, stage: &mut DemoStage<'_>) -> Result<(), DemoGap> {
        let Self { file, objects, .. } = self;
        for object in objects.iter_mut() {
            if object.resolved {
                continue;
            }
            object.resolved = true;
            let Some(kind) = object.kind.stage() else {
                continue;
            };
            // `dDemo_system_c::JSGFindObject` reads the name only in its two actor arms
            // (`d_demo.cpp:647-659`); the camera, ambient, light and fog tables answer whatever
            // the ID says, so a malformed ID fails the mount only for an actor.
            let name = if matches!(kind, ObjectKind::Actor | ObjectKind::ActorUnk) {
                match c_string(file, object.id_at) {
                    Some(name) => name.to_owned(),
                    None => {
                        let raw = file
                            .get(object.id_at..object.id_at + object.id_size)
                            .unwrap_or_default()
                            .to_vec();
                        return Err(DemoGap::UnreadableName {
                            at: object.id_at,
                            name: raw,
                        });
                    }
                }
            } else {
                String::new()
            };
            match stage.find_object(&name, kind) {
                Ok(target) => object.target = Some(target),
                Err(_) => {
                    return Err(DemoGap::NoObject {
                        kind: object.kind.name(),
                        name,
                    });
                }
            }
        }
        Ok(())
    }

    /// `stb::TControl::forward` - `stb.cpp:297-312`: `_54` is captured before the control track
    /// runs, the control track forwards first, then every object's answer is ORed in.
    ///
    /// `origin` is the demo transform: retail's `create` runs `mControl->forward(0)` *before*
    /// `transform_enable`/`transform_setOrigin` (`d_demo.cpp:730-734`), so the zero step is never
    /// transformed and only the later forwards are.
    fn forward_inner(&mut self, stage: &mut DemoStage<'_>, arg: u32, transforms: bool) -> bool {
        let Self {
            file,
            control,
            objects,
            fvb,
            gaps,
            suspended_at_start,
        } = self;
        let captured = control.suspend > 0;
        *suspended_at_start = captured;
        let origin = if transforms { stage.origin } else { None };
        let mut ctx = Ctx {
            stage,
            gaps,
            suspended: captured,
            origin,
            file,
            fvb,
        };
        let mut advanced = forward_track(control, &mut ControlHooks, arg, &mut ctx);
        for object in objects.iter_mut() {
            let TimelineObject {
                target,
                resolved,
                state,
                body,
                ..
            } = object;
            if !*resolved {
                continue;
            }
            let mut hooks = ObjectHooks {
                target: *target,
                body,
            };
            advanced = forward_track(state, &mut hooks, arg, &mut ctx) || advanced;
        }
        advanced
    }
}

impl DemoTimeline for Timeline {
    fn mount(&mut self, stage: &mut DemoStage<'_>) -> Result<(), DemoGap> {
        self.resolve(stage)?;
        self.forward_inner(stage, 0, false);
        Ok(())
    }

    fn begin(&mut self, stage: &mut DemoStage<'_>) -> bool {
        if self.resolve(stage).is_err() {
            return false;
        }
        self.forward_inner(stage, 0, false)
    }

    fn forward(&mut self, stage: &mut DemoStage<'_>) -> bool {
        if self.resolve(stage).is_err() {
            return false;
        }
        self.forward_inner(stage, 1, true)
    }

    fn suspended(&self) -> bool {
        self.suspended_at_start
    }

    fn take_gaps(&mut self) -> Vec<DemoCommandGap> {
        std::mem::take(&mut self.gaps)
    }
}

/// The per-step context a track's hooks reach: the stage the writes land on, the gap sink, the
/// demo file every read and name comes from, the function-value bank and the two control-level
/// facts - the captured suspension and the demo transform origin.
struct Ctx<'a, 's> {
    stage: &'a mut DemoStage<'s>,
    gaps: &'a mut Vec<DemoCommandGap>,
    suspended: bool,
    origin: Option<(Vec3, f32)>,
    file: &'a [u8],
    fvb: &'a [FvbObject],
}

/// `stb::TObject`'s virtuals as far as a track needs them - `on_begin`, `on_end`, `on_wait` and
/// `do_data` (`stb.cpp:138-151`).
trait TrackHooks {
    fn on_begin(&mut self, _ctx: &mut Ctx<'_, '_>) {}
    fn on_end(&mut self, _ctx: &mut Ctx<'_, '_>) {}
    fn on_wait(&mut self, frames: u32, ctx: &mut Ctx<'_, '_>);
    fn on_data(
        &mut self,
        id: Option<(&[u8], u32)>,
        data: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    );
    fn on_paragraph(
        &mut self,
        type_: u32,
        content: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    );
}

/// The control track's `stb::TObject` has no adaptor: every virtual is empty
/// (`stb.cpp:138-151`).
struct ControlHooks;

impl TrackHooks for ControlHooks {
    fn on_wait(&mut self, _frames: u32, _ctx: &mut Ctx<'_, '_>) {}

    fn on_data(
        &mut self,
        _id: Option<(&[u8], u32)>,
        _data: &[u8],
        _size: u32,
        _at: usize,
        _ctx: &mut Ctx<'_, '_>,
    ) {
    }

    fn on_paragraph(
        &mut self,
        _type_: u32,
        _content: &[u8],
        _size: u32,
        _at: usize,
        _ctx: &mut Ctx<'_, '_>,
    ) {
    }
}

/// One object's hooks: the adaptor body and the stage slot its writes land on.
struct ObjectHooks<'a> {
    target: Option<DemoObjectRef>,
    body: &'a mut Body,
}

impl TrackHooks for ObjectHooks<'_> {
    fn on_begin(&mut self, ctx: &mut Ctx<'_, '_>) {
        self.body.begin(self.target, ctx);
    }

    /// `JStudio::TObject::do_end` - `jstudio-object.cpp:256-261`, whose each adaptor's
    /// `adaptor_do_end` is `JSGFDisableFlag(1)` on the empty `JStage` defaults.
    fn on_end(&mut self, _ctx: &mut Ctx<'_, '_>) {}

    fn on_wait(&mut self, frames: u32, ctx: &mut Ctx<'_, '_>) {
        self.body.on_wait(self.target, frames, ctx);
    }

    fn on_data(
        &mut self,
        id: Option<(&[u8], u32)>,
        data: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    ) {
        self.body.on_data(self.target, id, data, size, at, ctx);
    }

    fn on_paragraph(
        &mut self,
        type_: u32,
        content: &[u8],
        size: u32,
        at: usize,
        ctx: &mut Ctx<'_, '_>,
    ) {
        self.body
            .on_paragraph(self.target, type_, content, size, at, ctx);
    }
}

/// `stb::TObject::forward` - `stb.cpp:56-136`, kept line for line: the flag's `0x8000`
/// deactivation, the suspended path that still ticks the wait hooks, and the sequence loop that
/// consumes `arg` frames across as many paragraphs as they last.
fn forward_track<H: TrackHooks>(
    state: &mut SequenceState,
    hooks: &mut H,
    arg: u32,
    ctx: &mut Ctx<'_, '_>,
) -> bool {
    let mut temp = false;
    let mut remaining = arg;
    'outer: loop {
        if state.flag & 0x8000 != 0 {
            if state.status != Status::Inactive {
                state.status = Status::Inactive;
                if state.b_sequence {
                    hooks.on_end(ctx);
                }
            }
            return true;
        }
        if state.status == Status::Inactive {
            hooks.on_begin(ctx);
            state.status = Status::Wait;
        }
        if ctx.suspended || state.suspend > 0 {
            if state.b_sequence {
                state.status = Status::Suspend;
                hooks.on_wait(remaining, ctx);
            }
            return true;
        }
        loop {
            let next = state.sequence_next;
            state.sequence = next;
            if next.is_none() {
                if state.b_sequence {
                    if !temp {
                        hooks.on_wait(0, ctx);
                    }
                    state.b_sequence = false;
                    state.status = Status::End;
                    hooks.on_end(ctx);
                }
                return false;
            }
            if !state.b_sequence {
                state.b_sequence = true;
                hooks.on_begin(ctx);
            }
            state.status = Status::Wait;
            if state.wait == 0 {
                process_sequence(state, hooks, ctx);
                if state.wait == 0 {
                    // The reference's `break` leaves the sequence loop for the outer one, which
                    // re-checks the flag, the inactive status and both suspension counts before
                    // the next sequence runs.
                    continue 'outer;
                }
            }
            temp = true;
            if remaining >= state.wait {
                let wait = state.wait;
                remaining -= state.wait;
                state.wait = 0;
                hooks.on_wait(wait, ctx);
            } else {
                state.wait -= remaining;
                hooks.on_wait(remaining, ctx);
                return true;
            }
        }
    }
}

/// `stb::TObject::process_sequence_` - `stb.cpp:153-215`.
fn process_sequence<H: TrackHooks + ?Sized>(
    state: &mut SequenceState,
    hooks: &mut H,
    ctx: &mut Ctx<'_, '_>,
) {
    let file = ctx.file;
    let Some(sequence) = state.sequence else {
        return;
    };
    let Some(head) = be_u32(file, sequence) else {
        ctx.gaps.push(DemoCommandGap::Truncated { at: sequence });
        state.sequence_next = None;
        return;
    };
    let type_ = (head >> 24) as u8;
    let param = head & 0x00FF_FFFF;
    state.sequence_next = match type_ {
        0 => None,
        0x01..=0x7F => Some(sequence + 4),
        _ => Some(sequence + 4 + param as usize),
    };
    match type_ {
        0 => {}
        1 => set_flag_operation(state, param),
        2 => state.wait = param,
        3 => {
            // `getSequence_offset(off)` - `stb.h:75-78`: the offset is from the current head,
            // and the result is a pointer into the file with no bound of its own.
            let target = (sequence as isize) + (sign_extend_24(param) as isize);
            if (0..=file.len() as isize).contains(&target) {
                state.sequence_next = Some(target as usize);
            } else {
                ctx.gaps.push(DemoCommandGap::JumpOutside {
                    at: sequence,
                    target,
                });
                state.sequence_next = None;
            }
        }
        // `suspend(val)` - `stb.h:72`: `_20` is a u32 that wraps.
        4 => state.suspend = state.suspend.wrapping_add(sign_extend_24(param) as u32),
        0x80 => {
            let start = sequence + 4;
            let end = start + param as usize;
            process_paragraphs(state, hooks, ctx, start, end);
        }
        _ => {}
    }
}

/// The paragraph loop inside an `0x80` sequence - `stb.cpp:192-210`.
fn process_paragraphs<H: TrackHooks + ?Sized>(
    state: &mut SequenceState,
    hooks: &mut H,
    ctx: &mut Ctx<'_, '_>,
    mut at: usize,
    end: usize,
) {
    let file = ctx.file;
    while at < end {
        let Some(paragraph) = read_paragraph(file, at) else {
            ctx.gaps.push(DemoCommandGap::Truncated { at });
            return;
        };
        let content_at = paragraph.content_start;
        let type_ = paragraph.type_;
        if type_ <= 0xFF {
            match type_ {
                1 => match paragraph.content.and_then(|content| be_u32(file, content)) {
                    Some(value) => set_flag_operation(state, value),
                    None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                },
                2 => match paragraph.content.and_then(|content| be_u32(file, content)) {
                    Some(value) => state.wait = value,
                    None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                },
                3 => {
                    // `setSequence_next(getSequence_offset(*(s32*)pContent))` - `stb.cpp:226-229`.
                    let jump = paragraph
                        .content
                        .and_then(|content| be_u32(file, content))
                        .map(|value| value as i32 as isize);
                    match jump {
                        Some(offset) => {
                            let Some(sequence) = state.sequence else {
                                return;
                            };
                            let target = sequence as isize + offset;
                            if (0..=file.len() as isize).contains(&target) {
                                state.sequence_next = Some(target as usize);
                            } else {
                                ctx.gaps.push(DemoCommandGap::JumpOutside {
                                    at: sequence,
                                    target,
                                });
                                state.sequence_next = None;
                            }
                        }
                        None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
                0x80 => {
                    let data = paragraph
                        .content
                        .and_then(|content| file.get(content..content + paragraph.size));
                    match data {
                        Some(data) => {
                            hooks.on_data(None, data, paragraph.size as u32, content_at, ctx)
                        }
                        // A zero size is the reference's `content == NULL` and `uSize == 0`: an
                        // empty record, not a truncation.
                        None if paragraph.size == 0 => hooks.on_data(None, &[], 0, content_at, ctx),
                        None => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
                0x81 => {
                    // `TParse_TParagraph_dataID` - `stb-data-parse.h:96-106`: the ID block is a
                    // `[_:16][id_size:16][id]` triple and the data follows its 4-byte alignment.
                    let Some(content) = paragraph.content else {
                        ctx.gaps.push(DemoCommandGap::Truncated { at });
                        at = paragraph.next;
                        continue;
                    };
                    let Some(id_size) = be_u16(file, content + 2) else {
                        ctx.gaps.push(DemoCommandGap::Truncated { at });
                        at = paragraph.next;
                        continue;
                    };
                    let data_at = content + 4 + align4(id_size as usize);
                    let data_size =
                        (paragraph.size as isize) - (data_at as isize - content as isize);
                    if data_size < 0 || data_at > file.len() {
                        ctx.gaps.push(DemoCommandGap::Truncated { at });
                        at = paragraph.next;
                        continue;
                    }
                    let id = file.get(content + 4..content + 4 + id_size as usize);
                    let data = file.get(data_at..data_at + data_size as usize);
                    match (id, data) {
                        (Some(id), Some(data)) => hooks.on_data(
                            Some((id, u32::from(id_size))),
                            data,
                            data_size as u32,
                            content_at,
                            ctx,
                        ),
                        _ => ctx.gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
                0x82 => {}
                _ => {}
            }
        } else {
            // A paragraph with no content is the reference's `NULL` pointer: the adaptors read
            // nothing from it, so the port hands them an empty slice rather than the bytes that
            // follow the header.
            let content: &[u8] = match paragraph.content {
                Some(_) => file.get(content_at..).unwrap_or(&[]),
                None => &[],
            };
            hooks.on_paragraph(type_, content, paragraph.size as u32, content_at, ctx);
        }
        at = paragraph.next;
    }
}

/// One paragraph header, already decoded from `TParse_TParagraph::getData`
/// (`stb-data-parse.cpp:36-48`).
struct Paragraph {
    size: usize,
    type_: u32,
    content: Option<usize>,
    content_start: usize,
    next: usize,
}

/// `parseVariableUInt_16_32_following` (`binary.cpp:14-34`) plus `TParse_TParagraph::getData`:
/// the 16-bit form is `[size:16][type:16]`, the 32-bit form `[0x8000|size:16][size_low:16]` then
/// a 32-bit type, and a non-zero size puts the content after the header with `align4` padding
/// before the next paragraph.
fn read_paragraph(file: &[u8], at: usize) -> Option<Paragraph> {
    let head = be_u16(file, at)?;
    let (size, type_, content_start, next) = if head & 0x8000 == 0 {
        (
            u32::from(head),
            u32::from(be_u16(file, at + 2)?),
            at + 4,
            at + 4,
        )
    } else {
        let low = u32::from(be_u16(file, at + 2)?);
        let size = ((u32::from(head) << 16) & 0x7FFF_0000) | low;
        (size, be_u32(file, at + 4)?, at + 8, at + 8)
    };
    if size == 0 {
        return Some(Paragraph {
            size: 0,
            type_,
            content: None,
            content_start,
            next,
        });
    }
    let next = content_start + align4(size as usize);
    Some(Paragraph {
        size: size as usize,
        type_,
        content: Some(content_start),
        content_start,
        next,
    })
}

/// `stb::TObject::process_paragraph_reserved_`'s flag arm - `stb.cpp:218-244` and
/// `process_sequence_`'s type 1: `TEOD_Unknown_01` sets, 2 clears, 3 toggles.
fn set_flag_operation(state: &mut SequenceState, value: u32) {
    let operation = value >> 16;
    let bits = (value & 0xFFFF) as u16;
    match operation {
        1 => state.flag |= bits,
        2 => state.flag &= bits,
        3 => state.flag ^= bits,
        _ => {}
    }
}

/// `TObject::toInt32FromUInt24_` - `stb.h:85-90`.
fn sign_extend_24(value: u32) -> i32 {
    if value & 0x0080_0000 != 0 {
        (value | 0xFF00_0000) as i32
    } else {
        value as i32
    }
}

/// One FVB block header: `fvb-data.h`'s `TBlock` - `size`, a 16-bit type and the ID's size, with
/// the ID at `+8` and the content past its 4-byte alignment (`fvb-data-parse.h:10-32`).
struct FvbBlock {
    type_: u16,
    id: Vec<u8>,
    content_at: usize,
    end: usize,
}

/// `fvb::TParse::parse` - `fvb.cpp:409-486` - over the `JFVB` block's content: the FVB header,
/// then every FVB block, each one appended to the bank. The flags `parseBlock_block_fvb_` passes
/// (`jstudio-control.cpp:101-115`) are 0 for `d_demo`'s one `parse_next`, so an unknown type is a
/// failure (`fvb.cpp:474-482`) and an existing ID is not deduplicated.
fn parse_fvb(
    file: &[u8],
    at: usize,
    bank: &mut Vec<FvbObject>,
    gaps: &mut Vec<DemoCommandGap>,
) -> Option<()> {
    if file.get(at..at + 4)? != b"FVB\0" {
        return None;
    }
    if be_u16(file, at + 4)? != 0xFEFF {
        return None;
    }
    let version = be_u16(file, at + 6)?;
    if !(2..=0x100).contains(&version) {
        return None;
    }
    let block_number = be_u32(file, at + 0x0C)?;
    let mut block = at + 0x10;
    for _ in 0..block_number {
        let header = fvb_block(file, block)?;
        let mut value = match header.type_ {
            1 => FunctionValue::composite(),
            2 => FunctionValue::Constant(f64::NAN),
            3 => FunctionValue::transition(),
            4 => FunctionValue::list(),
            5 => FunctionValue::list_parameter(),
            6 => FunctionValue::hermite(),
            _ => return None,
        };
        if prepare_value(file, &mut value, header.content_at, header.end, bank, gaps) {
            bank.push(FvbObject {
                id: header.id,
                value,
            });
        }
        block = header.end;
    }
    Some(())
}

fn fvb_block(file: &[u8], at: usize) -> Option<FvbBlock> {
    let size = be_u32(file, at)? as usize;
    if size < 8 {
        return None;
    }
    let end = at.checked_add(size)?;
    if end > file.len() {
        return None;
    }
    let type_ = be_u16(file, at + 4)?;
    let id_size = be_u16(file, at + 6)? as usize;
    let id_at = at + 8;
    let id = file.get(id_at..id_at + id_size)?.to_vec();
    Some(FvbBlock {
        type_,
        id,
        content_at: id_at + align4(id_size),
        end,
    })
}

/// `fvb::TObject::prepare` - `fvb.cpp:22-163`: walk the block's paragraphs, apply the class's
/// data paragraph and the attributes, then `prepare` the value. `false` is a block retail never
/// finishes building: the composite operation's row has no getter (`fvb.cpp:202-239`).
fn prepare_value(
    file: &[u8],
    value: &mut FunctionValue,
    content_at: usize,
    end: usize,
    bank: &mut Vec<FvbObject>,
    gaps: &mut Vec<DemoCommandGap>,
) -> bool {
    let mut at = content_at;
    while at < end {
        let Some(paragraph) = read_paragraph(file, at) else {
            gaps.push(DemoCommandGap::Truncated { at });
            break;
        };
        match paragraph.type_ {
            0 => break,
            1 => {
                if !prepare_data(file, value, &paragraph, gaps) {
                    return false;
                }
            }
            0x10 => refer_by_id(file, value, &paragraph, bank, gaps),
            0x11 => refer_by_index(file, value, &paragraph, bank, gaps),
            0x12 => {
                if let Some(range) = value.range_mut()
                    && let Some(content) = paragraph.content
                {
                    match (f32_at(file, content), f32_at(file, content + 4)) {
                        (Some(begin), Some(end)) => range.set(f64::from(begin), f64::from(end)),
                        _ => gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            0x13 => {
                if let Some(range) = value.range_mut()
                    && let Some(content) = paragraph.content
                {
                    match be_u32(file, content) {
                        Some(progress) => range.progress = progress,
                        None => gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            0x14 => {
                if let Some(range) = value.range_mut()
                    && let Some(content) = paragraph.content
                {
                    match be_u32(file, content) {
                        Some(adjust) => range.adjust = adjust,
                        None => gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            0x15 => {
                if let Some(range) = value.range_mut()
                    && let Some(content) = paragraph.content
                {
                    match (be_u16(file, content), be_u16(file, content + 2)) {
                        (Some(begin), Some(end)) => {
                            range.outside_begin = u32::from(begin);
                            range.outside_end = u32::from(end);
                        }
                        _ => gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            0x16 => {
                let slot = match value {
                    FunctionValue::Transition { interpolate, .. }
                    | FunctionValue::List { interpolate, .. }
                    | FunctionValue::ListParameter { interpolate, .. } => Some(interpolate),
                    _ => None,
                };
                if let Some(slot) = slot
                    && let Some(content) = paragraph.content
                {
                    match be_u32(file, content) {
                        Some(value) => *slot = value,
                        None => gaps.push(DemoCommandGap::Truncated { at }),
                    }
                }
            }
            _ => {}
        }
        at = paragraph.next;
    }
    value.prepare();
    true
}

/// `TObject_composite::prepare_data_` - `fvb.cpp:224-239`: the operation selects the setter and
/// the data getter, and the data word is read into `TData` the way the row's getter does.
fn prepare_data(
    file: &[u8],
    value: &mut FunctionValue,
    paragraph: &Paragraph,
    gaps: &mut Vec<DemoCommandGap>,
) -> bool {
    let at = paragraph.content_start;
    let Some(operation) = be_u32(file, at) else {
        gaps.push(DemoCommandGap::Truncated { at });
        return true;
    };
    let Some(data) = be_u32(file, at + 4) else {
        gaps.push(DemoCommandGap::Truncated { at });
        return true;
    };
    match value {
        FunctionValue::Composite {
            operation: slot,
            data: slot_data,
            ..
        } => {
            let op = match operation {
                1 => CompositeOp::Raw,
                2 => CompositeOp::Index,
                3 => CompositeOp::Parameter,
                4 => CompositeOp::Add,
                5 => CompositeOp::Subtract,
                6 => CompositeOp::Multiply,
                7 => CompositeOp::Divide,
                _ => {
                    gaps.push(DemoCommandGap::FvbComposite { operation });
                    return false;
                }
            };
            *slot = Some(op);
            *slot_data = match op {
                CompositeOp::Raw | CompositeOp::Index => CompositeData::Word(data),
                _ => CompositeData::Value(f64::from(f32::from_bits(data))),
            };
            true
        }
        FunctionValue::Constant(constant) => {
            match f32_at(file, at) {
                Some(value) => *constant = f64::from(value),
                None => gaps.push(DemoCommandGap::Truncated { at }),
            }
            true
        }
        FunctionValue::Transition {
            begin_value,
            end_value,
            ..
        } => {
            match (f32_at(file, at), f32_at(file, at + 4)) {
                (Some(begin), Some(end)) => {
                    *begin_value = f64::from(begin);
                    *end_value = f64::from(end);
                }
                _ => gaps.push(DemoCommandGap::Truncated { at }),
            }
            true
        }
        FunctionValue::List {
            data: values,
            interval,
            ..
        } => {
            let Some(step) = f32_at(file, at) else {
                gaps.push(DemoCommandGap::Truncated { at });
                return true;
            };
            let Some(count) = be_u32(file, at + 4) else {
                gaps.push(DemoCommandGap::Truncated { at });
                return true;
            };
            *interval = f64::from(step);
            match read_f32s(file, at + 8, count as usize) {
                Some(read) => *values = read,
                None => gaps.push(DemoCommandGap::Truncated { at }),
            }
            true
        }
        FunctionValue::ListParameter { data: values, .. } => {
            let Some(count) = be_u32(file, at) else {
                gaps.push(DemoCommandGap::Truncated { at });
                return true;
            };
            match read_f32s(file, at + 4, count as usize * 2) {
                Some(read) => *values = read,
                None => gaps.push(DemoCommandGap::Truncated { at }),
            }
            true
        }
        FunctionValue::Hermite {
            data: values,
            count,
            size,
            ..
        } => {
            let Some(word) = be_u32(file, at) else {
                gaps.push(DemoCommandGap::Truncated { at });
                return true;
            };
            *count = word & 0x0FFF_FFFF;
            *size = word >> 28;
            match read_f32s(file, at + 4, *count as usize * *size as usize) {
                Some(read) => *values = read,
                None => gaps.push(DemoCommandGap::Truncated { at }),
            }
            true
        }
    }
}

/// `TObject::prepare`'s `0x10` arm - `fvb.cpp:41-64`: a count, then `[size][ID]` pairs looked up
/// in the bank so far, each padded to 4 bytes.
fn refer_by_id(
    file: &[u8],
    value: &mut FunctionValue,
    paragraph: &Paragraph,
    bank: &mut Vec<FvbObject>,
    gaps: &mut Vec<DemoCommandGap>,
) {
    let at = paragraph.content_start;
    let Some(content) = paragraph.content else {
        gaps.push(DemoCommandGap::Truncated { at });
        return;
    };
    let Some(count) = be_u32(file, content) else {
        gaps.push(DemoCommandGap::Truncated { at });
        return;
    };
    let Some(refer) = value.refer_mut() else {
        return;
    };
    let mut ptr = content + 4;
    for _ in 0..count {
        let Some(size) = be_u32(file, ptr) else {
            gaps.push(DemoCommandGap::Truncated { at });
            return;
        };
        let id_at = ptr + 4;
        let Some(id) = file.get(id_at..id_at + size as usize) else {
            gaps.push(DemoCommandGap::Truncated { at });
            return;
        };
        if let Some(index) = bank.iter().position(|object| object.id.as_slice() == id) {
            refer.push(index);
        }
        ptr = id_at + align4(size as usize);
    }
}

/// `TObject::prepare`'s `0x11` arm - `fvb.cpp:65-87`: a count and that many positions in the
/// bank.
fn refer_by_index(
    file: &[u8],
    value: &mut FunctionValue,
    paragraph: &Paragraph,
    bank: &mut Vec<FvbObject>,
    gaps: &mut Vec<DemoCommandGap>,
) {
    let at = paragraph.content_start;
    let Some(content) = paragraph.content else {
        gaps.push(DemoCommandGap::Truncated { at });
        return;
    };
    let Some(count) = be_u32(file, content) else {
        gaps.push(DemoCommandGap::Truncated { at });
        return;
    };
    let Some(refer) = value.refer_mut() else {
        return;
    };
    let mut ptr = content + 4;
    for _ in 0..count {
        let Some(index) = be_u32(file, ptr) else {
            gaps.push(DemoCommandGap::Truncated { at });
            return;
        };
        if bank.get(index as usize).is_some() {
            refer.push(index as usize);
        }
        ptr += 4;
    }
}

fn read_f32s(file: &[u8], at: usize, count: usize) -> Option<Vec<f32>> {
    let mut values = Vec::with_capacity(count);
    for step in 0..count {
        values.push(f32_at(file, at + step * 4)?);
    }
    Some(values)
}

/// One object-form block header: `stb-data.h`'s `TBlock_object` - `size`, `type`, the flag, the
/// ID's size and the ID itself, with the content after the ID's 4-byte alignment.
struct ObjectBlock {
    flag: u16,
    id_at: usize,
    id_size: usize,
    content_at: usize,
}

fn object_block(file: &[u8], at: usize) -> Option<ObjectBlock> {
    let flag = be_u16(file, at + 8)?;
    let id_size = be_u16(file, at + 0x0A)? as usize;
    let id_at = at + 0x0C;
    let content_at = id_at.checked_add(align4(id_size))?;
    Some(ObjectBlock {
        flag,
        id_at,
        id_size,
        content_at,
    })
}

/// A `(const char*)` read of `bytes` from `at`: the reference hands its pointers into the file
/// buffer and lets `strcmp` stop at the next NUL anywhere after them, so this runs to the end of
/// the file rather than to the end of the block or paragraph the name lives in. `None` is a read
/// that found no terminator before the buffer ended, or bytes the port's name tables cannot
/// spell.
fn c_string(file: &[u8], at: usize) -> Option<&str> {
    let tail = file.get(at..)?;
    let end = tail.iter().position(|byte| *byte == 0)?;
    std::str::from_utf8(&tail[..end]).ok()
}

/// The same read over a slice that already ends where the reference's buffer does: an adaptor
/// paragraph's content is handed on as the file from its start, so this is [`c_string`] at 0.
fn c_string_slice(content: &[u8]) -> Option<&str> {
    c_string(content, 0)
}

/// The bytes a C-string read would look at, up to the first NUL, for a gap's payload.
fn name_bytes(content: &[u8]) -> Vec<u8> {
    content
        .iter()
        .copied()
        .take_while(|byte| *byte != 0)
        .collect()
}

fn align4(value: usize) -> usize {
    (value + 3) & !3
}

fn be_u16(bytes: &[u8], at: usize) -> Option<u16> {
    let end = at.checked_add(2)?;
    let slice = bytes.get(at..end)?;
    Some(u16::from_be_bytes([slice[0], slice[1]]))
}

fn be_u32(bytes: &[u8], at: usize) -> Option<u32> {
    let end = at.checked_add(4)?;
    let slice = bytes.get(at..end)?;
    Some(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    be_u32(bytes, at)
}

fn f32_at(bytes: &[u8], at: usize) -> Option<f32> {
    be_u32(bytes, at).map(f32::from_bits)
}
