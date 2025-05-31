use std::{
    collections::HashMap,
    io::{self, Write},
};

use ctru::prelude::*;
use friend_list::MiiMap;
use libdoodle::mii_data::MiiData;

mod extdata;
mod friend_list;

fn main() {
    ctru::applets::error::set_panic_hook(true);
    app_loop();
}

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

struct AppData {
    friends: MiiMap,
    doodles: MiiMap,
    mapping: Remapping,
}

fn app_loop() {
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

    phase_intro(&mut services, &mut data).unwrap();
    phase_select(&mut services, &mut data);
}

fn print_top_screen(
    con: &Console,
    doodles: &MiiMap,
    friends: &MiiMap,
    mapping: &Remapping,
    hover: usize,
) {
    con.clear();

    let mut index: usize = 0;
    for (pid, mii) in doodles {
        let mut friend_name: String = String::from("<don't map>");

        if let Some(friend_pid) = mapping.get(pid) {
            friend_name = friends.get(friend_pid).unwrap().mii_name.clone()
        }

        println!(
            " {} {: <23}{: >23}",
            if index == hover { '>' } else { ' ' },
            mii.mii_name,
            friend_name
        );
        index += 1;
    }
}

fn pick_friend(con: &Console, _friends: &MiiMap) {
    con.clear();
    println!("Select a friend:");
}

fn process_services(s: &mut Services) -> Result<(), ()> {
    if !s.apt.main_loop() {
        return Err(());
    }
    s.gfx.wait_for_vblank();
    s.hid.scan_input();
    if s.hid.keys_down().contains(KeyPad::START) {
        return Err(());
    }
    Ok(())
}

fn phase_intro(s: &mut Services, data: &mut AppData) -> Result<(), ()> {
    println!();
    print_center("Swapdoodle migration tool");
    println!();
    println!(
        "This tool will help you migrate your Swapdoodle\nnotes from a Nintendo environment\nto a Pretendo environment."
    );
    println!();
    println!(
        "After using this tool, your notes will be moved\nfrom \"Unknown sender\"\nto your friends' profiles."
    );
    println!();
    print_center("Press (A) to begin");

    loop {
        process_services(s)?;

        if s.hid.keys_down().contains(KeyPad::A) {
            (data.friends, data.doodles) = friendly_read_data();
            return Ok(());
        }
    }
}

fn phase_select(s: &mut Services, data: &mut AppData) {
    let mut hover: usize = 0;
    let mut dirty = true;

    loop {
        if !s.apt.main_loop() {
            return;
        }
        s.gfx.wait_for_vblank();
        s.hid.scan_input();

        if dirty {
            dirty = false;
            print_top_screen(&s.top_console, &data.doodles, &data.friends, &data.mapping, hover);
        }

        if s.hid.keys_down().contains(KeyPad::START) {
            return;
        }

        if s.hid.keys_down().contains(KeyPad::DPAD_UP) {
            if hover != 0 {
                dirty = true;
                hover -= 1;
            }
        }

        if s.hid.keys_down().contains(KeyPad::DPAD_DOWN) {
            if hover != (data.doodles.len() - 1) {
                dirty = true;
                hover += 1;
            }
        }

        if s.hid.keys_down().contains(KeyPad::A) {
            print!("\x1b[27;0H\x1b[1;37;41m");
            print_center("");
            print_center("Look at the bottom screen.");
            print_center("");
            s.bottom_console.select();
            pick_friend(&s.bottom_console, &data.friends);
            s.top_console.select();
            println!("\x1b[0m");
        }
    }
}

fn friendly_read_data() -> (MiiMap, MiiMap) {
    print!("Reading your friend list... ");
    _ = io::stdout().flush();
    let friends = friend_list::load_friend_list();
    println!("done!");

    print!("Reading your Swapdoodle extdata... ");
    _ = io::stdout().flush();
    let mut doodles = HashMap::<u32, MiiData>::new();
    for (_file, _filename, letter) in extdata::read() {
        if letter.common.sender_pid != 0
            && let Some(mii) = letter.sender_mii
        {
            doodles.insert(letter.common.sender_pid, mii);
        }
    }
    println!("done!");

    (friends, doodles)
}

fn print_center(a: &str) {
    println!("{:^1$}", a, 50);
}
