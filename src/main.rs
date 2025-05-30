use ctru::prelude::*;

mod friend_list;
mod extdata;

fn main() {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx = Gfx::new().unwrap();
    // let mut soc = Soc::new().unwrap();
    // soc.redirect_to_3dslink(true, true).unwrap();
    ctru::applets::error::set_panic_hook(true);

    let topConsole = Console::new(gfx.top_screen.borrow_mut());
    let bottomConsole = Console::new(gfx.bottom_screen.borrow_mut());

    topConsole.select();

    for (pid, mii) in friend_list::load_friend_list() {
        println!("{}: {}", pid, mii.mii_name);
    }

    println!("Reading notes:");
    for (file, filename, letter) in extdata::read() {
        println!("Got {} from {}", filename, match letter.sender_mii {
            Some(mii) => mii.mii_name,
            None => "<no mii>".to_string()
        });
    }

    while apt.main_loop() {
        gfx.wait_for_vblank();

        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        }
    }
}
