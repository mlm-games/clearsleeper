//! The vertex format a draw's payload is laid out in.
//!
//! A GX stream never says how big a vertex is. Two CP registers do: `VCD_lo` and
//! `VCD_Hi` say which attributes are present and whether each is `DIRECT` or an
//! `INDEX8`/`INDEX16` reference, and the `VAT_A..C` tables give the component
//! count and type of each `DIRECT` one. Only together do they yield a stride,
//! and the stride is what lets a replay step over a payload.
//!
//! An indexed attribute (`GX_INDEX8`/`GX_INDEX16`) carries an index instead of
//! a value, and the value lives in one of the CP's sixteen arrays, declared by
//! `GXSetArray` with a base and a stride (`tww/src/dolphin/gx/GXAttr.c:469`,
//! `GX_CP_REG_ARRAYBASE = 0xA0` / `GX_CP_REG_ARRAYSTRIDE = 0xB0`,
//! `tww/include/dolphin/gx/GXEnum.h:955-956`). A row of that array holds the
//! value in the very component count, type and fraction the `VAT` gives the
//! attribute, because the SDK sizes an array value by the same
//! `GXCompCnt`/`GXCompType` the vertex format carries (`GXSetVtxAttrFmt`,
//! `GXAttr.c:246`). So [`Element`] keeps the value's width apart from the
//! index's.
//!
//! The runs below are the CP register file's. `GXGetVtxDesc` and
//! `GXGetVtxAttrFmt` (`tww/src/dolphin/gx/GXAttr.c:194`, `:377`) read back the
//! same words `SETVCDATTR`/`SETVAT` (`:4`, `:246`) wrote, and `GX_BITFIELD_SET`
//! (`tww/include/dolphin/gx/GX.h:29`) numbers a field position from the MSB, so
//! an entry the `GX_CP_*` enumerations list as bits `[a-b]` sits at register
//! bits `31-b..31-a`. Dolphin carries the same layout LSB-first in
//! `TVtxDesc`/`UVAT_group0..2`
//! (`dolphin/Source/Core/VideoCommon/CPMemory.h`), and gecko in
//! `VcdLo`/`VcdHi`/`VatA`/`VatB`/`VatC`
//! (`gecko/crates/gecko/src/flipper/gx/regs.rs:519`, `:490`, `:294`, `:385`,
//! `:441`).
//!
//! Wire order is the `GX_VA_*` numbering (`tww/include/dolphin/gx/GXEnum.h:198`):
//! the nine matrix-index bytes, then position, normal, both colours and the
//! eight texture coordinates. Gecko's `decode_vertex` consumes the same order
//! (`gecko/crates/gecko/src/flipper/gx/vertex.rs:387`), as does Dolphin's
//! `VertexLoader::CompileVertexTranslator`
//! (`dolphin/Source/Core/VideoCommon/VertexLoader.cpp`).

/// One attribute of a vertex: where it sits and how wide it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Element {
    /// The attribute, `GX_VA_*` numbering.
    pub attr: u8,
    /// The `VCD` entry: `GX_DIRECT`, `GX_INDEX8` or `GX_INDEX16`. A matrix
    /// index carries its one-bit presence flag as `GX_DIRECT`.
    pub kind: u8,
    /// Byte offset from the start of the vertex.
    pub offset: u16,
    /// Byte width of this attribute in the vertex stream: the index for an
    /// `INDEX8`/`INDEX16` element, the value's own width for a direct one.
    pub size: u16,
    /// The `VAT` type field of the value: a `GXCompType` code for a position,
    /// normal or texture coordinate, a `GXColorType` code for a colour. An
    /// indexed element's value takes the same type in its array.
    pub ctype: u32,
    /// Fraction bits of the value, which scales as `raw / 2^frac`: the `VAT`
    /// shift of a position or texture coordinate, the type-derived count of a
    /// normal ([`normal_frac`]). An indexed element's value carries the same
    /// fraction in its array. 0 for a colour.
    pub frac: u8,
    /// Bytes the value takes in the array it is indexed through: the size the
    /// same attribute takes when `DIRECT`. 0 when `kind` is `GX_DIRECT`.
    pub array_size: u16,
}

impl Element {
    /// Whether the value lives in an array the element's index names.
    pub fn is_indexed(&self) -> bool {
        self.kind == GX_INDEX8 || self.kind == GX_INDEX16
    }

    /// Bytes one value takes: the element's own width for a direct attribute,
    /// the array row's for an indexed one.
    pub fn value_size(&self) -> u16 {
        if self.is_indexed() {
            self.array_size
        } else {
            self.size
        }
    }

    /// Components one value carries, as the `VAT` fields count them. For a
    /// position, normal or texture coordinate only: a colour's `ctype` is a
    /// `GXColorType` and its stored size follows [`color_size`] instead.
    pub fn components(&self) -> u16 {
        let size = component_size(self.ctype);
        if size == 0 {
            0
        } else {
            self.value_size() / size
        }
    }

    /// The `GXSetArray` slot this element reads through when it is indexed:
    /// `attr - GX_VA_POS` (`GDGeometry.c:124`, `cpAttr = attr - GX_VA_POS`),
    /// so 0 for a position, 1 for a normal and a `GX_VA_NBT` pair alike
    /// (`GXAttr.c:474-476`, which folds `GX_VA_NBT` onto `GX_VA_NRM`), 2 and 3
    /// for the two colours and 4..11 for texture coordinates 0..7. `None` for
    /// a direct element and for a matrix index, which carries its value itself.
    pub fn array_slot(&self) -> Option<u8> {
        (self.is_indexed() && self.attr >= GX_VA_POS).then(|| self.attr - GX_VA_POS)
    }

    /// Indices one element reads: one for an indexed attribute, three for a
    /// `GX_NRM_NBT` set with `nrmidx3` (`GXAttr.c:257-260`). Zero for a direct
    /// element, which carries its value itself.
    pub fn index_count(&self) -> u16 {
        if !self.is_indexed() {
            return 0;
        }
        let width = if self.kind == GX_INDEX8 { 1 } else { 2 };
        self.size / width
    }
}

/// A complete vertex layout, in wire order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VertexFormat {
    pub elements: Vec<Element>,
    /// Bytes per vertex. Zero when nothing is present.
    pub stride: u16,
}

impl VertexFormat {
    /// Read the layout out of a CP register file.
    ///
    /// `cp_mem` is indexed by register id, so `VCD_lo` lives at `0x50`,
    /// `VCD_Hi` at `0x60` and the eight `VAT` groups at `0x70`/`0x80`/`0x90`
    /// (`GXEnum.h:1033`, the `GXCPVATGrp0..2` enumerations, and `__GXSetVAT`
    /// writes `i | 0x70`, `i | 0x80` and `i | 0x90` for group `i`,
    /// `GXAttr.c:345-352`). Only group 0 is read: a `.dff` replay takes one
    /// format for a frame, and the vertex-format index a draw opcode carries in
    /// its low three bits is not passed this far.
    pub fn from_cp(cp_mem: &[u32]) -> Self {
        let vcd_lo = cp_mem.get(0x50).copied().unwrap_or(0);
        let vcd_hi = cp_mem.get(0x60).copied().unwrap_or(0);
        let vat_a = cp_mem.get(0x70).copied().unwrap_or(0);
        let vat_b = cp_mem.get(0x80).copied().unwrap_or(0);
        let vat_c = cp_mem.get(0x90).copied().unwrap_or(0);

        let mut elements = Vec::new();
        let mut offset = 0_u16;
        for attr in ATTRS {
            let kind = attr_kind(attr, vcd_lo, vcd_hi);
            if kind == GX_NONE {
                continue;
            }
            let size = element_size(attr, kind, vat_a, vat_b, vat_c);
            let indexed = kind == GX_INDEX8 || kind == GX_INDEX16;
            elements.push(Element {
                attr,
                kind,
                offset,
                size,
                ctype: attr_type(attr, vat_a, vat_b, vat_c),
                frac: attr_frac(attr, vat_a, vat_b, vat_c),
                array_size: if indexed {
                    element_size(attr, GX_DIRECT, vat_a, vat_b, vat_c)
                } else {
                    0
                },
            });
            offset += size;
        }
        Self {
            elements,
            stride: offset,
        }
    }

    /// One attribute of this layout, in wire order.
    pub fn element(&self, attr: u8) -> Option<&Element> {
        self.elements.iter().find(|e| e.attr == attr)
    }

    /// Read one attribute of one vertex out of a payload: the value's own bytes
    /// for a direct element, the index for an `INDEX8`/`INDEX16` one.
    pub fn read<'a>(&self, attr: u8, payload: &'a [u8], vertex: usize) -> Option<&'a [u8]> {
        let element = self.element(attr)?;
        let start = usize::from(element.offset) + vertex * usize::from(self.stride);
        payload.get(start..start + usize::from(element.size))
    }
}

/// Attributes in wire order, `GX_VA_*` numbering (`GXEnum.h:198`): the nine
/// matrix-index bytes, then position, normal, both colours and texture
/// coordinates 0..7.
const ATTRS: [u8; 21] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x10, 0x11, 0x12, 0x13, 0x14,
];

/// The matrix-index attributes carry one presence bit in `VCD_lo`; they run
/// from `GX_VA_PNMTXIDX` (0x00) up to just below `GX_VA_POS`
/// (`GXEnum.h:198`, `GXGetVtxDesc`, `GXAttr.c:198-206`).
const GX_VA_POS: u8 = 0x09;
const GX_VA_NRM: u8 = 0x0A;
const GX_VA_CLR0: u8 = 0x0B;
const GX_VA_CLR1: u8 = 0x0C;
const GX_VA_TEX0: u8 = 0x0D;

/// `VCD` entry kinds.
const GX_NONE: u8 = 0;
const GX_DIRECT: u8 = 1;
const GX_INDEX8: u8 = 2;
const GX_INDEX16: u8 = 3;

/// Component types, `GXCompType` (`GXEnum.h:247`).
const GX_U8: u32 = 0;
const GX_S8: u32 = 1;
const GX_U16: u32 = 2;
const GX_S16: u32 = 3;
const GX_F32: u32 = 4;
/// Codes 5..7 have no `GXCompType` name. Dolphin reads them as floats
/// (`ComponentFormat::InvalidFloat5..7`, "assumed to behave the same as
/// float", `dolphin/Source/Core/VideoCommon/CPMemory.h`) and sizes each as
/// four bytes (`GetElementSize`, same file).
const GX_FLOAT5: u32 = 5;
const GX_FLOAT6: u32 = 6;
const GX_FLOAT7: u32 = 7;

/// Color formats, `GXColorType` (`GXEnum.h:254`).
const GX_RGB565: u32 = 0;
const GX_RGB8: u32 = 1;
const GX_RGBX8: u32 = 2;
const GX_RGBA4: u32 = 3;
const GX_RGBA6: u32 = 4;
const GX_RGBA8: u32 = 5;

/// The `VCD` entry of one attribute.
///
/// `VCD_lo` carries the nine matrix-index bits and the position, normal and
/// colour pairs; `VCD_Hi` the eight texture-coordinate pairs
/// (`GXGetVtxDesc`, `GXAttr.c:198-219`; Dolphin `TVtxDesc`,
/// `dolphin/Source/Core/VideoCommon/CPMemory.h`).
fn attr_kind(attr: u8, vcd_lo: u32, vcd_hi: u32) -> u8 {
    match attr {
        // `GX_VA_PNMTXIDX`..`GX_VA_TEX7MTXIDX`: one presence bit each.
        0x00..=0x08 => field(vcd_lo, u32::from(attr), u32::from(attr)) as u8,
        GX_VA_POS => field(vcd_lo, 9, 10) as u8,
        GX_VA_NRM => field(vcd_lo, 11, 12) as u8,
        GX_VA_CLR0 => field(vcd_lo, 13, 14) as u8,
        GX_VA_CLR1 => field(vcd_lo, 15, 16) as u8,
        // `GX_VA_TEX0`..`GX_VA_TEX7`.
        0x0D..=0x14 => {
            let low = 2 * u32::from(attr - GX_VA_TEX0);
            field(vcd_hi, low, low + 1) as u8
        }
        _ => GX_NONE,
    }
}

/// Bytes one component takes. `GXCompType` values only: the `GXColorType`
/// numbering overlaps them, so a colour must not come through here.
pub(crate) fn component_size(kind: u32) -> u16 {
    match kind {
        GX_U8 | GX_S8 => 1,
        GX_U16 | GX_S16 => 2,
        GX_F32 | GX_FLOAT5 | GX_FLOAT6 | GX_FLOAT7 => 4,
        _ => 0,
    }
}

/// Bytes one whole colour attribute takes, `GXColorType` values. The stored
/// size follows the format alone, not the `GX_CLR_RGB`/`GX_CLR_RGBA` count
/// (`VertexLoader_Color::s_table_size`,
/// `dolphin/Source/Core/VideoCommon/VertexLoader_Color.h`).
fn color_size(kind: u32) -> u16 {
    match kind {
        GX_RGB565 | GX_RGBA4 => 2,
        GX_RGB8 | GX_RGBA6 => 3,
        GX_RGBX8 | GX_RGBA8 => 4,
        _ => 4,
    }
}

/// Bytes one attribute takes in a vertex.
fn element_size(attr: u8, kind: u8, vat_a: u32, vat_b: u32, vat_c: u32) -> u16 {
    if kind == GX_INDEX8 || kind == GX_INDEX16 {
        let index = if kind == GX_INDEX8 { 1 } else { 2 };
        // An indexed bi-normal with `nrmidx3` set reads one index per vector
        // (`GXAttr.c:257-260`, gecko `nrm_stream_size`,
        // `gecko/crates/gecko/src/flipper/gx/regs.rs:348`).
        if attr == GX_VA_NRM && nrm_index3(vat_a) && nrm_components(vat_a) == 9 {
            return index * 3;
        }
        return index;
    }
    match attr {
        // `GX_VA_PNMTXIDX`..`GX_VA_TEX7MTXIDX`: a matrix index byte each.
        0x00..=0x08 => 1,
        GX_VA_POS => pos_components(vat_a) * component_size(pos_fmt(vat_a)),
        GX_VA_NRM => nrm_components(vat_a) * component_size(nrm_fmt(vat_a)),
        GX_VA_CLR0 => color_size(clr0_fmt(vat_a)),
        GX_VA_CLR1 => color_size(clr1_fmt(vat_a)),
        // `GX_VA_TEX0`..`GX_VA_TEX7`.
        0x0D..=0x14 => {
            let tex = usize::from(attr - GX_VA_TEX0);
            tex_components(tex, vat_a, vat_b, vat_c)
                * component_size(tex_fmt(tex, vat_a, vat_b, vat_c))
        }
        _ => 0,
    }
}

/// The raw type field `VAT` carries for one attribute. A colour carries its
/// `GXColorType` here and every other attribute a `GXCompType`; an indexed
/// element carries the type its array value takes, the same one the attribute
/// would carry direct.
fn attr_type(attr: u8, vat_a: u32, vat_b: u32, vat_c: u32) -> u32 {
    match attr {
        GX_VA_POS => pos_fmt(vat_a),
        GX_VA_NRM => nrm_fmt(vat_a),
        GX_VA_CLR0 => clr0_fmt(vat_a),
        GX_VA_CLR1 => clr1_fmt(vat_a),
        // `GX_VA_TEX0`..`GX_VA_TEX7`, the last eight attributes.
        0x0D..=0x14 => tex_fmt(usize::from(attr - GX_VA_TEX0), vat_a, vat_b, vat_c),
        _ => 0,
    }
}

/// Fraction bits of a fixed-point attribute, see [`Element::frac`]. An indexed
/// element carries the fraction its array value takes, the same one the
/// attribute would carry direct.
fn attr_frac(attr: u8, vat_a: u32, vat_b: u32, vat_c: u32) -> u8 {
    match attr {
        GX_VA_POS => pos_frac(vat_a),
        GX_VA_NRM => normal_frac(nrm_fmt(vat_a)),
        // `GX_VA_TEX0`..`GX_VA_TEX7`.
        0x0D..=0x14 => tex_frac(usize::from(attr - GX_VA_TEX0), vat_a, vat_b, vat_c),
        _ => 0,
    }
}

/// Fraction bits a fixed-point normal component carries, from its
/// `GXCompType`.
///
/// The divisor is `2^(bits - signed - 1)`: a signed normal keeps one sign and
/// one integer bit, an unsigned one the integer bit alone. Dolphin's
/// `FracAdjust` is that rule and spells the four fractions out in its comment
/// (`dolphin/Source/Core/VideoCommon/VertexLoader_Normal.cpp`); the SDK's
/// `GetFracForNrm` (`tww/src/dolphin/gx/GXAttr.c:358`) names the same 6 and 14
/// for `GX_S8` and `GX_S16` but falls through to 0 for the unsigned types, so
/// the unsigned fractions follow Dolphin. `f32` carries none.
fn normal_frac(fmt: u32) -> u8 {
    match fmt {
        GX_U8 => 7,
        GX_S8 => 6,
        GX_U16 => 15,
        GX_S16 => 14,
        _ => 0,
    }
}

/// Position component count: `GX_POS_XY` is 2, `GX_POS_XYZ` 3
/// (`GXEnum.h:229`, `UVAT_group0.PosElements`).
fn pos_components(vat_a: u32) -> u16 {
    if field(vat_a, 0, 0) == 0 { 2 } else { 3 }
}

fn pos_fmt(vat_a: u32) -> u32 {
    field(vat_a, 1, 3)
}

fn pos_frac(vat_a: u32) -> u8 {
    field(vat_a, 4, 8) as u8
}

/// Normal component count: `GX_NRM_XYZ` is one normal, `GX_NRM_NBT` a normal,
/// binormal and tangent, nine components (`GXEnum.h:233`,
/// `UVAT_group0.NormalElements`).
fn nrm_components(vat_a: u32) -> u16 {
    if field(vat_a, 9, 9) == 0 { 3 } else { 9 }
}

fn nrm_fmt(vat_a: u32) -> u32 {
    field(vat_a, 10, 12)
}

/// `nrmidx3`: the three vectors of a bi-normal each carry their own index
/// (`GXEnum.h:1088`, `UVAT_group0.NormalIndex3`).
fn nrm_index3(vat_a: u32) -> bool {
    field(vat_a, 31, 31) != 0
}

fn clr0_fmt(vat_a: u32) -> u32 {
    field(vat_a, 14, 16)
}

fn clr1_fmt(vat_a: u32) -> u32 {
    field(vat_a, 18, 20)
}

/// Texture-coordinate component count: `GX_TEX_S` is 1, `GX_TEX_ST` 2
/// (`GXEnum.h:241`).
fn tex_components(tex: usize, vat_a: u32, vat_b: u32, vat_c: u32) -> u16 {
    let bit = match tex {
        0 => field(vat_a, 21, 21),
        1 => field(vat_b, 0, 0),
        2 => field(vat_b, 9, 9),
        3 => field(vat_b, 18, 18),
        4 => field(vat_b, 27, 27),
        5 => field(vat_c, 5, 5),
        6 => field(vat_c, 14, 14),
        _ => field(vat_c, 23, 23),
    };
    if bit == 0 { 1 } else { 2 }
}

/// Texture-coordinate `GXCompType`, from `UVAT_group0..2` (`CPMemory.h`;
/// `GXAttr.c:410-449`).
fn tex_fmt(tex: usize, vat_a: u32, vat_b: u32, vat_c: u32) -> u32 {
    match tex {
        0 => field(vat_a, 22, 24),
        1 => field(vat_b, 1, 3),
        2 => field(vat_b, 10, 12),
        3 => field(vat_b, 19, 21),
        4 => field(vat_b, 28, 30),
        5 => field(vat_c, 6, 8),
        6 => field(vat_c, 15, 17),
        _ => field(vat_c, 24, 26),
    }
}

/// Texture-coordinate fraction bits, the `VAT` shift a fixed-point set scales
/// by (`CPMemory.h` `Tex0Frac`..`Tex7Frac`; `GXAttr.c:413-449`).
fn tex_frac(tex: usize, vat_a: u32, vat_b: u32, vat_c: u32) -> u8 {
    let frac = match tex {
        0 => field(vat_a, 25, 29),
        1 => field(vat_b, 4, 8),
        2 => field(vat_b, 13, 17),
        3 => field(vat_b, 22, 26),
        4 => field(vat_c, 0, 4),
        5 => field(vat_c, 9, 13),
        6 => field(vat_c, 18, 22),
        _ => field(vat_c, 27, 31),
    };
    frac as u8
}

fn field(word: u32, low: u32, high: u32) -> u32 {
    (word >> low) & ((1 << (high - low + 1)) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cp(vcd_lo: u32, vcd_hi: u32, vat_a: u32) -> Vec<u32> {
        let mut cp = vec![0_u32; 0x100];
        cp[0x50] = vcd_lo;
        cp[0x60] = vcd_hi;
        cp[0x70] = vat_a;
        cp
    }

    #[test]
    fn a_position_and_texcoord_set_a_stride() {
        // Position direct, three `f32` components; texcoord 0 direct, two
        // `u16` with eight fraction bits. `VCD_lo` position is bits 9..10 and
        // `VCD_Hi` texcoord 0 bits 0..1; `VAT_A` position is bits 0..8 and
        // texcoord 0 bits 21..29, see the module doc.
        let cp = cp(
            u32::from(GX_DIRECT) << 9,
            u32::from(GX_DIRECT),
            1 | (GX_F32 << 1) | (1 << 21) | (GX_U16 << 22) | (8 << 25),
        );

        let format = VertexFormat::from_cp(&cp);
        assert_eq!(
            format.stride,
            3 * 4 + 2 * 2,
            "three f32 positions and two u16 uvs"
        );
        assert_eq!(format.elements.len(), 2);
        assert_eq!(
            format.elements[0],
            Element {
                attr: GX_VA_POS,
                kind: GX_DIRECT,
                offset: 0,
                size: 12,
                ctype: GX_F32,
                frac: 0,
                array_size: 0,
            }
        );
        assert_eq!(
            format.elements[1],
            Element {
                attr: GX_VA_TEX0,
                kind: GX_DIRECT,
                offset: 12,
                size: 4,
                ctype: GX_U16,
                frac: 8,
                array_size: 0,
            }
        );
    }

    #[test]
    fn indexed_attributes_are_one_index_wide() {
        // Position indexed 8, a single normal indexed 16 and texcoord 0
        // indexed 16.
        let vcd = cp(
            (u32::from(GX_INDEX8) << 9) | (u32::from(GX_INDEX16) << 11),
            u32::from(GX_INDEX16),
            (GX_S16 << 10) | (GX_F32 << 1),
        );
        let format = VertexFormat::from_cp(&vcd);
        assert_eq!(
            format.stride,
            1 + 2 + 2,
            "one index byte and two index shorts"
        );
        assert_eq!(format.element(GX_VA_NRM).map(|e| e.size), Some(2));

        // A bi-normal with `nrmidx3` reads three indices per vertex
        // (`GXAttr.c:257-260`).
        let vcd = cp(u32::from(GX_INDEX8) << 11, 0, (1 << 9) | (1 << 31));
        let format = VertexFormat::from_cp(&vcd);
        assert_eq!(format.stride, 3);
        assert_eq!(format.element(GX_VA_NRM).map(Element::index_count), Some(3));
    }

    /// An indexed element still names the value's shape, which is the same one
    /// the attribute takes direct: `VAT` sizes an array row and the stream
    /// index alike (`GXSetArray` declares only where the rows are,
    /// `GXAttr.c:469`), and the array slot is `attr - GX_VA_POS`
    /// (`GDGeometry.c:124`).
    #[test]
    fn an_indexed_element_names_its_values_shape() {
        // Position indexed 16, two `s16` components with 12 fraction bits;
        // normal indexed 8, three `f32`s; texcoord 4 indexed 8, two `u8`s with
        // four fraction bits. `VAT_A` position bits 0..8 and normal bits 9..12
        // (`CPMemory.h` `UVAT_group0`); texcoord 4 is `UVAT_group1`
        // `Tex4CoordElements` bit 27 and `Tex4CoordFormat` bits 28..30, with
        // its fraction in `UVAT_group2` bits 0..4.
        let mut mem = cp(
            (u32::from(GX_INDEX16) << 9) | (u32::from(GX_INDEX8) << 11),
            u32::from(GX_INDEX8) << 8,
            0 | (GX_S16 << 1) | (12 << 4) | (GX_F32 << 10),
        );
        mem[0x80] = (1 << 27) | (GX_U8 << 28);
        mem[0x90] = 4;

        let format = VertexFormat::from_cp(&mem);
        let position = format.element(GX_VA_POS).expect("position is present");
        assert_eq!(position.size, 2, "an `INDEX16` element is the index itself");
        assert_eq!(
            position.value_size(),
            2 * 2,
            "two `s16` components in the array"
        );
        assert_eq!(position.components(), 2);
        assert_eq!((position.ctype, position.frac), (GX_S16, 12));
        assert_eq!(position.array_slot(), Some(0));

        let normal = format.element(GX_VA_NRM).expect("normal is present");
        assert_eq!(normal.components(), 3);
        assert_eq!((normal.ctype, normal.frac), (GX_F32, 0));
        assert_eq!(normal.array_slot(), Some(1));

        let texcoord = format.element(0x11).expect("GX_VA_TEX4 is present");
        assert_eq!(texcoord.size, 1);
        assert_eq!(texcoord.value_size(), 2, "two `u8` components in the array");
        assert_eq!(
            (texcoord.ctype, texcoord.frac, texcoord.array_slot()),
            (GX_U8, 4, Some(8))
        );

        // A direct element has no array row and no slot.
        let direct = cp(u32::from(GX_DIRECT) << 9, 0, 1 | (GX_F32 << 1));
        let format = VertexFormat::from_cp(&direct);
        let position = format.element(GX_VA_POS).expect("position is present");
        assert_eq!((position.array_size, position.value_size()), (0, 12));
        assert_eq!(position.array_slot(), None);
        assert!(!position.is_indexed());
    }

    #[test]
    fn a_matrix_index_byte_leads_the_vertex() {
        // Position/normal matrix index and texcoord 3 matrix index, then a
        // direct position and texcoord.
        let cp = cp(
            (1 << 0) | (1 << 4) | (u32::from(GX_DIRECT) << 9),
            u32::from(GX_DIRECT),
            1 | (GX_F32 << 1) | (1 << 21) | (GX_S16 << 22),
        );
        let format = VertexFormat::from_cp(&cp);
        assert_eq!(format.stride, 1 + 1 + 12 + 4);
        assert_eq!(format.elements[0].attr, 0x00, "GX_VA_PNMTXIDX");
        assert_eq!((format.elements[0].offset, format.elements[0].size), (0, 1));
        assert_eq!(format.elements[1].attr, 0x04);
        assert_eq!((format.elements[1].offset, format.elements[1].size), (1, 1));
        assert_eq!(
            (format.elements[2].attr, format.elements[2].offset),
            (GX_VA_POS, 2)
        );
        assert_eq!(
            (format.elements[3].attr, format.elements[3].offset),
            (GX_VA_TEX0, 14)
        );
    }

    #[test]
    fn a_normals_fraction_follows_its_type() {
        for (ctype, frac) in [
            (GX_U8, 7),
            (GX_S8, 6),
            (GX_U16, 15),
            (GX_S16, 14),
            (GX_F32, 0),
        ] {
            let cp = cp(
                u32::from(GX_DIRECT) << 11,
                0,
                (ctype << 10) | (1 << 0) | (GX_F32 << 1),
            );
            let format = VertexFormat::from_cp(&cp);
            let normal = format.element(GX_VA_NRM).expect("normal is present");
            assert_eq!(normal.frac, frac, "type {ctype}");
            assert_eq!(normal.size, 3 * component_size(ctype), "type {ctype}");
        }
    }

    #[test]
    fn an_absent_attribute_contributes_nothing() {
        let format = VertexFormat::from_cp(&cp(0, 0, 0));
        assert!(format.elements.is_empty());
        assert_eq!(format.stride, 0);

        let format = VertexFormat::from_cp(&[]);
        assert_eq!(format.stride, 0);
    }

    #[test]
    fn a_payload_reads_the_attribute_it_was_asked_for() {
        let cp = cp(
            u32::from(GX_DIRECT) << 9,
            u32::from(GX_DIRECT),
            1 | (GX_F32 << 1) | (1 << 21) | (GX_U16 << 22),
        );
        let format = VertexFormat::from_cp(&cp);

        let mut payload = Vec::new();
        payload.extend_from_slice(&1.0_f32.to_be_bytes()); // vertex 0 pos x
        payload.extend_from_slice(&2.0_f32.to_be_bytes());
        payload.extend_from_slice(&3.0_f32.to_be_bytes());
        payload.extend_from_slice(&0x0102_u16.to_be_bytes()); // vertex 0 uv u
        payload.extend_from_slice(&0x0304_u16.to_be_bytes()); // vertex 0 uv v
        payload.extend_from_slice(&4.0_f32.to_be_bytes()); // vertex 1 pos x
        payload.extend_from_slice(&5.0_f32.to_be_bytes());
        payload.extend_from_slice(&6.0_f32.to_be_bytes());
        payload.extend_from_slice(&0x0506_u16.to_be_bytes()); // vertex 1 uv u
        payload.extend_from_slice(&0x0708_u16.to_be_bytes()); // vertex 1 uv v

        assert_eq!(format.read(GX_VA_POS, &payload, 0), Some(&payload[0..12]));
        assert_eq!(format.read(GX_VA_TEX0, &payload, 0), Some(&payload[12..16]));
        assert_eq!(format.read(GX_VA_POS, &payload, 1), Some(&payload[16..28]));
        assert_eq!(format.read(GX_VA_TEX0, &payload, 1), Some(&payload[28..32]));
        assert_eq!(
            format.read(GX_VA_NRM, &payload, 0),
            None,
            "no normal is present"
        );
    }
}
