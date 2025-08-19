use std::{
    collections::HashMap,
    io::{self, Cursor, Write},
};

use ctru::{prelude::KeyPad, services::cfgu::Region};
use libdoodle::{
    blocks::{common1, miistd1::MiiData},
    bpk1::{BPK1Blocks, BPK1File},
    files::letter::Letter,
};

use crate::{
    AppData, Services,
    extdata::{self, ExtdataArchive, SwapdoodleRegion},
    friend_list::{self, MiiMap},
    read::ReadExt,
};

pub fn reading(s: &mut Services, data: &mut AppData) -> Result<ExtdataArchive, ()> {
    s.top_console.clear();

    println!("We will begin by reading your Friend List");
    println!("and Swapdoodle extdata.");
    println!();

    let extdatas = (
        ExtdataArchive::open(SwapdoodleRegion::EU),
        ExtdataArchive::open(SwapdoodleRegion::US),
        ExtdataArchive::open(SwapdoodleRegion::JP),
    );

    let available: u8 = if extdatas.0.is_ok() { 1 } else { 0 }
        + if extdatas.1.is_ok().into() { 1 } else { 0 }
        + if extdatas.2.is_ok().into() { 1 } else { 0 };

    if available == 0 {
        println!("It appears you do not have any Swapdoodle extdata.");
        println!("If you believe this is in error, please let us know!");
        println!();
        println!("Press (A) to exit");

        loop {
            s.process()?;

            if s.hid.keys_down().contains(KeyPad::A) {
                return Err(());
            }
        }
    }

    let extdata;

    if available == 1 {
        extdata = extdatas
            .0
            .unwrap_or_else(|_| extdatas.1.unwrap_or_else(|_| extdatas.2.unwrap()));
        println!(
            "Detected region: {}",
            match extdata.region {
                SwapdoodleRegion::EU => "EU",
                SwapdoodleRegion::US => "US",
                SwapdoodleRegion::JP => "JP",
            }
        );
        println!();
        println!("Press (A) to begin reading.");

        loop {
            s.process()?;

            if s.hid.keys_down().contains(KeyPad::A) {
                break;
            }
        }
    } else {
        println!("Detected several regions.");
        println!("This tool can only work at one at a time.");
        println!("Please select a region:\n");

        match extdatas.0 {
            Ok(_) => println!("(X) EU"),
            Err(_) => {}
        };
        match extdatas.1 {
            Ok(_) => println!("(Y) US"),
            Err(_) => {}
        };
        match extdatas.2 {
            Ok(_) => println!("(B) JP"),
            Err(_) => {}
        };

        loop {
            s.process()?;

            if s.hid.keys_down().contains(KeyPad::X) {
                match extdatas.0 {
                    Ok(e) => {
                        extdata = e;
                        break;
                    }
                    Err(_) => {}
                };
            }
            if s.hid.keys_down().contains(KeyPad::Y) {
                match extdatas.1 {
                    Ok(e) => {
                        extdata = e;
                        break;
                    }
                    Err(_) => {}
                };
            }
            if s.hid.keys_down().contains(KeyPad::B) {
                match extdatas.2 {
                    Ok(e) => {
                        extdata = e;
                        break;
                    }
                    Err(_) => {}
                };
            }
        }
    }

    (data.friends, data.doodles) = friendly_read_data(&extdata);

    if data.friends.len() == 1 {
        println!("Your friend list is empty.");
        println!();
        println!("Swapdoodle notes are tied to friend data.");
        println!("I hope this doesn't sound rude, but here goes:");
        println!("If you don't have friends, there is not much we can do.");
        println!();
        println!("Feel free to re-run this tool later!");
        println!();
        println!("If you believe this is in error, please let us know!");
        println!();
        println!("Press (A) to exit.");

        loop {
            s.process()?;

            if s.hid.keys_down().contains(KeyPad::A) {
                return Err(());
            }
        }
    }

    if data.doodles.is_empty() {
        println!("We didn't find any notes from an unknown sender.");
        println!("You shouldn't need to run this tool.");
        println!();
        println!("If you believe this is in error, please let us know!");
        println!();
        println!("Press (A) to exit.");

        loop {
            s.process()?;

            if s.hid.keys_down().contains(KeyPad::A) {
                return Err(());
            }
        }
    }

    return Ok(extdata);
}

fn friendly_read_data(extdata: &ExtdataArchive) -> (MiiMap, MiiMap) {
    print!("Reading your friend list... ");
    _ = io::stdout().flush();

    let friends = friend_list::load_friend_list();
    println!("done!");

    println!("Reading your Swapdoodle extdata... ");

    println!("Reading file /letter/manage.bin...");
    let mut manage = BPK1Blocks::new_from_bpk1_bytes(&extdata.read_manage()).unwrap();
    let cominf = manage
        .iter_mut()
        .find(|k| k.name.as_bytes() == b"COMINF0")
        .expect("File /letter/manage.bin should have a COMINF0, but it doesn't!");

    let mut cursor = Cursor::new(&cominf.data);

    let mut unknown_pids = HashMap::<u32, u32>::new();

    let count = cursor.read_u32_le().unwrap();
    cursor.set_position(0x40);

    for _ in 0..count {
        let pos = cursor.position();
        let common =
            common1::CommonInfo::from_bytes(&(cursor.read_const_num_of_bytes::<0x40>().unwrap()))
                .unwrap();
        let sender_pid = common.sender_pid;

        if let None = friends.get(&sender_pid) {
            let letter_key = cursor.read_u32_le().unwrap();
            unknown_pids.insert(sender_pid, letter_key);
        }

        cursor.set_position(pos + 0x80);
    }
    println!("done.");

    let mut doodles = HashMap::<u32, MiiData>::new();

    unknown_pids.iter().for_each(|row| {
        let key = *row.1;
        if let Some(mii) = Letter::new_from_bpk1_bytes(&extdata.read_letter_index(key))
            .unwrap()
            .sender_mii
        {
            doodles.insert(*row.0, mii);
        }
    });

    println!("All done!");

    (friends, doodles)
}
