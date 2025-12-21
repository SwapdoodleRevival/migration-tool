use crate::{control_flow::MigrationFlow, gui::Gui};
use ctru::prelude::*;
use std::process::{ExitCode, Termination};

mod control_flow;
pub(crate) mod error;
mod extdata;
mod friend_list;
mod gui;
mod phases;
mod read;

fn main() -> ExitCode {
    ctru::set_panic_hook(true);
    match run() {
        MigrationFlow::Continue(v) => v.report(),
        MigrationFlow::Break(v) => v.report(),
    }
}

fn run() -> MigrationFlow {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx: Gfx = Gfx::new().unwrap();
    let _console = Console::new(gfx.bottom_screen.borrow_mut());
    let mut gui = Gui::init();

    phases::intro(&apt, &gfx, &mut hid, &mut gui).run()?;
    let (extdata, read_data) = phases::reading(&apt, &gfx, &mut hid, &mut gui).run()?;
    let mapping = phases::mapping(&apt, &gfx, &mut hid, &mut gui).run(read_data)?;
    phases::rewrite(&apt, &gfx, &mut hid, &mut gui).run(extdata, mapping)?;
    MigrationFlow::Continue(())
}
