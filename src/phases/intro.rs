use std::{
    collections::HashMap,
    io::{self, Write},
};

use ctru::prelude::KeyPad;
use libdoodle::mii_data::MiiData;

use crate::{
    AppData, Services, extdata,
    friend_list::{self, MiiMap},
    phases::print_center,
};

pub fn intro(s: &mut Services, data: &mut AppData) -> Result<(), ()> {
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
