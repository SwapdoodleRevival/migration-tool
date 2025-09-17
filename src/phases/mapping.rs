use std::{cell::RefCell, collections::HashMap, ffi::CString, rc::Rc, str::FromStr};

use citro2d_sys::{C2D_AlignCenter, C2D_AlignLeft, C2D_AlignRight, C2D_AtBaseline, C2D_Text};
use ctru::prelude::{Apt, Gfx, Hid, KeyPad};

use crate::{
    Services,
    friend_list::MiiMap,
    gui::{
        Gui, ScrollableView, ScrollableViewData, TOP_SCREEN_HEIGHT, TOP_SCREEN_WIDTH,
        TextBufferManager,
    },
    phases::ReadResult,
};

//                                .- PID of note sender
//                                v      .- PID of friend
pub type OldToNewPIDMapping = HashMap<u32, u32>;

pub fn mapping(s: &mut Services, read: ReadResult) -> Result<OldToNewPIDMapping, ()> {
    let mapping = RefCell::new(OldToNewPIDMapping::new());
    auto_match_by_mac(&mut mapping.borrow_mut(), &read);

    let mut names = HashMap::<u32, C2D_Text>::new();
    for (pid, mii) in read.doodles.iter() {
        names.insert(
            *pid,
            s.gui
                .textbuf
                .make_static_text(&CString::from_str(&mii.mii_name).unwrap_or_default()),
        );
    }
    for (pid, mii) in read.friends.iter() {
        names.insert(
            *pid,
            s.gui
                .textbuf
                .make_static_text(&CString::from_str(&mii.mii_name).unwrap_or_default()),
        );
    }

    let txt_dont_map = s.gui.textbuf.make_static_text(c"<don't map>");

    let mut scene = Scene {
        header_text: s.gui.textbuf.make_static_text(c"Mapping"),
        explanation_line1: s
            .gui
            .textbuf
            .make_static_text(c"The migration tool has attempted to automatically"),
        explanation_line2: s
            .gui
            .textbuf
            .make_static_text(c"match unknown Doodle authors to your friends."),
        explanation_line3: s
            .gui
            .textbuf
            .make_static_text(c"You can change the suggested mapping"),
        explanation_line4: s
            .gui
            .textbuf
            .make_static_text(c"before you begin the migration."),
        explanation_line5: s
            .gui
            .textbuf
            .make_static_text(c"Highlight a mapping with \u{E07D} "),
        explanation_line6: s
            .gui
            .textbuf
            .make_static_text(c"and press \u{E000} to change it."),
        press_a_continue: s
            .gui
            .textbuf
            .make_static_text(c"Press \u{E000} to continue"),
        press_b_back: s.gui.textbuf.make_static_text(c"\u{E001} Back"),
        press_x_clear: s.gui.textbuf.make_static_text(c"\u{E002} Clear"),
        press_y_done: s.gui.textbuf.make_static_text(c"\u{E003} Finish"),
        remapping: s.gui.textbuf.make_static_text(c"Remapping"),
        section_doodles: s.gui.textbuf.make_static_text(c"Doodle pals"),
        section_friends: s.gui.textbuf.make_static_text(c"3DS Friends"),
        gui: s.gui,
    };

    loop {
        Services::process(s.apt, s.gfx, s.hid)?;
        scene.begin_paint();
        scene.dialog_explanation();
        scene.end_paint();

        if s.hid.keys_down().contains(KeyPad::A) {
            break;
        }
    }

    pick_mapping(
        s.apt,
        s.gfx,
        s.hid,
        &mut scene,
        &names,
        txt_dont_map,
        &read,
        &mapping,
    )?;

    Ok(mapping.into_inner())
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
    apt: &Apt,
    gfx: &Gfx,
    hid: &mut Hid,
    scene: &mut Scene,
    names: &HashMap<u32, C2D_Text>,
    txt_dont_map: C2D_Text,
    read: &ReadResult,
    mapping: &RefCell<OldToNewPIDMapping>,
) -> Result<(), ()> {
    let mapping_picker = MappingPicker::new(mapping, &read.doodles, names, &txt_dont_map);
    let friends_picker = FriendPicker::new(&read.friends, &names);

    let mut view = ScrollableView::new(&mapping_picker, 0.0, 20.0, 200.0, TOP_SCREEN_WIDTH, 20.0);

    loop {
        Services::process(apt, gfx, hid)?;
        scene.begin_paint();
        scene.paint_mapping();
        view.render(scene.gui);
        scene.end_paint();

        if hid.keys_down().contains(KeyPad::DPAD_DOWN) {
            view.down();
        } else if hid.keys_down().contains(KeyPad::DPAD_UP) {
            view.up();
        } else if hid.keys_down().contains(KeyPad::A) {
            let pid = *read
                .doodles
                .iter()
                .enumerate()
                .find(|i| i.0 == view.current())
                .unwrap()
                .1
                .0;
            pick_friend(apt, gfx, hid, scene, &friends_picker, pid, read, mapping)?;
        } else if hid.keys_down().contains(KeyPad::Y) {
            return Ok(());
        }
    }
}

fn pick_friend(
    apt: &Apt,
    gfx: &Gfx,
    hid: &mut Hid,
    scene: &mut Scene,
    picker: &FriendPicker,
    pid: u32,
    read: &ReadResult,
    mapping: &RefCell<OldToNewPIDMapping>,
) -> Result<(), ()> {
    let mut view = ScrollableView::new(picker, 0.0, 20.0, 200.0, TOP_SCREEN_WIDTH, 20.0);

    loop {
        Services::process(apt, gfx, hid)?;
        scene.begin_paint();
        scene.paint_remapping(pid, picker.pid_name_texts);
        view.render(scene.gui);
        scene.end_paint();

        if hid.keys_down().contains(KeyPad::DPAD_DOWN) {
            view.down();
        } else if hid.keys_down().contains(KeyPad::DPAD_UP) {
            view.up();
        } else if hid.keys_down().contains(KeyPad::A) {
            let new_pid = *read
                .friends
                .iter()
                .enumerate()
                .find(|i| i.0 == view.current())
                .unwrap()
                .1
                .0;
            mapping.borrow_mut().insert(pid, new_pid);
            return Ok(());
        } else if hid.keys_down().contains(KeyPad::X) {
            mapping.borrow_mut().remove(&pid);
            return Ok(());
        } else if hid.keys_down().contains(KeyPad::B) {
            return Ok(());
        }
    }
}

struct Scene<'a> {
    gui: &'a Gui,
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
    remapping: C2D_Text,
}

impl<'a> Scene<'a> {
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
    fn render_line(&self, gui: &Gui, index: usize, x: f32, y: f32, width: f32, height: f32) {
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
                None => &self.text_dont_map,
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
    fn render_line(&self, gui: &Gui, index: usize, x: f32, y: f32, width: f32, height: f32) {
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
