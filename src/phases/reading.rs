use std::{
    collections::HashMap,
    io::{self, Cursor, Write},
};

use citro2d_sys::{C2D_AlignCenter, C2D_AlignLeft, C2D_Text};
use ctru::prelude::KeyPad;
use libdoodle::{
    blocks::{common1, miistd1::MiiData},
    bpk1::{BPK1Blocks, BPK1File},
    files::letter::Letter,
};

use crate::{
    Services,
    extdata::{ExtdataArchive, SwapdoodleRegion},
    friend_list::{self, MiiMap},
    gui::{Gui, ScrollableView, ScrollableViewData, TOP_SCREEN_WIDTH, TextBuffer},
    read::ReadExt,
};

pub struct ReadResult {
    pub friends: MiiMap,
    pub doodles: MiiMap,
}

pub fn reading(s: &mut Services) -> Result<(ExtdataArchive, ReadResult), ()> {
    s.console.clear();
    let mut scene = Scene::make(s.gui);

    let mut picker = ExtdataPicker::new();

    if picker.none_available() {
        loop {
            s.process()?;
            scene.begin_paint();
            scene.paint_no_extdata();
            scene.end_paint();

            if s.hid.keys_down().contains(KeyPad::A) {
                return Err(());
            }
        }
    }

    let extdata;

    if picker.single_available() {
        extdata = picker.available_archives.pop().unwrap();

        loop {
            s.process()?;
            scene.begin_paint();
            scene.paint_single_extdata(&extdata.region);
            scene.end_paint();

            if s.hid.keys_down().contains(KeyPad::A) {
                break;
            }
        }
    } else {
        let mut view = ScrollableView::new(&picker, 0.0, 100.0, 120.0, TOP_SCREEN_WIDTH, 20.0);

        loop {
            scene.begin_paint();
            scene.paint_several_extdata();
            view.render(s.gui);
            scene.end_paint();

            s.process()?;

            if s.hid.keys_down().contains(KeyPad::DPAD_UP) {
                view.up();
            } else if s.hid.keys_down().contains(KeyPad::DPAD_DOWN) {
                view.down();
            }

            if s.hid.keys_down().contains(KeyPad::A) {
                extdata = picker.available_archives.swap_remove(view.current());
                break;
            }
        }
    }

    let (friends, doodles) = friendly_read_data(&extdata);

    if friends.len() == 1 {
        println!("Your friend list is empty.");
        println!();
        println!("Swapdoodle notes are tied to friend data.");
        println!("I hope this doesn't sound rude, but here goes:");
        println!("If you don't have friends, there is not much we can do.");
        println!();
        println!("Feel free to re-run this tool later!");
        println!();
        println!("If you believe this is in error, please let us know!");
        println!();
        println!("Press (A) to exit.");

        loop {
            s.process()?;

            if s.hid.keys_down().contains(KeyPad::A) {
                return Err(());
            }
        }
    }

    if doodles.is_empty() {
        println!("We didn't find any notes from an unknown sender.");
        println!("You shouldn't need to run this tool.");
        println!();
        println!("If you believe this is in error, please let us know!");
        println!();
        println!("Press (A) to exit.");

        loop {
            s.process()?;

            if s.hid.keys_down().contains(KeyPad::A) {
                return Err(());
            }
        }
    }

    Ok((extdata, ReadResult { friends, doodles }))
}

fn friendly_read_data(extdata: &ExtdataArchive) -> (MiiMap, MiiMap) {
    print!("Reading your friend list... ");
    _ = io::stdout().flush();

    let friends = friend_list::load_friend_list();
    println!("done!");

    println!("Reading your Swapdoodle extdata... ");

    println!("Reading file /letter/manage.bin...");
    let mut manage = BPK1Blocks::new_from_bpk1_bytes(&extdata.read_manage()).unwrap();
    let cominf = manage
        .iter_mut()
        .find(|k| k.name.as_bytes() == b"COMINF0")
        .expect("File /letter/manage.bin should have a COMINF0, but it doesn't!");

    let mut cursor = Cursor::new(&cominf.data);

    let mut unknown_pids = HashMap::<u32, u32>::new();

    let count = cursor.read_u32_le().unwrap();
    cursor.set_position(0x40);

    for _ in 0..count {
        let pos = cursor.position();
        let common =
            common1::CommonInfo::from_bytes(&(cursor.read_const_num_of_bytes::<0x40>().unwrap()))
                .unwrap();
        let sender_pid = common.sender_pid;

        if friends.get(&sender_pid).is_none() {
            let letter_key = cursor.read_u32_le().unwrap();
            unknown_pids.insert(sender_pid, letter_key);
        }

        cursor.set_position(pos + 0x80);
    }
    println!("done.");

    let mut doodles = HashMap::<u32, MiiData>::new();

    unknown_pids.iter().for_each(|row| {
        let key = *row.1;
        if let Some(mii) = Letter::new_from_bpk1_bytes(&extdata.read_letter_index(key))
            .unwrap()
            .sender_mii
        {
            doodles.insert(*row.0, mii);
        }
    });

    println!("All done!");

    (friends, doodles)
}

struct Scene<'a> {
    gui: &'a Gui,
    textbuf: TextBuffer,
    header_text: C2D_Text,
    no_extdata: C2D_Text,
    error_lmk: C2D_Text,
    detected_one_reg: C2D_Text,
    detected_more_reg: C2D_Text,
    detected_more_reg_line1: C2D_Text,
    detected_more_reg_line2: C2D_Text,
    reg_eu: C2D_Text,
    btn_eu: C2D_Text,
    reg_us: C2D_Text,
    btn_us: C2D_Text,
    reg_jp: C2D_Text,
    btn_jp: C2D_Text,
    exit: C2D_Text,
    begin_reading: C2D_Text,
}

impl<'a> Scene<'a> {
    pub fn make(gui: &'a Gui) -> Self {
        let textbuf = TextBuffer::init(4096);

        Scene {
            gui,
            header_text: textbuf.make_static_text(c"Confirm save data"),
            no_extdata: textbuf
                .make_static_text(c"It appears you do not have any Swapdoodle extdata."),
            error_lmk: textbuf
                .make_static_text(c"If you believe this is in error, please let us know!"),
            exit: textbuf.make_static_text(c"Press \u{E000} to exit"),
            begin_reading: textbuf.make_static_text(c"Press \u{E000} to begin reading."),
            detected_one_reg: textbuf.make_static_text(c"Detected region:"),
            detected_more_reg: textbuf.make_static_text(c"Detected several regions."),
            detected_more_reg_line1: textbuf
                .make_static_text(c"This tool can only work with one at a time."),
            detected_more_reg_line2: textbuf.make_static_text(c"Please select a region:"),
            reg_eu: textbuf.make_static_text(c"Europe"),
            btn_eu: textbuf.make_static_text(c"\u{E002}"),
            reg_us: textbuf.make_static_text(c"USA"),
            btn_us: textbuf.make_static_text(c"\u{E003}"),
            reg_jp: textbuf.make_static_text(c"Japan"),
            btn_jp: textbuf.make_static_text(c"\u{E001}"),
            textbuf,
        }
    }

    pub fn begin_paint(&self) {
        self.gui.begin_frame();
        self.gui.header(&self.header_text);
    }

    pub fn paint_no_extdata(&self) {
        self.gui.text(&self.no_extdata, 10.0, 40.0, 0, 0.5);
        self.gui.text(&self.error_lmk, 10.0, 55.0, 0, 0.5);
        self.gui.text(
            &self.exit,
            TOP_SCREEN_WIDTH / 2.0,
            80.0,
            C2D_AlignCenter,
            0.7,
        );
    }

    pub fn paint_single_extdata(&self, region: &SwapdoodleRegion) {
        self.gui.text(&self.detected_one_reg, 10.0, 40.0, 0, 0.5);
        self.gui.text(
            match region {
                SwapdoodleRegion::EU => &self.reg_eu,
                SwapdoodleRegion::US => &self.reg_us,
                SwapdoodleRegion::JP => &self.reg_jp,
            },
            10.0,
            60.0,
            0,
            0.7,
        );

        self.gui.text(
            &self.begin_reading,
            TOP_SCREEN_WIDTH / 2.0,
            90.0,
            C2D_AlignCenter,
            0.7,
        );
    }

    pub fn end_paint(&self) {
        self.gui.end_frame();
    }

    fn paint_several_extdata(&self) {
        self.gui.text(&self.detected_more_reg, 10.0, 40.0, 0, 0.5);

        self.gui
            .text(&self.detected_more_reg_line1, 10.0, 55.0, 0, 0.5);

        self.gui
            .text(&self.detected_more_reg_line2, 10.0, 70.0, 0, 0.5);
    }
}

struct ExtdataPicker {
    textbuf: TextBuffer,
    reg_eu: C2D_Text,
    reg_us: C2D_Text,
    reg_jp: C2D_Text,

    available_archives: Vec<ExtdataArchive>,
}

impl ExtdataPicker {
    fn new() -> Self {
        let mut available_archives = vec![];
        if let Ok(archive) = ExtdataArchive::open(SwapdoodleRegion::EU) {
            available_archives.push(archive);
        }
        if let Ok(archive) = ExtdataArchive::open(SwapdoodleRegion::US) {
            available_archives.push(archive);
        }
        if let Ok(archive) = ExtdataArchive::open(SwapdoodleRegion::JP) {
            available_archives.push(archive);
        }
        let textbuf = TextBuffer::init(64);
        Self {
            available_archives,
            reg_eu: textbuf.make_static_text(c"Europe"),
            reg_us: textbuf.make_static_text(c"USA"),
            reg_jp: textbuf.make_static_text(c"Japan"),
            textbuf,
        }
    }

    fn none_available(&self) -> bool {
        self.available_archives.len() == 0
    }

    fn single_available(&self) -> bool {
        self.available_archives.len() == 1
    }
}

impl ScrollableViewData for ExtdataPicker {
    fn render_line(&self, gui: &Gui, index: usize, x: f32, y: f32, width: f32, height: f32) {
        let &text = match self.available_archives[index].region {
            SwapdoodleRegion::EU => &self.reg_eu,
            SwapdoodleRegion::US => &self.reg_us,
            SwapdoodleRegion::JP => &self.reg_jp,
        };

        gui.text(&text, x + 10.0, y, C2D_AlignLeft, 0.6);
    }

    fn count_items(&self) -> usize {
        self.available_archives.len()
    }
}
