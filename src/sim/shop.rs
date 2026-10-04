use bevy_ecs::prelude::*;

pub const SHOP_ACTOR: &str = "Bs1";
pub const SLOT_COUNT: usize = 3;

pub const COND_MUST_OWN_ITEM: u8 = 0x01;
pub const COND_MUST_NOT_OWN_ITEM: u8 = 0x02;
pub const COND_MAXIMUM: u8 = 0x04;
pub const COND_ANY_BOTTLE: u8 = 0x08;
pub const COND_EMPTY_BOTTLE: u8 = 0x10;
pub const COND_RUPEES: u8 = 0x20;

pub const NO_ITEM: u8 = 0xFF;
pub const ITEM_HEART_PIECE: u8 = 0x07;
pub const ITEM_BOMB_30: u8 = 0x0E;
pub const ITEM_ARROW_10: u8 = 0x10;
pub const ITEM_ARROW_30: u8 = 0x12;
pub const ITEM_BOW: u8 = 0x27;
pub const ITEM_BOMB_BAG: u8 = 0x31;
pub const ITEM_MAP: u8 = 0x4C;
pub const ITEM_EMPTY_BOTTLE: u8 = 0x50;
pub const ITEM_RED_POTION: u8 = 0x51;
pub const ITEM_BAIT_BAG: u8 = 0x2C;
pub const ITEM_BIRD_BAIT_5: u8 = 0x82;
pub const ITEM_HYOI_PEAR: u8 = 0x83;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stock {
    pub item: u8,
    pub must_not_own: u8,
    pub must_own: u8,
    pub conditions: u8,
    pub count: i32,
    pub price: u32,
    pub show_msg: u16,
    pub buy_msg: u16,
}

const fn stock(
    item: u8,
    must_not_own: u8,
    must_own: u8,
    conditions: u8,
    count: i32,
    price: u32,
    show_msg: u16,
    buy_msg: u16,
) -> Stock {
    Stock {
        item,
        must_not_own,
        must_own,
        conditions,
        count,
        price,
        show_msg,
        buy_msg,
    }
}

const FEED_BAG: Stock = stock(
    ITEM_BAIT_BAG,
    ITEM_BAIT_BAG,
    NO_ITEM,
    COND_RUPEES | COND_MUST_NOT_OWN_ITEM,
    -1,
    50,
    0x0F42,
    0x0F45,
);
const BAIT: Stock = stock(
    ITEM_BIRD_BAIT_5,
    ITEM_BIRD_BAIT_5,
    ITEM_BAIT_BAG,
    COND_RUPEES | COND_MAXIMUM | COND_MUST_OWN_ITEM,
    10,
    10,
    0x0F43,
    0x0F46,
);
const PEAR: Stock = stock(
    ITEM_HYOI_PEAR,
    ITEM_HYOI_PEAR,
    ITEM_BAIT_BAG,
    COND_RUPEES | COND_MAXIMUM | COND_MUST_OWN_ITEM,
    10,
    10,
    0x0F44,
    0x0F47,
);
const BOMB: Stock = stock(
    ITEM_BOMB_30,
    ITEM_BOMB_30,
    ITEM_BOMB_BAG,
    COND_RUPEES | COND_MAXIMUM,
    30,
    30,
    0x0F67,
    0x0F68,
);
const ARROW10: Stock = stock(
    ITEM_ARROW_10,
    ITEM_ARROW_10,
    ITEM_BOW,
    COND_RUPEES | COND_MAXIMUM | COND_MUST_OWN_ITEM,
    10,
    10,
    0x0F69,
    0x0F6A,
);
const ARROW30: Stock = stock(
    ITEM_ARROW_30,
    ITEM_ARROW_30,
    ITEM_BOW,
    COND_RUPEES | COND_MAXIMUM | COND_MUST_OWN_ITEM,
    30,
    30,
    0x0F6B,
    0x0F6C,
);
const RED_POTION: Stock = stock(
    ITEM_RED_POTION,
    ITEM_RED_POTION,
    ITEM_EMPTY_BOTTLE,
    COND_RUPEES | COND_EMPTY_BOTTLE | COND_ANY_BOTTLE,
    -1,
    60,
    0x0F6D,
    0x0F6E,
);
const BAZAAR_BOTTLE: Stock = stock(
    ITEM_EMPTY_BOTTLE,
    NO_ITEM,
    NO_ITEM,
    0,
    -1,
    0,
    0x2F6B,
    0x2F6C,
);
const BAZAAR_PIECE: Stock = stock(ITEM_HEART_PIECE, NO_ITEM, NO_ITEM, 0, -1, 0, 0x2F6D, 0x2F6E);
const BAZAAR_MAP: Stock = stock(ITEM_MAP, ITEM_MAP, NO_ITEM, 0, -1, 0, 0x2F6F, 0x2F70);

const GENERAL_THREE: [Stock; SLOT_COUNT] = [BAIT, PEAR, PEAR];
const GENERAL_FOUR: [Stock; SLOT_COUNT] = [ARROW10, ARROW30, BAIT];
const GENERAL_FIVE: [Stock; SLOT_COUNT] = [ARROW30, BOMB, RED_POTION];
const BAZAAR_STOCK: [Stock; SLOT_COUNT] = [BAZAAR_BOTTLE, BAZAAR_PIECE, BAZAAR_MAP];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shop {
    General(u8),
    Bazaar,
}

impl Shop {
    pub fn spawn(parameters: u32) -> Self {
        if parameters & 0xF == 0 {
            match (parameters >> 4) & 0xF {
                4 => Shop::General(4),
                5 => Shop::General(5),
                _ => Shop::General(3),
            }
        } else {
            Shop::Bazaar
        }
    }

    pub fn stock(&self) -> [Stock; SLOT_COUNT] {
        match self {
            Shop::General(4) => GENERAL_FOUR,
            Shop::General(5) => GENERAL_FIVE,
            Shop::General(3) => GENERAL_THREE,
            Shop::General(_) => [FEED_BAG, BAIT, PEAR],
            Shop::Bazaar => BAZAAR_STOCK,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Owned {
    pub items: [u8; 64],
    pub bottles: u8,
    pub rupees: u32,
}

impl Owned {
    pub fn has(&self, item: u8) -> bool {
        self.items.contains(&item)
    }

    pub fn bottles_empty(&self) -> bool {
        self.bottles > 0
    }
}

impl Stock {
    pub fn offered(&self, owned: &Owned) -> bool {
        if self.conditions & COND_MUST_OWN_ITEM != 0
            && self.must_own != NO_ITEM
            && !owned.has(self.must_own)
        {
            return false;
        }
        if self.conditions & COND_MUST_NOT_OWN_ITEM != 0
            && self.must_not_own != NO_ITEM
            && owned.has(self.must_not_own)
        {
            return false;
        }
        if self.conditions & COND_EMPTY_BOTTLE != 0 && !owned.bottles_empty() {
            return false;
        }
        owned.rupees >= self.price
    }
}