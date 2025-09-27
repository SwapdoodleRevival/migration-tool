use std::ops::ControlFlow;

use ctru::prelude::{Apt, Gfx, Hid, KeyPad};

pub mod intro;
pub mod mapping;
pub mod reading;
pub mod rewrite;

pub use intro::*;
pub use mapping::*;
pub use reading::*;
pub use rewrite::*;

pub type MigrationFlow<T = ()> = ControlFlow<(), T>;

fn process(apt: &Apt, gfx: &Gfx, hid: &mut Hid) -> MigrationFlow {
    if !apt.main_loop() {
        return MigrationFlow::Break(());
    }
    gfx.wait_for_vblank();
    hid.scan_input();
    if hid.keys_down().contains(KeyPad::START) {
        return MigrationFlow::Break(());
    }
    MigrationFlow::Continue(())
}
