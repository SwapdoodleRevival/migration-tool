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
        let key = row.1;
        let folder = key / 200;
        let filename = format!("/letter/{:04}/lt{:04}.bin", folder, key);
        println!("Reading file {}...", filename);
        if let Some(mii) = Letter::new_from_bpk1_bytes(&extdata.read_file(&filename))
            .unwrap()
            .sender_mii
        {
            doodles.insert(*row.0, mii);
        }
    });

    println!("All done!");

    (friends, doodles)
}
