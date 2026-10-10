//! GX blitting-processor register decode: the `.dff` frame state rebuilt into
//! the engine's material model.
//!
//! A recorded frame stores two raw images the GX command stream drove against:
//! `bp_mem`, the 256-entry blitting-processor register file ([`crate::dff`]), and
//! `tex_mem`, the 1 MiB of texture memory those registers pointed into. This
//! module reads the register file the way the GX SDK documents the registers,
//! not the way the J3D file format stores them.
//!
//! The two encodings are different and the difference matters. J3D's `MAT3`
//! (`j3d/bdl.rs:976`) carries each TEV stage in the *file* layout that
//! `J3DTevStage::setTevStageInfo` (`tww/src/JSystem/J3DGraphBase/J3DTevs.cpp:92`
//! via `tww/include/JSystem/J3DGraphBase/J3DTevs.h:92`) repacks into the
//! *register* layout `J3DTevStage::load` (`J3DTevs.h:112`) writes to the
//! hardware. `bp_mem` holds the register layout — the output of that repack —
//! so every bit offset below is a register offset, taken from the GX headers
//! that define the hardware fields, never a `MAT3` file offset.
//!
//! | GX register field | Spec authority |
//! |---|---|
//! | combiner/ksel/register packing | `tww/include/dolphin/gf/GFTev.h` |
//! | combiner operand *values* | `tww/include/dolphin/gx/GXEnum.h` |
//! | register *addresses* | `tww/include/dolphin/gx/GXEnum.h` `GXBPRegs` |
//! | texture/TREF field packing | not in the decomp headers; `gecko/crates/gecko/src/flipper/gx/regs.rs` |
//!
//! [`ShadingModel`] has no field for every GX register, so a field that changes
//! the shaded result but the engine cannot express becomes a typed
//! [`GxDecodeError`] rather than a silently dropped bit.

use std::fmt;

use repame_view3d::{DEFAULT_KCOLORS, KColors};
use repame_view3d::{ShadingModel, TevArg, TevBias, TevDest, TevOp, TevScale, TevStage};

use crate::assets::{
    GX_TF_C4X2, GX_TF_CI4, GX_TF_CI8, gc_texture_bytes, gc_texture_to_rgba, rgb565_to_rgba,
    rgb5a3_to_rgba,
};

const BP_MEM_LEN: usize = 256;

/// `GX_BP_REG_GENMODE = 0x0`, `tww/include/dolphin/gx/GXEnum.h:1310`.
const BP_GEN_MODE: usize = 0x00;
/// `GX_BP_REG_RAS1_TREF0 = 0x28`, `tww/include/dolphin/gx/GXEnum.h:1359`.
const BP_RAS1_TREF0: usize = 0x28;
/// `GX_BP_REG_TEVCOLORCOMBINER0 = 0xC0`, `tww/include/dolphin/gx/GXEnum.h:1496`.
const BP_TEV_COLOR_ENV0: usize = 0xC0;
/// `GX_BP_REG_TEVALPHACOMBINER0 = 0xC1`, `tww/include/dolphin/gx/GXEnum.h:1497`.
const BP_TEV_ALPHA_ENV0: usize = 0xC1;
/// `GX_BP_REG_TEVREG0LO = 0xE0`, `tww/include/dolphin/gx/GXEnum.h:1530`.
const BP_TEV_REGISTERL0: usize = 0xE0;
/// `GX_BP_REG_TEVKSEL0 = 0xF6`, `tww/include/dolphin/gx/GXEnum.h:1558`.
const BP_TEV_KSEL0: usize = 0xF6;
/// `GX_BP_REG_SETMODE0_TEX0 = 0x80`, `tww/include/dolphin/gx/GXEnum.h:1428`.
const BP_TX_SETMODE0_I0: usize = 0x80;
/// `GX_BP_REG_SETMODE0_TEX4 = 0xA0`, `tww/include/dolphin/gx/GXEnum.h:1462`.
const BP_TX_SETMODE0_I4: usize = 0xA0;
/// `GX_BP_REG_SETIMAGE0_TEX0 = 0x88`, `tww/include/dolphin/gx/GXEnum.h:1438`.
const BP_TX_SETIMAGE0_I0: usize = 0x88;
/// `GX_BP_REG_SETIMAGE3_TEX0 = 0x94`, `tww/include/dolphin/gx/GXEnum.h:1450`.
const BP_TX_SETIMAGE3_I0: usize = 0x94;
/// `GX_BP_REG_SETIMAGE0_TEX4 = 0xA8`, `tww/include/dolphin/gx/GXEnum.h:1472`.
const BP_TX_SETIMAGE0_I4: usize = 0xA8;
/// `GX_BP_REG_SETIMAGE3_TEX4 = 0xB4`, `tww/include/dolphin/gx/GXEnum.h:1484`.
const BP_TX_SETIMAGE3_I4: usize = 0xB4;
/// `GX_BP_REG_SETTLUT_TEX0 = 0x98`, `tww/include/dolphin/gx/GXEnum.h:1456`.
const BP_TX_SETTLUT_I0: usize = 0x98;
/// `GX_BP_REG_SETTLUT_TEX4 = 0xB8`, `tww/include/dolphin/gx/GXEnum.h:1490`.
const BP_TX_SETTLUT_I4: usize = 0xB8;

/// Stage count ceiling; `repame_view3d::MAX_TEV_STAGES`.
const TEV_STAGE_MAX: usize = 16;

/// Palette TMEM is addressed in 256-entry units,
/// `gecko/crates/gecko/src/flipper/gx/constants.rs:115`.
const TLUT_ENTRIES_PER_UNIT: usize = 256;
/// TLUT format codes, `gecko/crates/gecko/src/flipper/gx/draw.rs:69`.
const TLUT_RGB565: u32 = 1;
const TLUT_RGB5A3: u32 = 2;

/// A single bit run of a 24-bit blitting-processor word, `[low, high]`.
#[derive(Clone, Copy)]
struct Field {
    low: u32,
    high: u32,
}

impl Field {
    const fn new(low: u32, high: u32) -> Self {
        Self { low, high }
    }

    fn get(&self, word: u32) -> u32 {
        (word >> self.low) & ((1 << (self.high - self.low + 1)) - 1)
    }
}

/// `BP_TEV_COLOR` field runs, `tww/include/dolphin/gf/GFTev.h:7`.
/// `d@0 c@4 b@8 a@12 bias@16 op@18 clamp@19 scale@20 out@22 id@24`.
const TEV_A: Field = Field::new(12, 15);
const TEV_B: Field = Field::new(8, 11);
const TEV_C: Field = Field::new(4, 7);
const TEV_D: Field = Field::new(0, 3);
const TEV_BIAS: Field = Field::new(16, 17);
const TEV_OP: Field = Field::new(18, 18);
const TEV_CLAMP: Field = Field::new(19, 19);
const TEV_SCALE: Field = Field::new(20, 21);
const TEV_OUT: Field = Field::new(22, 23);

/// `BP_TEV_ALPHA` field runs, `tww/include/dolphin/gf/GFTev.h:21`.
/// `ras_sel@0 tex_sel@2 d@4 c@7 b@10 a@13 bias@16 op@18 clamp@19 scale@20 out@22 id@24`.
const TEVA_RAS_SEL: Field = Field::new(0, 1);
const TEVA_TEX_SEL: Field = Field::new(2, 3);
const TEVA_D: Field = Field::new(4, 6);
const TEVA_C: Field = Field::new(7, 9);
const TEVA_B: Field = Field::new(10, 12);
const TEVA_A: Field = Field::new(13, 15);

/// `BP_TEV_KSEL` field runs, `tww/include/dolphin/gf/GFTev.h:53`. Each word
/// covers two stages: `kcsel0@4 kasel0@9 kcsel1@14 kasel1@19`.
const KSEL_EVEN: (Field, Field) = (Field::new(4, 8), Field::new(9, 13));
const KSEL_ODD: (Field, Field) = (Field::new(14, 18), Field::new(19, 23));

/// `GenMode` active TEV stage count, field packing from
/// `gecko/crates/gecko/src/flipper/gx/regs.rs:905`. The decomp's
/// `GXBPGenMode` (`GXEnum.h:1184`) uses a different layout and names no stage
/// count, so this field is not in the decomp.
const GENMODE_NUMTEV: Field = Field::new(10, 13);

/// `TevOrder` texture-slot binding, `TevOrder` bit packing from
/// `gecko/crates/gecko/src/flipper/gx/regs.rs:638`. `texmap@0..=2` binds the
/// even stage, `texmap1@12..=14` the odd one; the texcoord and raster-channel
/// halves have no `TevStage` field and are reported as gaps rather than read.
const TREF0_TEXMAP: Field = Field::new(0, 2);
const TREF1_TEXMAP: Field = Field::new(12, 14);

/// `TxSetImage0`, `gecko/crates/gecko/src/flipper/gx/regs.rs:577`.
/// `width@0..=9 height@10..=19 format@20..=23`.
const IMAGE_W: Field = Field::new(0, 9);
const IMAGE_H: Field = Field::new(10, 19);
const IMAGE_FORMAT: Field = Field::new(20, 23);

/// `TxSetMode0`, `gecko/crates/gecko/src/flipper/gx/regs.rs:684`.
/// `wrap_s@0..=1 wrap_t@2..=3`.
const MODE_WRAP_S: Field = Field::new(0, 1);
const MODE_WRAP_T: Field = Field::new(2, 3);

/// `TevRegisterL`, `gecko/crates/gecko/src/flipper/gx/regs.rs:881`.
/// `r@0..=10 a@12..=22 reg_type@23`. `TevRegisterH` (`regs.rs:895`) swaps the
/// pair: `b@0..=10 g@12..=22 reg_type@23`.
const TREG_R: Field = Field::new(0, 10);
const TREG_A: Field = Field::new(12, 22);
const TREG_B: Field = Field::new(0, 10);
const TREG_G: Field = Field::new(12, 22);
/// Constant vs color bank split. `reg_type == 1` is the constant bank the
/// material's `kcolors` reads; 0 is the runtime TEV register file.
const TREG_TYPE: Field = Field::new(23, 23);

/// Why a blitting-processor register file could not become a material.
///
/// A GX field that shapes the shaded result but [`ShadingModel`] cannot hold
/// becomes one of these rather than being dropped: the engine folds a stage
/// from `reg = clamp(scale * (d [+-] ((1 - c) a + c b) + bias))` and has no
/// operand for, say, the fixed `7/8` gray a constant selector can name, nor a
/// channel-swap mode. Naming the field keeps the caller from silently
/// rendering a wrong image where the hardware would not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GxDecodeError {
    /// A `KONST` operand whose selector names a value the engine's [`TevArg`]
    /// A combiner bias code 3, the r-compare mode `repame_view3d::TevBias`
    /// does not model (`tww/include/dolphin/gf/GFTev.h:13` packs it, but the
    /// engine's arithmetic `code()`/`from_code()` admit only 0..2).
    Bias {
        stage: usize,
        alpha: bool,
    },
}

impl fmt::Display for GxDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bias { stage, alpha } => write!(
                f,
                "tev stage {stage} {}bias code 3 (compare) has no engine TevBias",
                if *alpha { "alpha " } else { "" }
            ),
        }
    }
}

impl std::error::Error for GxDecodeError {}

/// Decoded texture descriptor from a `GX_BP_REG_TEX*/TREF` slot.
///
/// A GX texture is spread over six registers — `TxSetImage0..3`
/// (size, format, address), `TxSetMode0/1` (wrap, filter) and the palette
/// binding — so one [`TexDescriptor`] gathers the fields the engine needs to
/// name the texture again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TexDescriptor {
    /// Byte offset into the frame's `tex_mem` image, `image_base << 5`
    /// (`TxSetImage3::ram_addr`,
    /// `gecko/crates/gecko/src/flipper/gx/regs.rs:595`).
    pub tmem_addr: u32,
    /// `image_base` in its native 32-byte units,
    /// `TxSetImage3` `gecko/crates/gecko/src/flipper/gx/regs.rs:591`.
    pub tmem_entries: u32,
    /// Stored as `width - 1`, `TxSetImage0` `regs.rs:578`.
    pub width: u16,
    /// Stored as `height - 1`, `TxSetImage0` `regs.rs:580`.
    pub height: u16,
    /// `GX_TF_*` code, `TxSetImage0` `regs.rs:584`, values
    /// `tww/include/dolphin/gx/GXEnum.h` / `ClearSleeper/src/assets.rs:805`.
    pub format: u8,
    /// `GX_REPEAT`-family code (clamp 0, repeat 1, mirror 2), `TxSetMode0`
    /// `regs.rs:684`.
    pub wrap_s: u8,
    /// See [`Self::wrap_s`], `TxSetMode0` `regs.rs:687`.
    pub wrap_t: u8,
    /// Packed palette binding: `tmem_offset << 2 | format`, where `tmem_offset`
    /// is in 256-entry units and `format` is a TLUT code (IA8 0, RGB565 1,
    /// RGB5A3 2), `gecko/crates/gecko/src/flipper/gx/draw.rs:99`.
    pub palette: u32,
}

/// Decoded blitting-processor register file: the 256 words a `.dff` stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BpState {
    pub words: [u32; BP_MEM_LEN],
}

impl BpState {
    /// Build from a `.dff`'s `bp_mem` (`crate::dff::DffFile::bp_mem`), which is
    /// `Vec<u32>` little-endian as stored (`crate::dff` reads each word with
    /// `u32::from_le_bytes`). A shorter input zero-fills; a longer one is
    /// truncated to the register file.
    pub fn from_words(words: &[u32]) -> Self {
        let mut slots = [0; BP_MEM_LEN];
        for (slot, word) in slots.iter_mut().zip(words.iter()) {
            *slot = *word;
        }
        Self { words: slots }
    }

    /// A 24-bit register value. BP writes carry the register id in the top byte
    /// of the wire dword (`j3d/fifo.rs` reports `id = value >> 24`), and the
    /// `.dff` `bp_mem` already dropped it, so both sources are masked here and
    /// every field extraction below reads a clean register value.
    fn word(&self, reg: usize) -> u32 {
        self.words.get(reg).copied().unwrap_or(0) & 0x00FF_FFFF
    }

    /// TEV stage count the register file declares, at least one so a cleared
    /// frame still yields a material rather than an empty program.
    fn stage_count(&self) -> usize {
        let declared = GENMODE_NUMTEV.get(self.word(BP_GEN_MODE)) as usize + 1;
        declared.clamp(1, TEV_STAGE_MAX)
    }

    /// Parse the TEV stage registers into the engine's material, or name the
    /// first GX field [`ShadingModel`] cannot express.
    ///
    /// Reads the color and alpha combiners (`GX_BP_REG_TEV*COMBINER*`,
    /// `GXEnum.h:1496`), the constant registers (`GX_BP_REG_TEVREG*LO/HI`,
    /// `GXEnum.h:1530`) and the selectors (`GX_BP_REG_TEVKSEL*`,
    /// `GXEnum.h:1558`).
    ///
    /// Deliberately not folded in, because [`ShadingModel`] holds no field for
    /// them or they belong to another unit: the indirect texture stages
    /// (`GX_BP_REG_INDTEVSTAGE*`, 0x10..0x1F) and their matrices (0x06..0x0E),
    /// the pipeline blend (`GX_BP_REG_BLENDMODE`, 0x41), Z-mode (0x40) and
    /// alpha-compare (0xF3), the fog block (0xE8..0xF2), the EFB copy block
    /// (0x49..0x52) and the scissor registers, and the XF-side lighting and
    /// ambient that live in `xf_mem`/`xf_regs` rather than `bp_mem`, which is
    /// why `lights` stays empty and `ambient` stays 0. The TEV color bank
    /// (`reg_type == 0` of the TEV registers) is likewise left out: it is the
    /// runtime register file the evaluator owns, and a static material cannot
    /// carry pre-loaded values for it.
    ///
    /// Note also that `bp_mem` is frame-final state, shared by every draw call
    /// in the frame: this reads whichever material was bound last, so a
    /// per-material reconstruction needs a per-draw snapshot of the register
    /// file taken from the FIFO command stream instead.
    pub fn shading_model(&self) -> Result<ShadingModel, GxDecodeError> {
        let count = self.stage_count();
        let mut stages = Vec::with_capacity(count);
        for stage in 0..count {
            stages.push(self.stage(stage)?);
        }
        Ok(ShadingModel {
            stages,
            kcolors: self.kcolors(),
            ..ShadingModel::default()
        })
    }

    /// One TEV stage from its color and alpha combiner words.
    fn stage(&self, stage: usize) -> Result<TevStage, GxDecodeError> {
        let color = self.word(BP_TEV_COLOR_ENV0 + stage * 2);
        let alpha = self.word(BP_TEV_ALPHA_ENV0 + stage * 2);

        let (ras_sel, tex_sel) = (TEVA_RAS_SEL.get(alpha), TEVA_TEX_SEL.get(alpha));
        let ksel = Ksel::read(self, stage);
        let tev = TevStage {
            tex_unit: self.tex_unit(stage),
            color_arg: [
                color_arg(TEV_A.get(color), ksel.color, stage)?,
                color_arg(TEV_B.get(color), ksel.color, stage)?,
                color_arg(TEV_C.get(color), ksel.color, stage)?,
                color_arg(TEV_D.get(color), ksel.color, stage)?,
            ],
            alpha_arg: [
                alpha_arg(TEVA_A.get(alpha), ksel.alpha, stage)?,
                alpha_arg(TEVA_B.get(alpha), ksel.alpha, stage)?,
                alpha_arg(TEVA_C.get(alpha), ksel.alpha, stage)?,
                alpha_arg(TEVA_D.get(alpha), ksel.alpha, stage)?,
            ],
            color_op: TevOp::from_code(TEV_OP.get(color) as u8).unwrap_or(TevOp::Add),
            alpha_op: TevOp::from_code(TEV_OP.get(alpha) as u8).unwrap_or(TevOp::Add),
            color_bias: bias(TEV_BIAS.get(color), stage, false)?,
            alpha_bias: bias(TEV_BIAS.get(alpha), stage, true)?,
            color_scale: TevScale::from_code(TEV_SCALE.get(color) as u8).unwrap_or(TevScale::X1),
            alpha_scale: TevScale::from_code(TEV_SCALE.get(alpha) as u8).unwrap_or(TevScale::X1),
            color_clamp: TEV_CLAMP.get(color) != 0,
            alpha_clamp: TEV_CLAMP.get(alpha) != 0,
            color_dest: TevDest::from_code(TEV_OUT.get(color) as u8).unwrap_or(TevDest::Prev),
            alpha_dest: TevDest::from_code(TEV_OUT.get(alpha) as u8).unwrap_or(TevDest::Prev),
            ras_sel: ras_sel.min(3) as u8,
            tex_sel: tex_sel.min(3) as u8,
            ..TevStage::default()
        };
        Ok(tev)
    }

    /// Texture unit this stage samples: the `texmap` half of its `RAS1_TREF*`
    /// order word, which binds each stage to a texture slot.
    fn tex_unit(&self, stage: usize) -> u8 {
        let order = self.word(BP_RAS1_TREF0 + stage / 2);
        if stage % 2 == 0 {
            TREF0_TEXMAP.get(order) as u8
        } else {
            TREF1_TEXMAP.get(order) as u8
        }
    }

    /// The four constant color registers, from `GX_BP_REG_TEVREG*LO/HI`
    /// (`GXEnum.h:1530`, packing `GFTev.h:37`).
    ///
    /// `TevRegisterL` (`regs.rs:881`) holds `r@0..=10` and `a@12..=22`,
    /// `TevRegisterH` (`regs.rs:895`) holds `b@0..=10` and `g@12..=22`, both
    /// 11-bit signed values the hardware scales by `1/255`
    /// (`gecko/crates/gecko/src/flipper/gx/tev.rs:64`). Bit 23 of the low word
    /// is `reg_type`: 1 marks the constant bank the material's kcolors read, 0
    /// marks the color bank that is the runtime register file the evaluator
    /// owns, so a slot left at 0 keeps the engine default rather than reading a
    /// constant the game never set.
    fn kcolors(&self) -> KColors {
        let mut kcolors = DEFAULT_KCOLORS;
        for reg in 0..4 {
            let low = self.word(BP_TEV_REGISTERL0 + reg * 2);
            let high = self.word(BP_TEV_REGISTERL0 + reg * 2 + 1);
            if TREG_TYPE.get(low) == 0 {
                continue;
            }
            kcolors[reg] = [
                s11_to_f32(TREG_R.get(low)),
                s11_to_f32(TREG_G.get(high)),
                s11_to_f32(TREG_B.get(high)),
                s11_to_f32(TREG_A.get(low)),
            ];
        }
        kcolors
    }

    /// Decode a texture descriptor from TMEM registers for `slot` (0..8).
    pub fn tex_descriptor(&self, slot: usize) -> Option<TexDescriptor> {
        if slot >= 8 {
            return None;
        }
        let image0 = self.word(slot_reg(BP_TX_SETIMAGE0_I0, BP_TX_SETIMAGE0_I4, slot));
        let image3 = self.word(slot_reg(BP_TX_SETIMAGE3_I0, BP_TX_SETIMAGE3_I4, slot));
        let mode0 = self.word(slot_reg(BP_TX_SETMODE0_I0, BP_TX_SETMODE0_I4, slot));
        let tlut = self.word(slot_reg(BP_TX_SETTLUT_I0, BP_TX_SETTLUT_I4, slot));

        Some(TexDescriptor {
            tmem_addr: image3 << 5,
            tmem_entries: image3,
            width: IMAGE_W.get(image0) as u16 + 1,
            height: IMAGE_H.get(image0) as u16 + 1,
            format: IMAGE_FORMAT.get(image0) as u8,
            wrap_s: MODE_WRAP_S.get(mode0) as u8,
            wrap_t: MODE_WRAP_T.get(mode0) as u8,
            palette: pack_tlut(tlut),
        })
    }
}

/// `GX_BP_REG_TEVKSEL*`: which constant each `KONST` operand reads. Stage N
/// reads `GX_BP_REG_TEVKSEL0 + N/2` (`F6`..`FD`, `GXEnum.h:1558`), even stages
/// taking the `*0` selectors and odd the `*1`
/// (`gecko/crates/gecko/src/flipper/gx/tev.rs:50`), packing from `GFTev.h:53`.
struct Ksel {
    color: u32,
    alpha: u32,
}

impl Ksel {
    fn read(state: &BpState, stage: usize) -> Self {
        let word = state.word(BP_TEV_KSEL0 + stage / 2);
        let (color, alpha) = if stage % 2 == 0 { KSEL_EVEN } else { KSEL_ODD };
        Self {
            color: color.get(word),
            alpha: alpha.get(word),
        }
    }
}

/// 11-bit signed fixed point to a normalized float, sign-extended then divided
/// by 255, as `gecko/crates/gecko/src/flipper/gx/tev.rs:64` does for the
/// constant registers.
fn s11_to_f32(value: u32) -> f32 {
    let value = value & 0x7FF;
    let signed = if value & 0x400 != 0 {
        value as i32 - 0x800
    } else {
        value as i32
    };
    signed as f32 / 255.0
}

/// Map a color-combiner operand code to the engine's [`TevArg`].
///
/// Codes are `GX_CC_*` (`GXEnum.h:293`): `CPREV` 0, `APREV` 1, `C0`/`A0` 2/3,
/// `C1`/`A1` 4/5, `C2`/`A2` 6/7, `TEXC` 8, `TEXA` 9, `RASC` 0xA, `RASA` 0xB,
/// `ONE` 0xC, `HALF` 0xD, `KONST` 0xE, `ZERO` 0xF.
fn color_arg(code: u32, ksel: u32, stage: usize) -> Result<TevArg, GxDecodeError> {
    match code {
        0x0 => Ok(TevArg::Color),
        0x1 => Ok(TevArg::Alpha),
        0x2 => Ok(TevArg::Reg0),
        0x3 => Ok(TevArg::Reg0),
        0x4 => Ok(TevArg::Reg1),
        0x5 => Ok(TevArg::Reg1),
        0x6 => Ok(TevArg::Reg2),
        0x7 => Ok(TevArg::Reg2),
        0x8 => Ok(TevArg::TexColor),
        0x9 => Ok(TevArg::TexAlpha),
        0xA => Ok(TevArg::Rasc),
        0xB => Ok(TevArg::Rasa),
        0xC => Ok(TevArg::One),
        0xD => Ok(TevArg::Half),
        0xE => Ok(konst_arg(ksel, false, stage)),
        _ => Ok(TevArg::Zero),
    }
}

/// Map an alpha-combiner operand code to the engine's [`TevArg`].
///
/// Codes are `GX_CA_*` (`GXEnum.h:335`): `APREV` 0, `A0`/`A1`/`A2` 1/2/3,
/// `TEXA` 4, `RASA` 5, `KONST` 6, `ZERO` 7.
fn alpha_arg(code: u32, ksel: u32, stage: usize) -> Result<TevArg, GxDecodeError> {
    match code {
        0x0 => Ok(TevArg::Alpha),
        0x1 => Ok(TevArg::Reg0),
        0x2 => Ok(TevArg::Reg1),
        0x3 => Ok(TevArg::Reg2),
        0x4 => Ok(TevArg::TexAlpha),
        0x5 => Ok(TevArg::Rasa),
        0x6 => Ok(konst_arg(ksel, true, stage)),
        _ => Ok(TevArg::Zero),
    }
}

/// Map a 5-bit `KONST` selector to the engine's [`TevArg`].
///
/// The two hardware tables differ past the grays: only the color table has
/// whole-register selects at `0x0C..=0x0F` (`GX_TEV_KCSEL_K0 = 0x0C`,
/// `GXEnum.h:545`), while the alpha table starts its register selects at `0x10`
/// (`GX_TEV_KASEL_K0_R = 0x10`, `GXEnum.h:518`). Applying one table to both
/// would mis-read every alpha KONST.
fn konst_arg(sel: u32, alpha: bool, _stage: usize) -> TevArg {
    match sel {
        0x00..=0x07 => TevArg::Konst(sel as u8),
        0x0C..=0x0F if !alpha => TevArg::KColor((sel - 0x0C) as u8),
        0x10..=0x1F => TevArg::Konst(sel as u8),
        _ => TevArg::Zero,
    }
}

fn bias(code: u32, stage: usize, alpha: bool) -> Result<TevBias, GxDecodeError> {
    TevBias::from_code(code as u8).ok_or(GxDecodeError::Bias { stage, alpha })
}

/// Texture slot 0..3 uses the `*_I0` base, 4..7 the `*_I4` base 0x20 on,
/// `tx_slot_reg`,
/// `gecko/crates/gecko/src/flipper/gx/bp.rs:735`.
fn slot_reg(base_i0: usize, base_i4: usize, slot: usize) -> usize {
    if slot < 4 { base_i0 + slot } else { base_i4 + (slot - 4) }
}

/// Pack a `TX_SETTLUT` word into [`TexDescriptor::palette`]'s
/// `tmem_offset << 2 | format` form, `gecko/crates/gecko/src/flipper/gx/draw.rs:99`.
fn pack_tlut(word: u32) -> u32 {
    let tmem_offset = word & 0x3FF;
    let format = (word >> 10) & 0x3;
    (tmem_offset << 2) | format
}

/// Decode a texture out of the `.dff`'s `tex_mem` image into RGBA8.
///
/// Reads `gc_texture_bytes(format, w, h)` bytes at `desc.tmem_addr`, builds the
/// palette for indexed formats out of the same image at the TLUT offset, and
/// hands both to `gc_texture_to_rgba` (`crate::assets`). Returns `None` when
/// the format is unknown or the image is too short for the declared size.
pub fn decode_tmem_texture(tex_mem: &[u8], desc: &TexDescriptor) -> Option<Vec<u8>> {
    let width = usize::from(desc.width);
    let height = usize::from(desc.height);
    let needed = gc_texture_bytes(desc.format, width, height)?;
    let start = desc.tmem_addr as usize;
    let bytes = tex_mem.get(start..start.checked_add(needed)?)?;
    let palette = tmem_palette(tex_mem, desc);
    gc_texture_to_rgba(desc.format, width, height, bytes, &palette)
}

/// Palette entries for an indexed texture, read from the TLUT region of
/// `tex_mem`. The byte base is `tmem_offset * 512` (256 `u16` entries per
/// unit), `slot_palette`,
/// `gecko/crates/gecko/src/flipper/gx/bp.rs:740`.
fn tmem_palette(tex_mem: &[u8], desc: &TexDescriptor) -> Vec<[u8; 4]> {
    let entries = match desc.format {
        GX_TF_CI4 => 16,
        GX_TF_CI8 => 256,
        GX_TF_C4X2 => 16_384,
        _ => return Vec::new(),
    };
    let tmem_offset = (desc.palette >> 2) & 0x3FF;
    let format = desc.palette & 0x3;
    let base = (tmem_offset as usize) * (TLUT_ENTRIES_PER_UNIT * 2);
    let mut palette = Vec::with_capacity(entries);
    for index in 0..entries {
        let at = base + index * 2;
        let Some(pair) = tex_mem.get(at..at + 2) else {
            break;
        };
        let texel = u16::from_be_bytes([pair[0], pair[1]]);
        palette.push(match format {
            TLUT_RGB565 => rgb565_to_rgba(texel),
            TLUT_RGB5A3 => rgb5a3_to_rgba(texel),
            _ => {
                let intensity = (texel >> 8) as u8;
                [intensity, intensity, intensity, texel as u8]
            }
        });
    }
    palette
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(words: &mut [u32], reg: usize, value: u32) {
        words[reg] = value & 0x00FF_FFFF;
    }

    /// The word `BP_TEV_COLOR` packs, `tww/include/dolphin/gf/GFTev.h:7`.
    fn color_env(a: u32, b: u32, c: u32, d: u32, bias: u32, op: u32, clamp: u32, scale: u32, dest: u32) -> u32 {
        (d & 0xF)
            | ((c & 0xF) << 4)
            | ((b & 0xF) << 8)
            | ((a & 0xF) << 12)
            | ((bias & 0x3) << 16)
            | ((op & 0x1) << 18)
            | ((clamp & 0x1) << 19)
            | ((scale & 0x3) << 20)
            | ((dest & 0x3) << 22)
    }

    /// The word `BP_TEV_ALPHA` packs, `tww/include/dolphin/gf/GFTev.h:21`.
    /// The `ras_sel`/`tex_sel` pair stays zero, so the swap error never trips.
    fn alpha_env(a: u32, b: u32, c: u32, d: u32, bias: u32, op: u32, clamp: u32, scale: u32, dest: u32) -> u32 {
        (d & 0x7) << 4
            | ((c & 0x7) << 7)
            | ((b & 0x7) << 10)
            | ((a & 0x7) << 13)
            | ((bias & 0x3) << 16)
            | ((op & 0x1) << 18)
            | ((clamp & 0x1) << 19)
            | ((scale & 0x3) << 20)
            | ((dest & 0x3) << 22)
    }

    #[test]
    fn one_stage_round_trips_every_field() {
        let mut words = [0_u32; 256];
        write(&mut words, BP_GEN_MODE, 0);
        write(
            &mut words,
            BP_TEV_COLOR_ENV0,
            color_env(0xA, 0xD, 0x2, 0xF, 1, 0, 1, 1, 2),
        );
        write(
            &mut words,
            BP_TEV_ALPHA_ENV0,
            alpha_env(0x5, 0x4, 0x7, 0x1, 2, 1, 1, 3, 3),
        );

        let model = BpState::from_words(&words[..])
            .shading_model()
            .expect("hand-built stage decodes");

        assert_eq!(model.stages.len(), 1);
        let stage = model.stages[0];

        assert_eq!(
            stage.color_arg,
            [TevArg::Rasc, TevArg::Half, TevArg::Reg0, TevArg::Zero]
        );
        assert_eq!(
            stage.alpha_arg,
            [TevArg::Rasa, TevArg::TexAlpha, TevArg::Zero, TevArg::Reg0]
        );
        assert_eq!(stage.color_op, TevOp::Add);
        assert_eq!(stage.alpha_op, TevOp::Sub);
        assert_eq!(stage.color_bias, TevBias::AddHalf);
        assert_eq!(stage.alpha_bias, TevBias::SubHalf);
        assert_eq!(stage.color_scale, TevScale::X2);
        assert_eq!(stage.alpha_scale, TevScale::Divide2);
        assert_eq!(stage.color_dest, TevDest::Reg1);
        assert_eq!(stage.alpha_dest, TevDest::Reg2);
        assert!(stage.color_clamp);
        assert!(stage.alpha_clamp);
        model.validate().expect("program validates");
    }

    #[test]
    fn konst_operand_maps_selector_to_kcolor() {
        let mut words = [0_u32; 256];
        write(&mut words, BP_GEN_MODE, 0);
        write(
            &mut words,
            BP_TEV_COLOR_ENV0,
            color_env(0xE, 0xE, 0xE, 0xE, 0, 0, 0, 0, 0),
        );
        write(
            &mut words,
            BP_TEV_ALPHA_ENV0,
            alpha_env(0x6, 0x6, 0x6, 0x6, 0, 0, 0, 0, 0),
        );
        write(&mut words, BP_TEV_KSEL0, (0xC << 4) | (0x10 << 9));

        let stage = BpState::from_words(&words[..])
            .shading_model()
            .expect("konst stage decodes").stages[0];

        // Color selects a whole register at `0x0C`; alpha starts its register
        // selects at `0x10`, which is the per-component `K0_R`.
        assert_eq!(stage.color_arg, [TevArg::KColor(0); 4]);
        assert_eq!(stage.alpha_arg, [TevArg::Konst(0x10); 4]);
    }

    #[test]
    fn konst_fixed_gray_maps_to_the_selector() {
        let mut words = [0_u32; 256];
        write(&mut words, BP_GEN_MODE, 0);
        write(
            &mut words,
            BP_TEV_COLOR_ENV0,
            color_env(0x0, 0x0, 0x0, 0xE, 0, 0, 0, 0, 0),
        );
        write(&mut words, BP_TEV_KSEL0, 1 << 4);

        let model = BpState::from_words(&words[..])
            .shading_model()
            .expect("7/8 gray is a Konst selector");
        assert_eq!(model.stages[0].color_arg[3], TevArg::Konst(1));
    }

    #[test]
    fn channel_swap_selects_the_channels() {
        let mut words = [0_u32; 256];
        write(&mut words, BP_GEN_MODE, 0);
        write(
            &mut words,
            BP_TEV_COLOR_ENV0,
            color_env(0x0, 0x0, 0x0, 0x0, 0, 0, 0, 0, 0),
        );
        write(&mut words, BP_TEV_ALPHA_ENV0, 0b11);

        let model = BpState::from_words(&words[..])
            .shading_model()
            .expect("swap selects channels");
        // `ras_sel = 3` (APREV reads rasterized alpha) and `tex_sel = 0`
        // (TEXA reads the texel's red).
        assert_eq!(model.stages[0].ras_sel, 3);
        assert_eq!(model.stages[0].tex_sel, 0);
    }

    #[test]
    fn constant_bank_feeds_kcolors_only() {
        let mut words = [0_u32; 256];
        write(&mut words, BP_GEN_MODE, 0);
        write(
            &mut words,
            BP_TEV_REGISTERL0,
            (1_u32 << 23) | 0x0FF | (0x0FF << 12),
        );
        write(&mut words, BP_TEV_REGISTERL0 + 1, 0x0FF | (0x0FF << 12));
        write(&mut words, BP_TEV_REGISTERL0 + 2, 0x0FF | (0x0FF << 12));

        let model = BpState::from_words(&words[..])
            .shading_model()
            .expect("registers decode");

        assert_eq!(model.kcolors[0], [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(model.kcolors[1], DEFAULT_KCOLORS[1]);
    }

    #[test]
    fn texture_descriptor_and_rgba_decode() {
        let mut words = [0_u32; 256];
        write(&mut words, 0x88, (7 & 0x3FF) | ((7 & 0x3FF) << 10) | (4 << 20));
        write(&mut words, 0x94, 0x10);
        write(&mut words, 0x80, 0 | (1 << 2));
        write(&mut words, 0x98, 2_u32 << 10);

        let state = BpState::from_words(&words[..]);
        let desc = state.tex_descriptor(0).expect("slot 0 decodes");
        assert_eq!((desc.width, desc.height), (8, 8));
        assert_eq!(desc.format, 4);
        assert_eq!((desc.wrap_s, desc.wrap_t), (0, 1));
        assert_eq!(desc.tmem_addr, 0x10 * 32);
        assert_eq!(desc.tmem_entries, 0x10);

        let mut tex_mem = vec![0_u8; 1024 * 1024];
        let at = desc.tmem_addr as usize;
        tex_mem[at..at + 2].copy_from_slice(&[0xF8, 0x00]);

        let rgba = decode_tmem_texture(&tex_mem, &desc).expect("rgb565 decodes");
        assert_eq!(rgba.len(), 8 * 8 * 4);
        assert_eq!(&rgba[0..4], &[255, 0, 0, 255]);
    }

    #[test]
    fn garbage_register_file_never_panics() {
        let mut words = [0_u32; 256];
        for (index, word) in words.iter_mut().enumerate() {
            *word = (index as u32).wrapping_mul(0x00C0_FFEE) & 0x00FF_FFFF;
        }
        match BpState::from_words(&words[..]).shading_model() {
            Ok(model) => assert!(!model.stages.is_empty()),
            Err(error) => assert!(error.to_string().contains("stage")),
        }
    }

    #[test]
    fn cleared_register_file_is_a_valid_default_material() {
        let model = BpState::from_words(&[])
            .shading_model()
            .expect("a cleared frame yields one default stage");
        assert_eq!(model.stages.len(), 1);
        model.validate().expect("default program validates");
    }

    #[test]
    fn out_of_range_slot_and_format_are_none() {
        let state = BpState::from_words(&[0_u32; 256]);
        assert!(state.tex_descriptor(8).is_none());

        let garbage = TexDescriptor {
            tmem_addr: 0,
            tmem_entries: 0,
            width: 8,
            height: 8,
            format: 0x7,
            wrap_s: 0,
            wrap_t: 0,
            palette: 0,
        };
        assert!(decode_tmem_texture(&vec![0_u8; 1024], &garbage).is_none());
    }

    #[test]
    fn short_input_zero_fills() {
        let state = BpState::from_words(&[0xABCD]);
        assert_eq!(state.words[0], 0xABCD);
        assert_eq!(state.words[1], 0);
        assert_eq!(state.words[BP_GEN_MODE], 0xABCD);
    }

    #[test]
    fn fifo_wire_value_with_register_id_decodes() {
        // `j3d/fifo.rs:167` reports a BP write as `id << 24 | data`, so a
        // replay building `bp_mem` stores that whole dword; the id byte must
        // not bleed into the fields or the `<< 5` image address.
        let mut words = [0_u32; 256];
        words[BP_GEN_MODE] = 0;
        words[BP_TEV_COLOR_ENV0] = (BP_TEV_COLOR_ENV0 as u32) << 24;
        words[BP_TEV_ALPHA_ENV0] = (BP_TEV_ALPHA_ENV0 as u32) << 24;
        words[0x88] = (7 | (7 << 10) | (4 << 20)) | (0x88_u32 << 24);
        words[0x94] = 0x10 | (0x94_u32 << 24);

        let state = BpState::from_words(&words[..]);
        let model = state.shading_model().expect("id-tagged stage decodes");
        assert_eq!(model.stages.len(), 1);
        assert_eq!(model.stages[0].color_arg, [TevArg::Color; 4]);
        assert_eq!(model.stages[0].alpha_arg, [TevArg::Alpha; 4]);

        let desc = state.tex_descriptor(0).expect("slot 0 decodes");
        assert_eq!((desc.width, desc.height, desc.format), (8, 8, 4));
        assert_eq!(desc.tmem_addr, 0x10 * 32);
        assert_eq!(desc.tmem_entries, 0x10);
    }
}

