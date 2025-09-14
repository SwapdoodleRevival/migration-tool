use std::io::Write;

use std::io::Cursor;

use citro2d_sys::{C2D_AlignCenter, C2D_Text};
use ctru::prelude::KeyPad;
use libdoodle::{
    blocks::common1,
    bpk1::{BPK1Blocks, BPK1File},
};

use crate::extdata::ExtdataArchive;
use crate::gui::{Gui, TOP_SCREEN_HEIGHT, TOP_SCREEN_WIDTH, TextBuffer};
use crate::phases::OldToNewPIDMapping;
use crate::{Services, read::ReadExt};

pub fn rewrite(
    s: &mut Services,
    extdata: ExtdataArchive,
    mapping: OldToNewPIDMapping,
) -> Result<(), ()> {
    let scene = Scene::make(s.gui);

    loop {
        s.process()?;
        scene.paint_ready_page();

        if s.hid.keys_down().contains(KeyPad::A) {
            break;
        }
    }

    s.console.clear();
    scene.paint_progess_page();
    do_rewrite(extdata, mapping);

    loop {
        s.process()?;
        scene.paint_done_page();

        if s.hid.keys_down().contains(KeyPad::A) {
            break;
        }
    }

    Ok(())
}

fn do_rewrite(extdata: ExtdataArchive, mapping: OldToNewPIDMapping) {
    println!("Reading manage.bin...");
    let mut manage = BPK1Blocks::new_from_bpk1_bytes(&extdata.read_manage()).unwrap();
    let cominf = manage
        .iter_mut()
        .find(|k| k.name.as_bytes() == b"COMINF0")
        .expect("manage.bin should have a COMINF0, but it doesn't!");

    let mut cursor = Cursor::new(&mut cominf.data);

    let count = cursor.read_u32_le().unwrap();
    cursor.set_position(0x40);
    for _ in 0..count {
        let pos = cursor.position();
        let sender_pid =
            common1::CommonInfo::from_bytes(&(cursor.read_const_num_of_bytes::<0x40>().unwrap()))
                .unwrap()
                .sender_pid;

        if let Some(new_pid) = mapping.get(&sender_pid) {
            let letter_key = cursor.read_u32_le().unwrap();
            cursor.set_position(pos + 24);
            cursor.write_all(&u32::to_le_bytes(*new_pid)).unwrap();

            let mut letter =
                BPK1Blocks::new_from_bpk1_bytes(&extdata.read_letter_index(letter_key)).unwrap();
            let common_block = match letter.iter_mut().find(|k| k.name.as_bytes() == b"COMMON1") {
                Some(k) => k,
                None => continue,
            };

            common_block.data[24..28].copy_from_slice(&u32::to_le_bytes(*new_pid));

            let out = BPK1Blocks::bytes_from_bpk1_blocks(letter).unwrap();

            extdata.write_letter_index(letter_key, &out);
        }

        cursor.set_position(pos + 0x80);
    }

    println!("Rewriting manage.bin...");
    extdata.write_file(
        "/letter/manage.bin",
        &(BPK1Blocks::bytes_from_bpk1_blocks(manage).unwrap()),
    );
    println!("Rewrote manage.bin.");
}

struct Scene<'a> {
    gui: &'a Gui,
    textbuf: TextBuffer,
    header_text: C2D_Text,
    action_text: C2D_Text,
    nobkp_line1_text: C2D_Text,
    nobkp_line2_text: C2D_Text,
    begin: C2D_Text,
    no_exit: C2D_Text,
    progress: C2D_Text,
    progress_observe_bottom: C2D_Text,
    header_text_finished: C2D_Text,
    finished_line: C2D_Text,
    exit: C2D_Text,
    exit_a: C2D_Text,
}

impl<'a> Scene<'a> {
    pub fn make(gui: &'a Gui) -> Self {
        let textbuf = TextBuffer::init(4096);

        Scene {
            gui,
            header_text: textbuf.make_static_text(c"Ready to migrate"),
            action_text: textbuf
                .make_static_text(c"We can now start migrating your Swapdoodle notes."),
            nobkp_line1_text: textbuf.make_static_text(
                c"Reminder: This tool does not back up your extra data before migrating!",
            ),
            nobkp_line2_text: textbuf
                .make_static_text(c"If you do not have a backup, DO NOT CONTINUE!!!"),
            begin: textbuf.make_static_text(c"Press \u{E000} to begin"),
            exit: textbuf.make_static_text(c"Press Start to exit"),
            exit_a: textbuf.make_static_text(c"Press \u{E000} to exit"),
            no_exit: textbuf
                .make_static_text(c"You cannot interrupt the migration once it has begun."),
            progress: textbuf.make_static_text(c"Migrating in progress..."),
            progress_observe_bottom: textbuf.make_static_text(c"Look at the bottom screen."),
            header_text_finished: textbuf.make_static_text(c"Finished!"),
            finished_line: textbuf.make_static_text(c"Your notes have been migrated."),
            textbuf,
        }
    }

    pub fn paint_ready_page(&self) {
        self.gui.begin_frame();
        self.gui.header(&self.header_text);
        self.gui.text(&self.action_text, 10.0, 40.0, 0, 0.5);

        self.gui
            .text_danger(&self.nobkp_line1_text, 10.0, 60.0, 0, 0.4);
        self.gui
            .text_danger(&self.nobkp_line2_text, 10.0, 70.0, 0, 0.4);

        self.gui.text(
            &self.begin,
            TOP_SCREEN_WIDTH / 2.0,
            100.0,
            C2D_AlignCenter,
            0.7,
        );
        self.gui.text(
            &self.exit,
            TOP_SCREEN_WIDTH / 2.0,
            130.0,
            C2D_AlignCenter,
            0.7,
        );
        self.gui.text(
            &self.no_exit,
            TOP_SCREEN_WIDTH / 2.0,
            155.0,
            C2D_AlignCenter,
            0.5,
        );

        self.gui.end_frame();
    }

    pub fn paint_progess_page(&self) {
        self.gui.begin_frame();
        self.gui.text(
            &self.progress,
            TOP_SCREEN_WIDTH / 2.0,
            TOP_SCREEN_HEIGHT / 2.0 - 40.0,
            C2D_AlignCenter,
            0.7,
        );
        self.gui.text(
            &self.progress_observe_bottom,
            TOP_SCREEN_WIDTH / 2.0,
            TOP_SCREEN_HEIGHT / 2.0 + 40.0,
            C2D_AlignCenter,
            0.5,
        );
        self.gui.end_frame();
    }

    fn paint_done_page(&self) {
        self.gui.begin_frame();
        self.gui.header(&self.header_text_finished);

        self.gui.text(&self.finished_line, 10.0, 40.0, 0, 0.5);

        self.gui.text(
            &self.exit_a,
            TOP_SCREEN_WIDTH / 2.0,
            100.0,
            C2D_AlignCenter,
            0.7,
        );

        self.gui.end_frame();
    }
}
