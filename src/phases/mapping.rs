use ctru::prelude::{Console, KeyPad};

use crate::{
    AppData, Remapping, Services,
    friend_list::MiiMap,
    phases::print_center,
};

pub fn mapping(s: &mut Services, data: &mut AppData) -> Result<(), ()> {
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
