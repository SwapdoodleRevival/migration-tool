use std::{collections::HashMap, io::Write};

use std::io::{Cursor, Seek};

use ctru::prelude::KeyPad;
use libdoodle::{
    blocks::common1,
    bpk1::{BPK1Blocks, BPK1File},
};

use crate::extdata::ExtdataArchive;
use crate::{AppData, Services, extdata, read::ReadExt};

pub fn rewrite(
    s: &mut Services,
    extdata: &mut ExtdataArchive,
    data: &mut AppData,
) -> Result<(), ()> {
    s.bottom_console.clear();
    s.top_console.clear();
    println!("The tool will now start rewriting your Swapdoodle save data.");
    println!("Reminder: THIS TOOL DOES NOT CREATE A BACKUP!!!");
    println!("If you do *not* have one, DO NOT CONTINUE!!!");
    println!();
    println!("Press (A) to begin");
    println!("Press (Start) to exit");

    loop {
        s.process()?;

        if s.hid.keys_down().contains(KeyPad::A) {
            break;
        }
    }

    s.top_console.clear();
    do_rewrite(extdata, &data.mapping);
    println!("\n");
    println!("Done!!!");
    println!("Press (A) to exit");

    loop {
        s.process()?;

        if s.hid.keys_down().contains(KeyPad::A) {
            break;
        }
    }

    Ok(())
}

fn do_rewrite(extdata: &mut ExtdataArchive, mapping: &HashMap<u32, u32>) {
    println!("Reading manage.bin...");
    let mut manage = BPK1Blocks::new_from_bpk1_bytes(&extdata.read_manage()).unwrap();
    let cominf = manage
        .iter_mut()
        .find(|k| k.name.as_bytes() == b"COMINF0")
        .expect("manage.bin should have a COMINF0, but it doesn't!");

    let mut cursor = Cursor::new(&mut cominf.data);

    let count = cursor.read_u32_le().unwrap();
    cursor.set_position(0x40);
    for _ in 0..count {
        let pos = cursor.position();
        let sender_pid =
            common1::CommonInfo::from_bytes(&(cursor.read_const_num_of_bytes::<0x40>().unwrap()))
                .unwrap()
                .sender_pid;

        if let Some(new_pid) = mapping.get(&sender_pid) {
            let letter_key = cursor.read_u32_le().unwrap();
            cursor.set_position(pos + 24);
            cursor.write_all(&u32::to_le_bytes(*new_pid)).unwrap();

            let mut letter = BPK1Blocks::new_from_bpk1_bytes(&extdata.read_letter_index(letter_key)).unwrap();
            let common_block = match letter.iter_mut().find(|k| k.name.as_bytes() == b"COMMON1") {
                Some(k) => k,
                None => continue,
            };

            common_block.data[24..28].copy_from_slice(&u32::to_le_bytes(*new_pid));

            let out = BPK1Blocks::bytes_from_bpk1_blocks(letter).unwrap();

            extdata.write_letter_index(letter_key, &out);
        }

        cursor.set_position(pos + 0x80);
    }

    println!("Rewriting manage.bin...");
    extdata.write_file(
        "/letter/manage.bin",
        &(BPK1Blocks::bytes_from_bpk1_blocks(manage).unwrap()),
    );
    println!("Rewrote manage.bin.");
}
