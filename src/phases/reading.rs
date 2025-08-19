use std::{
    collections::HashMap,
    io::{self, Cursor, Write},
};

use citro2d_sys::{
    C2D_AlignCenter, C2D_AtBaseline, C2D_Color32, C2D_DrawRectSolid, C2D_DrawText, C2D_SceneBegin,
    C2D_TargetClear, C2D_Text, C2D_WithColor,
};
use citro3d_sys::{C3D_FRAME_SYNCDRAW, C3D_FrameBegin, C3D_FrameEnd};
use ctru::{prelude::KeyPad, services::cfgu::Region};
use libdoodle::{
    blocks::{common1, miistd1::MiiData},
    bpk1::{BPK1Blocks, BPK1File},
    files::letter::Letter,
};

use crate::{
    AppData, Services,
    extdata::{self, ExtdataArchive, SwapdoodleRegion},
    friend_list::{self, MiiMap},
    gui::{GUI, TOP_SCREEN_WIDTH, TextBuffer},
    read::ReadExt,
};

pub fn reading(s: &mut Services, data: &mut AppData) -> Result<ExtdataArchive, ()> {
    s.console.clear();
    let scene = Scene::make(s.gui);
    scene.begin_paint();

    let extdatas = (
        ExtdataArchive::open(SwapdoodleRegion::EU),
        ExtdataArchive::open(SwapdoodleRegion::US),
        ExtdataArchive::open(SwapdoodleRegion::JP),
    );

    let available: u8 = if extdatas.0.is_ok() { 1 } else { 0 }
        + if extdatas.1.is_ok().into() { 1 } else { 0 }
        + if extdatas.2.is_ok().into() { 1 } else { 0 };

    if available == 0 {
        scene.paint_no_extdata();
        scene.end_paint();

        loop {
            s.process()?;

            if s.hid.keys_down().contains(KeyPad::A) {
                return Err(());
            }
        }
    }

    let extdata;

    if available == 1 {
        extdata = extdatas
            .0
            .unwrap_or_else(|_| extdatas.1.unwrap_or_else(|_| extdatas.2.unwrap()));

        scene.paint_single_extdata(&extdata.region);
        scene.end_paint();

        loop {
            s.process()?;

            if s.hid.keys_down().contains(KeyPad::A) {
                break;
            }
        }
    } else {
        scene.paint_several_extdata(&extdatas);
        scene.end_paint();

        loop {
            s.process()?;

            if s.hid.keys_down().contains(KeyPad::X) {
                match extdatas.0 {
                    Ok(e) => {
                        extdata = e;
                        break;
                    }
                    Err(_) => {}
                };
            }
            if s.hid.keys_down().contains(KeyPad::Y) {
                match extdatas.1 {
                    Ok(e) => {
                        extdata = e;
                        break;
                    }
                    Err(_) => {}
                };
            }
            if s.hid.keys_down().contains(KeyPad::B) {
                match extdatas.2 {
                    Ok(e) => {
                        extdata = e;
                        break;
                    }
                    Err(_) => {}
                };
            }
        }
    }

    (data.friends, data.doodles) = friendly_read_data(&extdata);

    if data.friends.len() == 1 {
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

    if data.doodles.is_empty() {
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

    return Ok(extdata);
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

        if let None = friends.get(&sender_pid) {
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
    gui: &'a GUI,
    white: u32,
    blue: u32,
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
    pub fn make(gui: &'a GUI) -> Self {
        unsafe {
            let textbuf = TextBuffer::init(4096);

            Scene {
                gui,
                white: C2D_Color32(255, 255, 255, 255),
                blue: C2D_Color32(0, 40, 199, 255),
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
                textbuf: textbuf,
            }
        }
    }

    pub fn begin_paint(&self) {
        unsafe {
            C3D_FrameBegin(C3D_FRAME_SYNCDRAW);
            C2D_TargetClear(self.gui.screen, C2D_Color32(20, 20, 20, 255));
            C2D_SceneBegin(self.gui.screen);
            C2D_DrawRectSolid(0.0, 0.0, 0.0, TOP_SCREEN_WIDTH, 30.0, self.blue);
            TextBuffer::draw(
                &self.header_text,
                TOP_SCREEN_WIDTH / 2.0,
                22.0,
                C2D_WithColor | C2D_AlignCenter | C2D_AtBaseline,
                self.white,
                0.7,
            );
        }
    }

    pub fn paint_no_extdata(&self) {
        TextBuffer::draw(&self.no_extdata, 10.0, 40.0, C2D_WithColor, self.white, 0.5);
        TextBuffer::draw(&self.error_lmk, 10.0, 55.0, C2D_WithColor, self.white, 0.5);
        TextBuffer::draw(
            &self.exit,
            TOP_SCREEN_WIDTH / 2.0,
            80.0,
            C2D_WithColor | C2D_AlignCenter,
            self.white,
            0.7,
        );
    }

    pub fn paint_single_extdata(&self, region: &SwapdoodleRegion) {
        TextBuffer::draw(
            &self.detected_one_reg,
            10.0,
            40.0,
            C2D_WithColor,
            self.white,
            0.5,
        );
        TextBuffer::draw(
            match region {
                SwapdoodleRegion::EU => &self.reg_eu,
                SwapdoodleRegion::US => &self.reg_us,
                SwapdoodleRegion::JP => &self.reg_jp,
            },
            10.0,
            60.0,
            C2D_WithColor,
            self.white,
            0.7,
        );

        TextBuffer::draw(
            &self.begin_reading,
            TOP_SCREEN_WIDTH / 2.0,
            90.0,
            C2D_WithColor | C2D_AlignCenter,
            self.white,
            0.7,
        );
    }

    pub fn end_paint(&self) {
        unsafe {
            C3D_FrameEnd(0);
        }
    }

    fn paint_several_extdata(
        &self,
        extdatas: &(
            Result<ExtdataArchive, ()>,
            Result<ExtdataArchive, ()>,
            Result<ExtdataArchive, ()>,
        ),
    ) {
        TextBuffer::draw(
            &self.detected_more_reg,
            10.0,
            40.0,
            C2D_WithColor,
            self.white,
            0.5,
        );

        TextBuffer::draw(
            &self.detected_more_reg_line1,
            10.0,
            55.0,
            C2D_WithColor,
            self.white,
            0.5,
        );

        TextBuffer::draw(
            &self.detected_more_reg_line2,
            10.0,
            70.0,
            C2D_WithColor,
            self.white,
            0.5,
        );

        let mut y: f32 = 90.0;

        match extdatas.0 {
            Ok(_) => {
                TextBuffer::draw(&self.btn_eu, 10.0, y, C2D_WithColor, self.white, 0.7);
                TextBuffer::draw(&self.reg_eu, 30.0, y, C2D_WithColor, self.white, 0.7);

                y += 30.0;
            }
            Err(_) => {}
        };
        match extdatas.1 {
            Ok(_) => {
                TextBuffer::draw(&self.btn_us, 10.0, y, C2D_WithColor, self.white, 0.7);
                TextBuffer::draw(&self.reg_us, 30.0, y, C2D_WithColor, self.white, 0.7);

                y += 30.0;
            }
            Err(_) => {}
        };
        match extdatas.2 {
            Ok(_) => {
                TextBuffer::draw(&self.btn_jp, 10.0, y, C2D_WithColor, self.white, 0.7);
                TextBuffer::draw(&self.reg_jp, 30.0, y, C2D_WithColor, self.white, 0.7);

                y += 30.0;
            }
            Err(_) => {}
        };
    }
}
