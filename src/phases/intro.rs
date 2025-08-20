use std::{
    collections::HashMap,
    io::{self, Write},
    mem,
};

use crate::{
    Services,
    gui::{GUI, TOP_SCREEN_WIDTH, TextBuffer},
};
use citro2d_sys::{
    C2D_AlignCenter, C2D_AtBaseline, C2D_Color32, C2D_DrawRectSolid, C2D_SceneBegin,
    C2D_TargetClear, C2D_Text, C2D_TextBuf, C2D_TextBufDelete, C2D_TextOptimize, C2D_WithColor,
};
use citro3d_sys::{C3D_FRAME_SYNCDRAW, C3D_FrameBegin, C3D_FrameEnd};
use ctru::prelude::KeyPad;

pub fn intro(s: &mut Services) -> Result<(), ()> {
    let scene = Scene::make(s.gui);
    scene.paint(); // only needed once

    loop {
        s.process()?;

        if s.hid.keys_down().contains(KeyPad::A) {
            return Ok(());
        }
    }
}

struct Scene<'a> {
    gui: &'a GUI,
    white: u32,
    red: u32,
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
        unsafe {
            let textbuf = TextBuffer::init(4096);

            Scene {
                gui,
                white: C2D_Color32(255, 255, 255, 255),
                red: C2D_Color32(255, 0, 0, 255),
                header_text: textbuf.make_static_text(c"Swapdoodle Migration Tool"),
                intro_line1_text: textbuf
                    .make_static_text(c"This tool will help you migrate your Swapdoodle notes"),
                intro_line2_text: textbuf
                    .make_static_text(c"from a Nintendo environment to a Pretendo environment."),
                goal_line1_text: textbuf
                    .make_static_text(c"After using this tool, your notes will be moved"),
                goal_line2_text: textbuf
                    .make_static_text(c"from \"Unknown sender\" to your friends' profiles."),
                nobkp_line1_text: textbuf.make_static_text(
                    c"Please note: This tool does not back up your extra data before migrating it!",
                ),
                nobkp_line2_text: textbuf.make_static_text(
                    c"If you do not have an EXTRA DATA backup, make one now using Checkpoint.",
                ),
                nobkp_line3_text: textbuf.make_static_text(
                    c"(open Checkpoint, press \u{E002} for extra data, and back up Swapdoodle)",
                ),
                begin: textbuf.make_static_text(c"Press \u{E000} to begin"),
                exit: textbuf.make_static_text(c"Press Start to exit"),
                exit_anytime: textbuf.make_static_text(c"(you can do this at any point)"),
                textbuf: textbuf,
            }
        }
    }

    pub fn paint(&self) {
        unsafe {
            C3D_FrameBegin(C3D_FRAME_SYNCDRAW);
            C2D_TargetClear(self.gui.screen, C2D_Color32(20, 20, 20, 255));
            C2D_SceneBegin(self.gui.screen);
            let bg = C2D_Color32(0, 40, 199, 255);
            C2D_DrawRectSolid(0.0, 0.0, 0.0, TOP_SCREEN_WIDTH, 30.0, bg);
            TextBuffer::draw(
                &self.header_text,
                TOP_SCREEN_WIDTH / 2.0,
                22.0,
                C2D_WithColor | C2D_AlignCenter | C2D_AtBaseline,
                self.white,
                0.7,
            );
            TextBuffer::draw(
                &self.intro_line1_text,
                10.0,
                40.0,
                C2D_WithColor,
                self.white,
                0.5,
            );
            TextBuffer::draw(
                &self.intro_line2_text,
                10.0,
                55.0,
                C2D_WithColor,
                self.white,
                0.5,
            );
            TextBuffer::draw(
                &self.goal_line1_text,
                10.0,
                80.0,
                C2D_WithColor,
                self.white,
                0.5,
            );
            TextBuffer::draw(
                &self.goal_line2_text,
                10.0,
                95.0,
                C2D_WithColor,
                self.white,
                0.5,
            );

            TextBuffer::draw(
                &self.nobkp_line1_text,
                10.0,
                120.0,
                C2D_WithColor,
                self.red,
                0.4,
            );
            TextBuffer::draw(
                &self.nobkp_line2_text,
                10.0,
                130.0,
                C2D_WithColor,
                self.red,
                0.4,
            );
            TextBuffer::draw(
                &self.nobkp_line3_text,
                10.0,
                140.0,
                C2D_WithColor,
                self.red,
                0.4,
            );

            TextBuffer::draw(
                &self.begin,
                TOP_SCREEN_WIDTH / 2.0,
                160.0,
                C2D_WithColor | C2D_AlignCenter,
                self.white,
                0.7,
            );
            TextBuffer::draw(
                &self.exit,
                TOP_SCREEN_WIDTH / 2.0,
                190.0,
                C2D_WithColor | C2D_AlignCenter,
                self.white,
                0.7,
            );
            TextBuffer::draw(
                &self.exit_anytime,
                TOP_SCREEN_WIDTH / 2.0,
                210.0,
                C2D_WithColor | C2D_AlignCenter,
                self.white,
                0.5,
            );

            C3D_FrameEnd(0);
        }
    }
}
