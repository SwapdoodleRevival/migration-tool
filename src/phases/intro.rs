use crate::{
    Services,
    gui::{GUI, TOP_SCREEN_WIDTH, TextBuffer},
};
use citro2d_sys::{C2D_AlignCenter, C2D_Text};
use ctru::prelude::KeyPad;

pub fn intro(s: &mut Services) -> Result<(), ()> {
    let scene = Scene::make(s.gui);

    loop {
        s.process()?;
        scene.paint();

        if s.hid.keys_down().contains(KeyPad::A) {
            return Ok(());
        }
    }
}

struct Scene<'a> {
    gui: &'a GUI,
    textbuf: TextBuffer,
    header_text: C2D_Text,
    intro_line1_text: C2D_Text,
    intro_line2_text: C2D_Text,
    goal_line1_text: C2D_Text,
    goal_line2_text: C2D_Text,
    nobkp_line1_text: C2D_Text,
    nobkp_line2_text: C2D_Text,
    nobkp_line3_text: C2D_Text,
    begin: C2D_Text,
    exit: C2D_Text,
    exit_anytime: C2D_Text,
}

impl<'a> Scene<'a> {
    pub fn make(gui: &'a GUI) -> Self {
        let textbuf = TextBuffer::init(4096);

        Scene {
            gui,
            header_text: textbuf.make_static_text(c"Swapdoodle Migration Tool"),
            intro_line1_text: textbuf.make_static_text(c"This tool will help you migrate your Swapdoodle notes"),
            intro_line2_text: textbuf.make_static_text(c"from a Nintendo environment to a Pretendo environment."),
            goal_line1_text: textbuf.make_static_text(c"After using this tool, your notes will be moved"),
            goal_line2_text: textbuf.make_static_text(c"from \"Unknown sender\" to your friends' profiles."),
            nobkp_line1_text: textbuf.make_static_text(c"Please note: This tool does not back up your extra data before migrating it!"),
            nobkp_line2_text: textbuf.make_static_text(c"If you do not have an EXTRA DATA backup, make one now using Checkpoint."),
            nobkp_line3_text: textbuf.make_static_text(c"(open Checkpoint, press \u{E002} for extra data, and back up Swapdoodle)"),
            begin: textbuf.make_static_text(c"Press \u{E000} to begin"),
            exit: textbuf.make_static_text(c"Press Start to exit"),
            exit_anytime: textbuf.make_static_text(c"(you can do this at any point)"),
            textbuf,
        }
    }

    pub fn paint(&self) {
        self.gui.begin_frame();
        self.gui.header(&self.header_text);
        self.gui.text(&self.intro_line1_text, 10.0, 40.0, 0, 0.5);
        self.gui.text(&self.intro_line2_text, 10.0, 55.0, 0, 0.5);
        self.gui.text(&self.goal_line1_text, 10.0, 80.0, 0, 0.5);
        self.gui.text(&self.goal_line2_text, 10.0, 95.0, 0, 0.5);

        self.gui.text_danger(&self.nobkp_line1_text, 10.0, 120.0, 0, 0.4);
        self.gui.text_danger(&self.nobkp_line2_text, 10.0, 130.0, 0, 0.4);
        self.gui.text_danger(&self.nobkp_line3_text, 10.0, 140.0, 0, 0.4);

        self.gui.text(&self.begin, TOP_SCREEN_WIDTH / 2.0, 160.0, C2D_AlignCenter, 0.7);
        self.gui.text(&self.exit, TOP_SCREEN_WIDTH / 2.0, 190.0, C2D_AlignCenter, 0.7);
        self.gui.text(&self.exit_anytime, TOP_SCREEN_WIDTH / 2.0, 210.0, C2D_AlignCenter, 0.5);

        self.gui.end_frame();
    }
}
