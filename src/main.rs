use std::{
    collections::HashMap,
    io::{self, Write},
};

use ctru::prelude::*;
use extdata::create_writer;
use friend_list::MiiMap;
use libdoodle::mii_data::MiiData;

mod extdata;
mod friend_list;
mod phases;
pub mod menu;

//                       .- PID of note sender
//                       v      .- PID of friend
type Remapping = HashMap<u32, u32>;

struct Services<'a> {
    apt: &'a Apt,
    hid: &'a mut Hid,
    gfx: &'a Gfx,
    bottom_console: Console<'a>,
    top_console: Console<'a>,
}

impl<'a> Services<'a> {
    fn process(&mut self) -> Result<(), ()> {
        if !self.apt.main_loop() {
            return Err(());
        }
        self.gfx.wait_for_vblank();
        self.hid.scan_input();
        if self.hid.keys_down().contains(KeyPad::START) {
            return Err(());
        }
        Ok(())
    }
}

struct AppData {
    friends: MiiMap,
    doodles: MiiMap,
    mapping: Remapping,
}

fn main() {
    ctru::applets::error::set_panic_hook(true);
    _ = run();
}

fn run() -> Result<(), ()> {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx: Gfx = Gfx::new().unwrap();
    let bottom_console = Console::new(gfx.bottom_screen.borrow_mut());
    let top_console = Console::new(gfx.top_screen.borrow_mut());

    let mut services = Services {
        apt: &apt,
        gfx: &gfx,
        hid: &mut hid,
        bottom_console: bottom_console,
        top_console: top_console,
    };

    let mut data = AppData {
        friends: MiiMap::new(),
        doodles: MiiMap::new(),
        mapping: Remapping::new(),
    };

    phases::intro(&mut services, &mut data)?;
    phases::mapping(&mut services, &mut data)?;
    Ok(())
}
