use std::{
    collections::HashMap,
    io::{self, Write},
};

use ctru::prelude::*;
use friend_list::MiiMap;
use libdoodle::mii_data::MiiData;

mod extdata;
mod friend_list;
mod phases;

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

    if (true) {
        data.friends.insert(1, _test_dummyMii());
        data.friends.insert(2, _test_dummyMii());
        data.friends.insert(3, _test_dummyMii());
        data.friends.insert(4, _test_dummyMii());
        data.friends.insert(5, _test_dummyMii());
        data.friends.insert(6, _test_dummyMii());
        data.friends.insert(7, _test_dummyMii());
        data.friends.insert(8, _test_dummyMii());
        data.friends.insert(9, _test_dummyMii());
        data.friends.insert(10, _test_dummyMii());
        data.friends.insert(11, _test_dummyMii());
        data.friends.insert(12, _test_dummyMii());
        data.friends.insert(13, _test_dummyMii());
        data.friends.insert(14, _test_dummyMii());
        data.friends.insert(15, _test_dummyMii());
        data.friends.insert(16, _test_dummyMii());
        data.friends.insert(17, _test_dummyMii());
        data.friends.insert(18, _test_dummyMii());
        data.friends.insert(19, _test_dummyMii());
        data.friends.insert(20, _test_dummyMii());
        data.friends.insert(91, _test_dummyMii());
        data.friends.insert(92, _test_dummyMii());
        data.friends.insert(93, _test_dummyMii());
        data.friends.insert(94, _test_dummyMii());
        data.friends.insert(95, _test_dummyMii());
        data.friends.insert(96, _test_dummyMii());
        data.friends.insert(97, _test_dummyMii());
        data.friends.insert(98, _test_dummyMii());
        data.friends.insert(99, _test_dummyMii());
        data.friends.insert(910, _test_dummyMii());
        data.friends.insert(911, _test_dummyMii());
        data.friends.insert(912, _test_dummyMii());
        data.friends.insert(913, _test_dummyMii());
        data.friends.insert(914, _test_dummyMii());
        data.friends.insert(915, _test_dummyMii());
        data.friends.insert(916, _test_dummyMii());
        data.friends.insert(917, _test_dummyMii());
        data.friends.insert(918, _test_dummyMii());
        data.friends.insert(919, _test_dummyMii());
        data.friends.insert(920, _test_dummyMii());
    }

    phases::mapping(&mut services, &mut data)?;
    Ok(())
}

fn _test_dummyMii() -> MiiData {
    let mut dummyMii = MiiData::from_bytes([
        0x03, 0x00, 0x00, 0x30, 0xF7, 0x3E, 0x4E, 0x72, 0xC2, 0x75, 0x29, 0x6E, 0x9C, 0xB8, 0x3E,
        0xF6, 0x7C, 0xBB, 0x8A, 0x6C, 0x70, 0xB5, 0x00, 0x00, 0x94, 0x59, 0x56, 0x00, 0x6F, 0x00,
        0x6C, 0x00, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x56, 0x00, 0x6F, 0x00, 0x6C, 0x00, 0x74,
        0x00, 0x7F, 0x2D, 0x00, 0x00, 0x1F, 0x01, 0x82, 0x68, 0x44, 0x16, 0x33, 0x34, 0x45, 0x12,
        0x81, 0x12, 0x0F, 0x66, 0x0D, 0x00, 0x00, 0x29, 0xB0, 0x49, 0x48, 0x50, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ])
    .unwrap();
    dummyMii.mii_name = "Sample".to_string();
    dummyMii
}
