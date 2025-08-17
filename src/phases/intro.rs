use std::{
    collections::HashMap,
    io::{self, Write},
};

use ctru::prelude::KeyPad;
use libdoodle::{blocks::miistd1::MiiData, files::letter::Letter};

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
    println!("\x1b[37;41m");
    println!("Please note: This tool does not back up");
    println!("your save data before migrating!");
    println!();
    println!("If you do *not* have a backup,");
    println!("make one now using Checkpoint.");
    println!();
    println!("\x1b[0m");
    print_center("Press (A) to begin");
    println!();
    print_center("Press (START) to exit");
    print_center("(you can do this at any time)");
    println!();

    loop {
        s.process()?;

        if s.hid.keys_down().contains(KeyPad::A) {
            return Ok(());
        }
    }
}
