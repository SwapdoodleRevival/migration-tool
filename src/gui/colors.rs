pub const TEXT_PRIMARY: u32 = rgba(255, 255, 255, 255);
pub const TEXT_DANGER: u32 = rgba(255, 0, 0, 255);
pub const BLUE: u32 = rgba(0, 40, 199, 255);
pub const SIDE_FRIENDS: u32 = rgba(245, 142, 11, 80);
pub const SIDE_SWAPDOODLE: u32 = rgba(55, 83, 9, 100);
pub const SHADOW: u32 = rgba(0, 0, 0, 120);
pub const HIGHLIGHT: u32 = rgba(255, 255, 255, 90);
pub const BACKGROUND: u32 = rgba(20, 20, 20, 255);

pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    r as u32 | (g as u32) << 8 | (b as u32) << 16 | (a as u32) << 24
}
