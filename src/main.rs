use std::cell::OnceCell;

use ctru::prelude::*;

use crate::gui::Gui;

mod extdata;
mod friend_list;
mod gui;
mod phases;
mod read;

struct Services<'a> {
    apt: &'a Apt,
    hid: &'a mut Hid,
    gfx: &'a Gfx,
    gui: &'a mut Gui,
    console: Console<'a>,
}

impl<'a> Services<'a> {
    pub fn process(apt: &Apt, gfx: &Gfx, hid: &mut Hid) -> Result<(), ()> {
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
}

fn main() {
    ctru::applets::error::set_panic_hook(true);
    _ = run();
}

fn run() -> Result<(), ()> {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx: Gfx = Gfx::new().unwrap();
    let console = Console::new(gfx.bottom_screen.borrow_mut());
    let mut gui = Gui::init();

    let mut services = Services {
        apt: &apt,
        gfx: &gfx,
        hid: &mut hid,
        gui: &mut gui,
        console,
    };

    phases::intro(&mut services)?;
    let (extdata, read_data) = phases::reading(&mut services)?;
    let mapping = phases::mapping(&mut services, read_data)?;
    phases::rewrite(&mut services, extdata, mapping)?;
    Ok(())
}
