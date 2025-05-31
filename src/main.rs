use std::{
    collections::HashMap,
    io::{self, Write},
};

use ctru::{console::Axis, prelude::*};
use friend_list::MiiMap;
use libdoodle::mii_data::MiiData;

mod extdata;
mod friend_list;

fn main() {
    ctru::applets::error::set_panic_hook(true);
    app_loop();
}

//                       -- PID of note sender
//                       v      .- PID of friend
type Remapping = HashMap<u32, u32>;

fn app_loop() {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx = Gfx::new().unwrap();
    let bottom_console = Console::new(gfx.bottom_screen.borrow_mut());
    let top_console = Console::new(gfx.top_screen.borrow_mut());

    let friends: MiiMap;
    let doodles: MiiMap;
    let mapping = Remapping::new();

    introduce();

    loop {
        if !apt.main_loop() {
            return;
        }

        gfx.wait_for_vblank();

        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            return;
        }
        if hid.keys_down().contains(KeyPad::A) {
            (friends, doodles) = friendly_read_data();
            break;
        }
    }

    let mut hover: usize = 0;
    let mut dirty = true;

    loop {
        if !apt.main_loop() {
            return;
        }

        gfx.wait_for_vblank();

        if dirty {
            dirty = false;
            print_top_screen(&top_console, &doodles, &friends, &mapping, hover);
        }

        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            return;
        }

        if hid.keys_down().contains(KeyPad::DPAD_UP) {
            if hover != 0 {
                dirty = true;
                hover -= 1;
            }
        }

        if hid.keys_down().contains(KeyPad::DPAD_DOWN) {
            if hover != (doodles.len() - 1) {
                dirty = true;
                hover += 1;
            }
        }

        if hid.keys_down().contains(KeyPad::A) {
            print!("\x1b[27;0H\x1b[1;37;41m");
            print_center("");
            print_center("Look at the bottom screen.");
            print_center("");
            bottom_console.select();
            pick_friend(&bottom_console, &friends);
            top_console.select();
            println!("\x1b[0m");
        }
    }
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

fn pick_friend(con: &Console, friends: &MiiMap) {
    con.clear();
    println!("Select a friend:");
}

fn introduce() {
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
}

fn friendly_read_data() -> (MiiMap, MiiMap) {
    print!("Reading your friend list... ");
    _ = io::stdout().flush();
    let friends = friend_list::load_friend_list();
    println!("done!");

    print!("Reading your Swapdoodle extdata... ");
    _ = io::stdout().flush();
    let mut doodles = HashMap::<u32, MiiData>::new();
    for (file, filename, letter) in extdata::read() {
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
