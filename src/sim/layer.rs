use super::flags::StoryBits;

const DAY: u8 = 0;
pub const SEA: &str = "sea";
const FORSAKEN_FORTRESS: u8 = 1;
const WINDFALL: u8 = 11;
const OUTSET: u8 = 44;

pub fn no(stage: &str, room: u8, stories: &StoryBits) -> u8 {
    let layer = DAY;
    if stage == SEA {
        if room == OUTSET {
            if stories.test(0x0520) {
                return layer | 4;
            }
            if stories.test(0x0E20) {
                return layer | 2;
            }
            if stories.test(0x0101) {
                return 9;
            }
        }
        if room == WINDFALL {
            if stories.test(0x2D01) {
                return layer | 4;
            }
        }
        if room == FORSAKEN_FORTRESS {
            return if stories.test(0x1820) { 3 } else { 1 };
        }
        return layer;
    }
    if stage == "A_mori" && stories.test(0x0F80) {
        return layer | 2;
    }
    if stage == "Asoko" && stories.test(0x0520) {
        return layer | 2;
    }
    if stage == "M2tower" && stories.test(0x2D01) {
        return layer | 2;
    }
    if stage == "GanonK" && !stories.test(0x3B02) {
        return 8;
    }
    if stage == "GTower" && !stories.test(0x4002) {
        return 8;
    }
    layer
}
