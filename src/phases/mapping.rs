use ctru::prelude::{Console, KeyPad};

use crate::{AppData, Remapping, Services, friend_list::MiiMap, phases::print_center};

pub fn mapping(s: &mut Services, data: &mut AppData) -> Result<(), ()> {
    let mut hover: usize = 0;
    let mut dirty = true;
    auto_match_by_mac(data);
    s.bottom_console.select();
    //                                                |
    println!("The migration tool has attempted ");
    println!("to automatically match");
    println!("doodle senders (left)");
    println!("to your friends (right),");
    println!("based on Mii data.");
    println!();
    println!("You are now free to");
    println!("change these as you please.");
    println!();
    println!("Use Up/Down to move the cursor,");
    println!("and A to change the mapping.");
    s.top_console.select();

    loop {
        s.process()?;

        if dirty {
            dirty = false;
            print_mapping_picker(
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
            print!("\x1b[26;0H\x1b[1;37;41m");
            print_center("");
            print_center("Select the friend to map to.");
            print_center("(A) Select, (B) Cancel, (X) Clear");
            print_center("");
            s.bottom_console.select();

            if let Some(friend_pid) = pick_friend(s, data)? {
                let doodler_pid = get_nth(hover, &data.doodles).unwrap();
                if friend_pid != 0 {
                    data.mapping.insert(doodler_pid, friend_pid);
                } else {
                    data.mapping.remove(&doodler_pid);
                }
            }

            s.bottom_console.clear();
            s.top_console.select();
            println!("\x1b[0m");
            dirty = true;
        }
    }
}

fn auto_match_by_mac(data: &mut AppData) {
    for doodler in &data.doodles {
        let mac = doodler.1.creator_mac_address;
        for friend in &data.friends {
            if friend.1.creator_mac_address == mac {
                data.mapping.insert(*doodler.0, *friend.0);
                break;
            }
        }
    }
}

fn pick_friend(s: &mut Services, data: &mut AppData) -> Result<Option<u32>, ()> {
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
            print_friend_picker(&data, hover);
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
            return Ok(Some(get_nth(hover, &data.friends).unwrap()));
        }

        if s.hid.keys_down().contains(KeyPad::B) {
            return Ok(None);
        }

        if s.hid.keys_down().contains(KeyPad::X) {
            return Ok(Some(0));
        }
    }
}

fn get_nth(mut index: usize, container: &MiiMap) -> Option<u32> {
    for element in container {
        if index == 0 {
            return Some(*element.0);
        }
        index -= 1;
    }
    return None;
}

fn print_mapping_picker(
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
            "{} {} {: <23}{: >23} {}",
            if index == hover { "\x1b[37;44m" } else { "" },
            if index == hover { '>' } else { ' ' },
            mii.mii_name,
            friend_name,
            if index == hover { "\x1b[0m" } else { "" },
        );
        index += 1;
    }
}

fn print_friend_picker(data: &AppData, hover: usize) {
    let mut index: usize = 0;

    for (_pid, mii) in &data.friends {
        println!(
            "{} {} {: <37}{}",
            if index == hover { "\x1b[37;44m" } else { "" },
            if index == hover { '>' } else { ' ' },
            mii.mii_name,
            if index == hover { "\x1b[0m" } else { "" },
        );
        index += 1;
    }
}
