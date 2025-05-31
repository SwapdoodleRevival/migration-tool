use std::{
    collections::HashMap,
    io::{self, Write},
};

use ctru::prelude::*;
use friend_list::MiiMap;
use libdoodle::mii_data::MiiData;

mod extdata;
mod friend_list;

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

    phase_intro(&mut services, &mut data)?;
    phase_select(&mut services, &mut data)?;
    Ok(())
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

fn print_bottom_screen(data: &AppData, hover: usize) {
    let mut index: usize = 0;

    for (pid, mii) in &data.friends {
        println!(
            " {} {}",
            if index == hover { '>' } else { ' ' },
            mii.mii_name,
        );
        index += 1;
    }
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
    print_center("Press (START) at any time to exit");
    println!();

    loop {
        s.process()?;

        if s.hid.keys_down().contains(KeyPad::A) {
            (data.friends, data.doodles) = friendly_read_data();
            return Ok(());
        }
    }
}

fn phase_select(s: &mut Services, data: &mut AppData) -> Result<(), ()> {
    let mut hover: usize = 0;
    let mut dirty = true;

    loop {
        s.process()?;

        if dirty {
            dirty = false;
            print_top_screen(
                &s.top_console,
                &data.doodles,
                &data.friends,
                &data.mapping,
                hover,
            );
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
            print_center("Select the friend to map to.");
            print_center("");
            s.bottom_console.select();
            pick_friend(s, data)?;
            s.bottom_console.clear();
            s.top_console.select();
            println!("\x1b[0m");
            dirty = true;
        }
    }
}

fn pick_friend(s: &mut Services, data: &mut AppData) -> Result<(), ()> {
    s.bottom_console.clear();

    let mut hover: usize = 0;
    let mut dirty = true;

    for (pid, mii) in &data.friends {
        println!("- {}", mii.mii_name);
    }

    loop {
        s.process()?;

        if dirty {
            dirty = false;
            s.bottom_console.clear();
            print_bottom_screen(&data, hover);
        }

        if s.hid.keys_down().contains(KeyPad::DPAD_UP) {
            if hover != 0 {
                dirty = true;
                hover -= 1;
            }
        }

        if s.hid.keys_down().contains(KeyPad::DPAD_DOWN) {
            if hover != (data.friends.len() - 1) {
                dirty = true;
                hover += 1;
            }
        }

        if s.hid.keys_down().contains(KeyPad::A) {
            return Ok(());
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
