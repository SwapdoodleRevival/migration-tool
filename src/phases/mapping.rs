use std::{cell::RefCell, collections::HashMap, ffi::CString, str::FromStr};

use citro2d_sys::{C2D_AlignCenter, C2D_AlignLeft, C2D_AlignRight, C2D_AtBaseline, C2D_Text};
use ctru::prelude::{Apt, Gfx, Hid, KeyPad};

use crate::{
    friend_list::MiiMap,
    gui::{scrollable_view::{ScrollableView, ScrollableViewData}, Gui, TOP_SCREEN_HEIGHT, TOP_SCREEN_WIDTH},
    phases::{process, MigrationFlow, ReadResult},
};

//                                    .- PID of note sender
//                                    v      .- PID of friend
pub type OldToNewPIDMapping = HashMap<u32, u32>;

pub fn mapping<'a>(apt: &'a Apt, gfx: &'a Gfx, hid: &'a mut Hid, gui: &'a mut Gui) -> Scene<'a> {
    Scene::new(apt, gfx, hid, gui)
}

pub struct Scene<'a> {
    names: HashMap<u32, C2D_Text>,
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
    section_doodles: C2D_Text,
    section_friends: C2D_Text,
    dont_map: C2D_Text,
    remapping: C2D_Text,
    apt: &'a Apt,
    gfx: &'a Gfx,
    hid: &'a mut Hid,
    gui: &'a mut Gui,
}

impl<'a> Scene<'a> {
    fn new(apt: &'a Apt, gfx: &'a Gfx, hid: &'a mut Hid, gui: &'a mut Gui) -> Self {
        Scene {
            names: HashMap::<u32, C2D_Text>::new(),
            header_text: gui.textbuf.make_static_text(c"Mapping"),
            explanation_line1: gui
                .textbuf
                .make_static_text(c"The migration tool has attempted to automatically"),
            explanation_line2: gui
                .textbuf
                .make_static_text(c"match unknown Doodle authors to your friends."),
            explanation_line3: gui
                .textbuf
                .make_static_text(c"You can change the suggested mapping"),
            explanation_line4: gui
                .textbuf
                .make_static_text(c"before you begin the migration."),
            explanation_line5: gui
                .textbuf
                .make_static_text(c"Highlight a mapping with \u{E07D} "),
            explanation_line6: gui
                .textbuf
                .make_static_text(c"and press \u{E000} to change it."),
            press_a_continue: gui.textbuf.make_static_text(c"Press \u{E000} to continue"),
            press_b_back: gui.textbuf.make_static_text(c"\u{E001} Back"),
            press_x_clear: gui.textbuf.make_static_text(c"\u{E002} Clear"),
            press_y_done: gui.textbuf.make_static_text(c"\u{E003} Finish"),
            remapping: gui.textbuf.make_static_text(c"Remapping"),
            section_doodles: gui.textbuf.make_static_text(c"Doodle pals"),
            section_friends: gui.textbuf.make_static_text(c"3DS Friends"),
            dont_map: gui.textbuf.make_static_text(c"<don't map>"),
            apt,
            gfx,
            hid,
            gui,
        }
    }

    pub fn run(mut self, read: ReadResult) -> MigrationFlow<OldToNewPIDMapping> {
        let mapping = RefCell::new(OldToNewPIDMapping::new());

        Self::auto_match_by_mac(&mapping, &read);

        for (pid, mii) in read.doodles.iter() {
            self.names.insert(
                *pid,
                self.gui
                    .textbuf
                    .make_static_text(&CString::from_str(&mii.mii_name).unwrap_or_default()),
            );
        }
        for (pid, mii) in read.friends.iter() {
            self.names.insert(
                *pid,
                self.gui
                    .textbuf
                    .make_static_text(&CString::from_str(&mii.mii_name).unwrap_or_default()),
            );
        }

        loop {
            process(self.apt, self.gfx, self.hid)?;
            self.begin_paint();
            self.dialog_explanation();
            self.end_paint();

            if self.hid.keys_down().contains(KeyPad::A) {
                break;
            }
        }

        self.mapping_editor(&mapping, &read)?;

        MigrationFlow::Continue(mapping.into_inner())
    }

    fn auto_match_by_mac(mapping: &RefCell<OldToNewPIDMapping>, read: &ReadResult) {
        let mut mapping = mapping.borrow_mut();
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

    fn mapping_editor(
        &mut self,
        mapping: &RefCell<OldToNewPIDMapping>,
        read: &ReadResult,
    ) -> MigrationFlow {
        let mapping_picker =
            MappingPicker::new(mapping, &read.doodles, &self.names, &self.dont_map);
        let friends_picker = FriendPicker::new(&read.friends, &self.names);

        let mut view =
            ScrollableView::new(&mapping_picker, 0.0, 20.0, 200.0, TOP_SCREEN_WIDTH, 20.0);

        loop {
            process(self.apt, self.gfx, self.hid)?;
            self.begin_paint();
            self.paint_mapping();
            view.render(self.gui);
            self.end_paint();

            if self.hid.keys_down().contains(KeyPad::DPAD_DOWN) {
                view.down();
            } else if self.hid.keys_down().contains(KeyPad::DPAD_UP) {
                view.up();
            } else if self.hid.keys_down().contains(KeyPad::A) {
                let doodle_pal_pid = *read
                    .doodles
                    .iter()
                    .enumerate()
                    .find(|i| i.0 == view.current())
                    .unwrap()
                    .1
                    .0;

                {
                    let mut view = ScrollableView::new(
                        &friends_picker,
                        0.0,
                        20.0,
                        200.0,
                        TOP_SCREEN_WIDTH,
                        20.0,
                    );
                    loop {
                        process(self.apt, self.gfx, self.hid)?;
                        self.begin_paint();
                        self.paint_remapping(doodle_pal_pid, friends_picker.pid_name_texts);
                        view.render(self.gui);
                        self.end_paint();

                        if self.hid.keys_down().contains(KeyPad::DPAD_DOWN) {
                            view.down();
                        } else if self.hid.keys_down().contains(KeyPad::DPAD_UP) {
                            view.up();
                        } else if self.hid.keys_down().contains(KeyPad::A) {
                            let new_pid = *read
                                .friends
                                .iter()
                                .enumerate()
                                .find(|i| i.0 == view.current())
                                .unwrap()
                                .1
                                .0;
                            mapping.borrow_mut().insert(doodle_pal_pid, new_pid);
                            break;
                        } else if self.hid.keys_down().contains(KeyPad::X) {
                            mapping.borrow_mut().remove(&doodle_pal_pid);
                            break;
                        } else if self.hid.keys_down().contains(KeyPad::B) {
                            break;
                        }
                    }
                }
            } else if self.hid.keys_down().contains(KeyPad::Y) {
                return MigrationFlow::Continue(());
            }
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

    pub fn paint_mapping(&self) {
        self.gui
            .text(&self.section_doodles, 10.0, 225.0, C2D_AlignLeft, 0.5);
        self.gui.text(
            &self.section_friends,
            TOP_SCREEN_WIDTH - 10.0,
            225.0,
            C2D_AlignRight,
            0.5,
        );

        self.gui.rect(
            0.0,
            20.0,
            TOP_SCREEN_WIDTH / 2.0,
            TOP_SCREEN_HEIGHT - 20.0,
            self.gui.side_swapdoodle,
        );

        self.gui.rect(
            TOP_SCREEN_WIDTH / 2.0,
            20.0,
            TOP_SCREEN_WIDTH / 2.0,
            TOP_SCREEN_HEIGHT - 20.0,
            self.gui.side_friends,
        );

        self.gui.rect(
            TOP_SCREEN_WIDTH / 2.0 - 40.0,
            TOP_SCREEN_HEIGHT - 15.0,
            40.0 * 2.0,
            15.0,
            self.gui.bg,
        );

        self.gui.text(
            &self.press_y_done,
            TOP_SCREEN_WIDTH / 2.0,
            225.0,
            C2D_AlignCenter,
            0.5,
        );
    }

    pub fn paint_remapping(&self, pid: u32, names: &HashMap<u32, C2D_Text>) {
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
        self.gui
            .text(names.get(&pid).unwrap(), 110.0, 15.0, C2D_AtBaseline, 0.7);
    }
}

struct MappingPicker<'a> {
    mapping: &'a RefCell<OldToNewPIDMapping>,
    doodles: &'a MiiMap,
    pid_name_texts: &'a HashMap<u32, C2D_Text>,
    text_dont_map: &'a C2D_Text,
}

impl<'a> MappingPicker<'a> {
    fn new(
        mapping: &'a RefCell<OldToNewPIDMapping>,
        doodles: &'a MiiMap,
        pid_name_texts: &'a HashMap<u32, C2D_Text>,
        text_dont_map: &'a C2D_Text,
    ) -> Self {
        Self {
            mapping,
            doodles,
            pid_name_texts,
            text_dont_map,
        }
    }
}

impl ScrollableViewData for MappingPicker<'_> {
    fn render_line(&self, gui: &Gui, index: usize, x: f32, y: f32, width: f32, _height: f32) {
        let current_pid = self
            .doodles
            .iter()
            .enumerate()
            .find(|i| i.0 == index)
            .unwrap()
            .1
            .0;
        let mapping_value = self.mapping.borrow();
        let mapped_to = mapping_value.get(current_pid);
        gui.text(
            &self.pid_name_texts[current_pid],
            x + 15.0,
            y,
            C2D_AlignLeft,
            0.6,
        );
        gui.text(
            match mapped_to {
                Some(key) => &self.pid_name_texts[key],
                None => self.text_dont_map,
            },
            width - 10.0,
            y,
            C2D_AlignRight,
            0.6,
        );
    }

    fn count_items(&self) -> usize {
        self.doodles.len()
    }
}

struct FriendPicker<'a> {
    friends: &'a MiiMap,
    pid_name_texts: &'a HashMap<u32, C2D_Text>,
}

impl<'a> FriendPicker<'a> {
    fn new(friends: &'a MiiMap, pid_name_texts: &'a HashMap<u32, C2D_Text>) -> Self {
        Self {
            friends,
            pid_name_texts,
        }
    }
}

impl ScrollableViewData for FriendPicker<'_> {
    fn render_line(&self, gui: &Gui, index: usize, x: f32, y: f32, _width: f32, _height: f32) {
        let current = self
            .friends
            .iter()
            .enumerate()
            .find(|i| i.0 == index)
            .unwrap()
            .1;
        gui.text(
            &self.pid_name_texts[current.0],
            x + 15.0,
            y,
            C2D_AlignLeft,
            0.6,
        );
    }

    fn count_items(&self) -> usize {
        self.friends.len()
    }
}
