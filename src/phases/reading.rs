use std::{
    collections::HashMap,
    io::{self, Write},
};

use ctru::{prelude::KeyPad, services::cfgu::Region};
use libdoodle::{blocks::miistd1::MiiData, files::letter::Letter};

use crate::{
    AppData, Services,
    extdata::{self, ExtdataArchive, SwapdoodleRegion},
    friend_list::{self, MiiMap},
};

pub fn reading(s: &mut Services, data: &mut AppData) -> Result<ExtdataArchive, ()> {
    s.top_console.clear();

    println!("We will begin by reading your Friend List");
    println!("and Swapdoodle extdata.");
    println!();
    println!("Press (A) to begin.");
    println!();

    let extdata = ExtdataArchive::open(SwapdoodleRegion::EU).unwrap();

    loop {
        s.process()?;

        if s.hid.keys_down().contains(KeyPad::A) {
            (data.friends, data.doodles) = friendly_read_data(&extdata);
            return Ok(extdata);
        }
    }
}

fn friendly_read_data(extdata: &ExtdataArchive) -> (MiiMap, MiiMap) {
    print!("Reading your friend list... ");
    _ = io::stdout().flush();

    let friends = friend_list::load_friend_list();
    println!("done!");

    print!("Reading your Swapdoodle extdata... ");
    _ = io::stdout().flush();
    let mut doodles = HashMap::<u32, MiiData>::new();
    for (_file, _filename, letter) in extdata.read::<Letter>() {
        if letter.common.sender_pid != 0
            && let Some(mii) = letter.sender_mii
        {
            doodles.insert(letter.common.sender_pid, mii);
        }
    }
    println!("done!");

    (friends, doodles)
}
