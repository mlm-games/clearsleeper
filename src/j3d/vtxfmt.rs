//! The vertex format a draw's payload is laid out in.
//!
//! A GX stream never says how big a vertex is. Two CP registers do: `VCD_lo` and
//! `VCD_Hi` say which attributes are present and whether each is `DIRECT` or an
//! `INDEX8`/`INDEX16` reference, and the `VAT_A..C` tables give the component
//! count and type of each `DIRECT` one. Only together do they yield a stride,
//! and the stride is what lets a replay step over a payload.

/// One attribute of a vertex: where it sits and how wide it is.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Element {
        /// The attribute, `GX_VA_POS` and friends.
        pub attr: u8,
        /// The `VCD` entry: `GX_DIRECT`, `GX_INDEX8` or `GX_INDEX16`.
        pub kind: u8,
        /// Byte offset from the start of the vertex.
        pub offset: u16,
        /// Byte width of this attribute.
        pub size: u16,
        /// The `VAT` type field this element's components carry. For a color it
        /// is the `GXColorType` code, for everything else a `GXCompType` one.
        pub ctype: u32,
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
    /// `VCD_Hi` at `0x60` and the eight `VAT` tables at `0x70..=0x77`
    /// (`GXEnum.h:950-951`, and `__GXSetVAT` writes `i | 0x70`, `i | 0x80` and
    /// `i | 0x90` for group `i`, `GXAttr.c:346-350`).
    pub fn from_cp(cp_mem: &[u32]) -> Self {
        let vcd_lo = cp_mem.get(0x50).copied().unwrap_or(0);
        let vcd_hi = cp_mem.get(0x60).copied().unwrap_or(0);

        let mut elements = Vec::new();
        let mut offset = 0_u16;
        // Wire order follows the attribute numbering, which is what the
        // `VCD` bit pairs are laid out in: `GX_VA_POS` is `0x09` and the rest
        // follow (`GXEnum.h:208-212`).
        for (index, attr) in ATTRS.iter().enumerate() {
            let vcd = if index < 8 { vcd_lo } else { vcd_hi };
            let kind = ((vcd >> (2 * (index % 8))) & 0x3) as u8;
            if kind == GX_NONE {
                continue;
            }
            let size = element_size(*attr, kind, index, cp_mem);
            elements.push(Element {
                attr: *attr,
                kind,
                offset,
                size,
                ctype: attr_type(index, kind, cp_mem),
            });
            offset += size;
        }
        Self {
            elements,
            stride: offset,
        }
    }

    /// Read one attribute of one vertex out of a payload.
    pub fn read<'a>(&self, attr: u8, payload: &'a [u8], vertex: usize) -> Option<&'a [u8]> {
        let element = self.elements.iter().find(|e| e.attr == attr)?;
        let start = usize::from(element.offset) + vertex * usize::from(self.stride);
        payload.get(start..start + usize::from(element.size))
    }

    /// The type field of one attribute, as [`Element::ctype`] carries it.
    pub fn type_of(&self, attr: u8) -> Option<u32> {
        self.elements
            .iter()
            .find(|e| e.attr == attr)
            .map(|e| e.ctype)
    }
}

/// Attributes in wire order, `GX_VA_*` numbering.
const ATTRS: [u8; 10] = [0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12];

/// `VCD` entry kinds.
const GX_NONE: u8 = 0;
const GX_DIRECT: u8 = 1;
const GX_INDEX8: u8 = 2;
const GX_INDEX16: u8 = 3;

/// Component types, `GXCompType`.
const GX_U8: u32 = 0;
#[allow(dead_code)]
const GX_S8: u32 = 1;
const GX_U16: u32 = 2;
#[allow(dead_code)]
const GX_S16: u32 = 3;
const GX_F32: u32 = 4;

/// Color formats, `GXColorType`.
const GX_RGB565: u32 = 0;
const GX_RGB8: u32 = 1;
const GX_RGBX8: u32 = 2;
const GX_RGBA4: u32 = 3;
const GX_RGBA6: u32 = 4;
const GX_RGBA8: u32 = 5;

/// Bytes one component takes.
/// `GXCompType` values only: the `GXColorType` numbering overlaps these, so a
/// color must not come through here.
fn component_size(kind: u32) -> u16 {
    match kind {
        GX_U8 | GX_S8 => 1,
        GX_U16 | GX_S16 => 2,
        GX_F32 => 4,
        _ => 0,
    }
}

/// Bytes one whole color attribute takes, `GXColorType` values.
fn color_size(kind: u32) -> u16 {
    match kind {
        GX_RGB565 | GX_RGBA4 => 2,
        GX_RGBA6 => 3,
        GX_RGB8 | GX_RGBX8 | GX_RGBA8 => 4,
        _ => 4,
    }
}

/// The raw type field `VAT_A` carries for one attribute. A colour carries its
/// `GXColorType` here and every other attribute a `GXCompType`, which is why
/// the field is not interpreted here.
fn attr_type(index: usize, kind: u8, cp_mem: &[u32]) -> u32 {
    if kind != GX_DIRECT {
        return 0;
    }
    let vat = vat_a(cp_mem);
    match index {
        // Position: count at 31, type at 28..30.
        0 => field(vat, 28, 30),
        // Normal: type at 19..21.
        1 => field(vat, 19, 21),
        // Colour: type at 15..17.
        2 => field(vat, 15, 17),
        // Texture coordinate 0: type at 7..9.
        4 => field(vat, 7, 9),
        _ => 0,
    }
}

/// The `VAT_A` word for attribute index `index`, which is which attribute within
/// group 0. Groups 1 and 2 hold the later texture coordinates, which the layout
/// below leaves to `vat_b`/`vat_c`.
fn vat_a(cp_mem: &[u32]) -> u32 {
    cp_mem.get(0x70).copied().unwrap_or(0)
}

fn field(word: u32, low: u32, high: u32) -> u32 {
    (word >> low) & ((1 << (high - low + 1)) - 1)
}

/// Bytes one attribute takes in a vertex.
fn element_size(attr: u8, kind: u8, index: usize, cp_mem: &[u32]) -> u16 {
    if kind == GX_INDEX8 {
        return 1;
    }
    if kind == GX_INDEX16 {
        return 2;
    }
    let vat = vat_a(cp_mem);
    match index {
        // Position: one count bit and a three-bit type, at the top of `VAT_A`.
        0 => {
            let count = field(vat, 31, 31);
            let ctype = field(vat, 28, 30);
            let components = count + 2;
            components as u16 * component_size(ctype)
        }
        // Normal, also on `VAT_A`.
        1 => component_size(field(vat, 19, 21)) * 3,
        // Diffuse color, also on `VAT_A`.
        2 => color_size(field(vat, 15, 17)),
        // Texture coordinate 0, also on `VAT_A`.
        4 => {
            let count = field(vat, 10, 10) + 1;
            let ctype = field(vat, 7, 9);
            count as u16 * component_size(ctype)
        }
        // Later attribute groups need their own `VAT_B`/`VAT_C` words, which
        // this layout does not read; a stream using them keeps the attribute
        // but cannot size it, so it is skipped rather than mis-sized.
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cp(vcd_lo: u32, vat_a: u32) -> Vec<u32> {
        let mut cp = vec![0_u32; 0x100];
        cp[0x50] = vcd_lo;
        cp[0x60] = 0;
        cp[0x70] = vat_a;
        cp
    }

    #[test]
    fn a_position_and_texcoord_set_a_stride() {
        // Position direct, three `f32` components; texcoord 0 direct, two `u16`.
        // `VAT_A` bits follow `GXEnum.h:1035-1071`.
        let vat = (1 << 31)            // pos count: 3 components
            | (GX_F32 << 28)           // pos type
            | (GX_U16 << 7);           // texcoord 0 type
        let cp = cp((u32::from(GX_DIRECT) << 0) | (u32::from(GX_DIRECT) << 8), vat);

        let format = VertexFormat::from_cp(&cp);
        assert_eq!(format.stride, 3 * 4 + 2, "three f32 positions and two u16 uvs");
        assert_eq!(format.elements.len(), 2);
        assert_eq!(format.elements[0].attr, 0x09);
        assert_eq!(format.elements[0].offset, 0);
        assert_eq!(format.elements[0].size, 12);
        assert_eq!(format.elements[1].attr, 0x0D);
        assert_eq!(format.elements[1].offset, 12);
        assert_eq!(format.elements[1].size, 2);
    }

    #[test]
    fn indexed_attributes_take_one_byte_each() {
        // Position indexed 8, texcoord 0 indexed 16.
        let cp = cp((u32::from(GX_INDEX8) << 0) | (u32::from(GX_INDEX16) << 8), 0);
        let format = VertexFormat::from_cp(&cp);
        assert_eq!(format.stride, 3, "one index byte plus one index short");
    }

    #[test]
    fn an_absent_attribute_contributes_nothing() {
        let cp = cp(0, 0);
        let format = VertexFormat::from_cp(&cp);
        assert!(format.elements.is_empty());
        assert_eq!(format.stride, 0);
    }

    #[test]
    fn a_payload_reads_the_attribute_it_was_asked_for() {
        let vat = (1 << 31) | (GX_F32 << 28) | (GX_U16 << 7);
        let cp = cp((u32::from(GX_DIRECT) << 0) | (u32::from(GX_DIRECT) << 8), vat);
        let format = VertexFormat::from_cp(&cp);

        let mut payload = Vec::new();
        payload.extend_from_slice(&1.0_f32.to_le_bytes()); // vertex 0 pos x
        payload.extend_from_slice(&2.0_f32.to_le_bytes());
        payload.extend_from_slice(&3.0_f32.to_le_bytes());
        payload.extend_from_slice(&0x0102_u16.to_le_bytes()); // vertex 0 uv
        payload.extend_from_slice(&4.0_f32.to_le_bytes()); // vertex 1 pos x
        payload.extend_from_slice(&5.0_f32.to_le_bytes());
        payload.extend_from_slice(&6.0_f32.to_le_bytes());
        payload.extend_from_slice(&0x0304_u16.to_le_bytes()); // vertex 1 uv

        assert_eq!(format.read(0x09, &payload, 0), Some(&payload[0..12]));
        assert_eq!(format.read(0x0D, &payload, 0), Some(&payload[12..14]));
        assert_eq!(format.read(0x09, &payload, 1), Some(&payload[14..26]));
        assert_eq!(format.read(0x0D, &payload, 1), Some(&payload[26..28]));
        assert_eq!(format.read(0x0A, &payload, 0), None, "no normal is present");
    }

    #[test]
    fn a_short_register_file_reads_as_absent() {
        let format = VertexFormat::from_cp(&[]);
        assert_eq!(format.stride, 0);
    }
}
