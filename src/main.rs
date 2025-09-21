use ctru::prelude::*;

use crate::gui::Gui;

mod extdata;
mod friend_list;
mod gui;
mod phases;
mod read;

fn main() {
    ctru::set_panic_hook(true);
    _ = run();
}

fn run() -> Result<(), ()> {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx: Gfx = Gfx::new().unwrap();
    let _console = Console::new(gfx.bottom_screen.borrow_mut());
    let mut gui = Gui::init();

    phases::intro(&apt, &gfx, &mut hid, &mut gui).run()?;
    let (extdata, read_data) = phases::reading(&apt, &gfx, &mut hid, &mut gui).run()?;
    let mapping = phases::mapping(&apt, &gfx, &mut hid, &mut gui).run(read_data)?;
    phases::rewrite(&apt, &gfx, &mut hid, &mut gui).run(extdata, mapping)?;
    Ok(())
}
