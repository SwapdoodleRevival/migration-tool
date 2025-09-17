use ctru::prelude::{Apt, Gfx, Hid, KeyPad};

pub mod intro;
pub mod mapping;
pub mod reading;
pub mod rewrite;

pub use intro::*;
pub use mapping::*;
pub use reading::*;
pub use rewrite::*;

fn process(apt: &Apt, gfx: &Gfx, hid: &mut Hid) -> Result<(), ()> {
    if !apt.main_loop() {
        return Err(());
    }
    gfx.wait_for_vblank();
    hid.scan_input();
    if hid.keys_down().contains(KeyPad::START) {
        return Err(());
    }
    Ok(())
}
