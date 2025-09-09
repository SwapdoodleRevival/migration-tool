use std::{collections::HashMap, ffi::CString, str::FromStr};

use citro2d_sys::{C2D_AlignRight, C2D_AtBaseline, C2D_Text};
use ctru::prelude::KeyPad;

use crate::{
    Services,
    friend_list::MiiMap,
    gui::{GUI, TOP_SCREEN_HEIGHT, TOP_SCREEN_WIDTH, TextBuffer},
    phases::ReadResult,
};

//                                .- PID of note sender
//                                v      .- PID of friend
pub type OldToNewPIDMapping = HashMap<u32, u32>;

pub fn mapping(s: &mut Services, read: ReadResult) -> Result<OldToNewPIDMapping, ()> {
    let mut mapping = OldToNewPIDMapping::new();
    auto_match_by_mac(&mut mapping, &read);
    let mut scene = Scene::make(s.gui, &read);

    loop {
        s.process()?;
        scene.begin_paint();
        scene.dialog_explanation();
        scene.end_paint();

        if s.hid.keys_down().contains(KeyPad::A) {
            break;
        }
    }

    pick_mapping(s, &mut scene, &read, &mut mapping)?;

    Ok(mapping)
}

fn auto_match_by_mac(mapping: &mut OldToNewPIDMapping, read: &ReadResult) {
    for doodler in &read.doodles {
        let mac = doodler.1.creator_mac_address;
        for friend in &read.friends {
            if friend.1.creator_mac_address == mac {
                mapping.insert(*doodler.0, *friend.0);
                break;
            }
        }
    }
}

fn pick_mapping(
    s: &mut Services,
    scene: &mut Scene,
    read: &ReadResult,
    mapping: &mut OldToNewPIDMapping,
) -> Result<(), ()> {
    let mut dirty = true;

    loop {
        s.process()?;
        if dirty {
            scene.begin_paint();
            scene.paint_mapping(mapping, &read.doodles);
            scene.end_paint();
            dirty = false;
        }

        if s.hid.keys_down().contains(KeyPad::DPAD_DOWN) {
            dirty = true;
            scene.down(&read.doodles);
        } else if s.hid.keys_down().contains(KeyPad::DPAD_UP) {
            dirty = true;
            scene.up(&read.doodles);
        } else if s.hid.keys_down().contains(KeyPad::A) {
            let pid = *read
                .doodles
                .iter()
                .enumerate()
                .find(|i| i.0 == scene.get_index())
                .unwrap()
                .1
                .0;
            pick_friend(pid, s, scene, read, mapping)?;
            dirty = true;
        } else if s.hid.keys_down().contains(KeyPad::Y) {
            return Ok(());
        }
    }
}

fn pick_friend(
    pid: u32,
    s: &mut Services,
    scene: &mut Scene,
    read: &ReadResult,
    mapping: &mut OldToNewPIDMapping,
) -> Result<(), ()> {
    let mut dirty = true;

    loop {
        s.process()?;
        if dirty {
            scene.paint_whole_friend_picker(pid, &read.friends);
            scene.end_paint();
            dirty = false;
        }
        if s.hid.keys_down().contains(KeyPad::DPAD_DOWN) {
            dirty = true;
            scene.friend_down(&read.friends);
        } else if s.hid.keys_down().contains(KeyPad::DPAD_UP) {
            dirty = true;
            scene.friend_up(&read.friends);
        } else if s.hid.keys_down().contains(KeyPad::A) {
            let new_pid = *read
                .friends
                .iter()
                .enumerate()
                .find(|i| i.0 == scene.get_friend_index())
                .unwrap()
                .1
                .0;
            mapping.insert(pid, new_pid);
            return Ok(());
        } else if s.hid.keys_down().contains(KeyPad::X) {
            mapping.remove(&pid);
            return Ok(());
        } else if s.hid.keys_down().contains(KeyPad::B) {
            return Ok(());
        }
    }
}

struct Scene<'a> {
    gui: &'a GUI,
    textbuf: TextBuffer,
    header_text: C2D_Text,
    explanation_line1: C2D_Text,
    explanation_line2: C2D_Text,
    explanation_line3: C2D_Text,
    explanation_line4: C2D_Text,
    explanation_line5: C2D_Text,
    explanation_line6: C2D_Text,
    press_a_continue: C2D_Text,
    press_y_done: C2D_Text,
    press_x_clear: C2D_Text,
    press_b_back: C2D_Text,
    scroll_for_more: C2D_Text,
    dont_map: C2D_Text,
    remapping: C2D_Text,
    names: HashMap<u32, C2D_Text>,
    index: usize,
    index_friend: usize,
}

impl<'a> Scene<'a> {
    pub fn make(gui: &'a GUI, read: &ReadResult) -> Self {
        let textbuf = TextBuffer::init(4096 * 8);

        let mut names = HashMap::<u32, C2D_Text>::new();
        for (pid, mii) in read.doodles.iter() {
            let s = CString::from_str(&mii.mii_name).unwrap_or_default();
            names.insert(*pid, textbuf.make_static_text(&s));
        }
        for (pid, mii) in read.friends.iter() {
            let s = CString::from_str(&mii.mii_name).unwrap_or_default();
            names.insert(*pid, textbuf.make_static_text(&s));
        }

        Scene {
            gui,
            header_text: textbuf.make_static_text(c"Mapping"),
            explanation_line1: textbuf
                .make_static_text(c"The migration tool has attempted to automatically"),
            explanation_line2: textbuf
                .make_static_text(c"match unknown Doodle authors to your friends."),
            explanation_line3: textbuf.make_static_text(c"You can change the suggested mapping"),
            explanation_line4: textbuf.make_static_text(c"before you begin the migration."),
            explanation_line5: textbuf.make_static_text(c"Highlight a mapping with \u{E07D} "),
            explanation_line6: textbuf.make_static_text(c"and press \u{E000} to change it."),
            press_a_continue: textbuf.make_static_text(c"Press \u{E000} to continue"),
            press_b_back: textbuf.make_static_text(c"\u{E001} Back"),
            press_x_clear: textbuf.make_static_text(c"\u{E002} Clear"),
            press_y_done: textbuf.make_static_text(c"\u{E003} Finish"),
            scroll_for_more: textbuf.make_static_text(c"... scroll for more ..."),
            dont_map: textbuf.make_static_text(c"<don't map>"),
            remapping: textbuf.make_static_text(c"Remapping"),
            names,
            textbuf,
            index: 0,
            index_friend: 0,
        }
    }

    pub fn begin_paint(&self) {
        self.gui.begin_frame();
        self.gui.header_small(&self.header_text);
    }

    pub fn dialog_explanation(&self) {
        const DIALOG_PADDING: f32 = 40.0;
        self.gui.dialog();
        self.gui.blue_rect(
            DIALOG_PADDING,
            DIALOG_PADDING,
            TOP_SCREEN_WIDTH - 2.0 * DIALOG_PADDING,
            TOP_SCREEN_HEIGHT - 2.0 * DIALOG_PADDING,
        );
        self.gui.text(
            &self.explanation_line1,
            DIALOG_PADDING + 10.0,
            DIALOG_PADDING + 10.0,
            0,
            0.5,
        );
        self.gui.text(
            &self.explanation_line2,
            DIALOG_PADDING + 10.0,
            DIALOG_PADDING + 25.0,
            0,
            0.5,
        );
        self.gui.text(
            &self.explanation_line3,
            DIALOG_PADDING + 10.0,
            DIALOG_PADDING + 50.0,
            0,
            0.5,
        );
        self.gui.text(
            &self.explanation_line4,
            DIALOG_PADDING + 10.0,
            DIALOG_PADDING + 65.0,
            0,
            0.5,
        );
        self.gui.text(
            &self.explanation_line5,
            DIALOG_PADDING + 10.0,
            DIALOG_PADDING + 90.0,
            0,
            0.5,
        );
        self.gui.text(
            &self.explanation_line6,
            DIALOG_PADDING + 10.0,
            DIALOG_PADDING + 105.0,
            0,
            0.5,
        );
        self.gui.text(
            &self.press_a_continue,
            DIALOG_PADDING + 10.0,
            DIALOG_PADDING + 130.0,
            0,
            0.7,
        );
    }

    pub fn end_paint(&self) {
        self.gui.end_frame();
    }

    pub fn down(&mut self, doodles: &MiiMap) {
        self.index = self.index.saturating_add(1);
        if self.index >= doodles.len() {
            self.index = 0;
        }
    }

    pub fn up(&mut self, doodles: &MiiMap) {
        if self.index == 0 {
            self.index = doodles.len() - 1;
        } else {
            self.index = self.index.saturating_sub(1);
        }
    }

    pub fn friend_down(&mut self, friends: &MiiMap) {
        self.index_friend = self.index_friend.saturating_add(1);
        if self.index_friend >= friends.len() {
            self.index_friend = 0;
        }
    }

    pub fn friend_up(&mut self, friends: &MiiMap) {
        if self.index_friend == 0 {
            self.index_friend = friends.len() - 1;
        } else {
            self.index_friend = self.index_friend.saturating_sub(1);
        }
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_friend_index(&self) -> usize {
        self.index_friend
    }

    pub fn paint_mapping(&self, mapping: &OldToNewPIDMapping, doodles: &MiiMap) {
        const PAGE_SIZE: usize = 10;
        let mut line: usize = 0;

        self.gui.rect(
            0.0,
            20.0,
            TOP_SCREEN_WIDTH / 2.0,
            TOP_SCREEN_HEIGHT - 20.0 - 15.0,
            self.gui.dark_green,
        );
        self.gui.rect(
            TOP_SCREEN_WIDTH / 2.0,
            20.0,
            TOP_SCREEN_WIDTH / 2.0,
            TOP_SCREEN_HEIGHT - 20.0 - 15.0,
            self.gui.dark_purple,
        );

        for (i, (pid, mii)) in doodles.iter().enumerate() {
            if line == PAGE_SIZE {
                self.gui.text(&self.scroll_for_more, 10.0, 225.0, 0, 0.45);
                break;
            }

            if i < ((self.index / PAGE_SIZE) * PAGE_SIZE) {
                continue;
            }

            let ypos: f32 = 25.0 + line as f32 * 20.0;

            if i == self.index {
                self.gui.highlight(0.0, ypos, TOP_SCREEN_WIDTH, 20.0);
            }

            self.gui
                .text(self.names.get(pid).unwrap(), 10.0, ypos, 0, 0.6);

            self.gui.text(
                match mapping.get(pid) {
                    Some(new) => self.names.get(new).unwrap(),
                    None => &self.dont_map,
                },
                TOP_SCREEN_WIDTH - 10.0,
                ypos,
                C2D_AlignRight,
                0.6,
            );

            line += 1;
        }
        self.gui.text(
            &self.press_y_done,
            TOP_SCREEN_WIDTH - 10.0,
            225.0,
            C2D_AlignRight,
            0.5,
        );
    }

    pub fn paint_whole_friend_picker(&self, pid: u32, friends: &MiiMap) {
        const PAGE_SIZE: usize = 10;

        unsafe {
            self.gui.begin_frame();
            self.gui.blue_rect(0.0, 0.0, TOP_SCREEN_WIDTH, 20.0);
            self.gui.text(
                &self.press_b_back,
                TOP_SCREEN_WIDTH - 10.0,
                225.0,
                C2D_AlignRight,
                0.5,
            );
            self.gui.text(
                &self.press_x_clear,
                TOP_SCREEN_WIDTH - 70.0,
                225.0,
                C2D_AlignRight,
                0.5,
            );
            self.gui
                .text(&self.remapping, 10.0, 15.0, C2D_AtBaseline, 0.7);
            self.gui.text(
                self.names.get(&pid).unwrap(),
                110.0,
                15.0,
                C2D_AtBaseline,
                0.7,
            );
            let mut line: usize = 0;
            for (i, (pid, mii)) in friends.iter().enumerate() {
                if line == PAGE_SIZE {
                    self.gui.text(&self.scroll_for_more, 10.0, 225.0, 0, 0.45);
                    break;
                }

                if i < ((self.index_friend / PAGE_SIZE) * PAGE_SIZE) {
                    continue;
                }

                let ypos: f32 = 25.0 + line as f32 * 20.0;

                if i == self.index_friend {
                    self.gui.highlight(0.0, ypos, TOP_SCREEN_WIDTH, 20.0);
                }

                self.gui
                    .text(self.names.get(pid).unwrap(), 10.0, ypos, 0, 0.6);

                line += 1;
            }
        }
    }
}
