use crate::{
    control_flow::MigrationFlow,
    extdata::{get_cominf0_cursor, ExtdataArchive, COMINF0Read},
    gui::{Gui, TOP_SCREEN_HEIGHT, TOP_SCREEN_WIDTH},
    phases::{process, OldToNewPIDMapping},
    read::ReadExt,
};
use citro2d_sys::{C2D_AlignCenter, C2D_Text};
use ctru::prelude::*;
use libdoodle::{
    blocks::common1,
    bpk1::{BPK1Blocks, BPK1File},
};
use std::io::{Cursor, Write};

pub fn rewrite<'a>(apt: &'a Apt, gfx: &'a Gfx, hid: &'a mut Hid, gui: &'a mut Gui) -> Scene<'a> {
    Scene::new(apt, gfx, hid, gui)
}

pub struct Scene<'a> {
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
    apt: &'a Apt,
    gfx: &'a Gfx,
    hid: &'a mut Hid,
    gui: &'a mut Gui,
}

impl<'a> Scene<'a> {
    fn new(apt: &'a Apt, gfx: &'a Gfx, hid: &'a mut Hid, gui: &'a mut Gui) -> Self {
        Scene {
            header_text: gui.textbuf.make_static_text(c"Ready to migrate"),
            action_text: gui
                .textbuf
                .make_static_text(c"We can now start migrating your Swapdoodle notes."),
            nobkp_line1_text: gui.textbuf.make_static_text(
                c"Reminder: This tool does not back up your extra data before migrating!",
            ),
            nobkp_line2_text: gui
                .textbuf
                .make_static_text(c"If you do not have a backup, DO NOT CONTINUE!!!"),
            begin: gui.textbuf.make_static_text(c"Press \u{E000} to begin"),
            exit: gui.textbuf.make_static_text(c"Press Start to exit"),
            exit_a: gui.textbuf.make_static_text(c"Press \u{E000} to exit"),
            no_exit: gui
                .textbuf
                .make_static_text(c"You cannot interrupt the migration once it has begun."),
            progress: gui.textbuf.make_static_text(c"Migrating in progress..."),
            progress_observe_bottom: gui.textbuf.make_static_text(c"Look at the bottom screen."),
            header_text_finished: gui.textbuf.make_static_text(c"Finished!"),
            finished_line: gui
                .textbuf
                .make_static_text(c"Your notes have been migrated."),
            apt,
            gfx,
            hid,
            gui,
        }
    }

    pub fn run(self, extdata: ExtdataArchive, mapping: OldToNewPIDMapping) -> MigrationFlow {
        loop {
            process(self.apt, self.gfx, self.hid)?;
            self.paint_ready_page();

            if self.hid.keys_down().contains(KeyPad::A) {
                break;
            }
        }

        self.paint_progess_page();
        Self::do_rewrite(extdata, mapping);

        loop {
            process(self.apt, self.gfx, self.hid)?;
            self.paint_done_page();

            if self.hid.keys_down().contains(KeyPad::A) {
                break;
            }
        }

        MigrationFlow::Continue(())
    }

    fn do_rewrite(extdata: ExtdataArchive, mapping: OldToNewPIDMapping) {
        let mut manage = BPK1Blocks::new_from_bpk1_bytes(&extdata.read_manage()).unwrap();
        let mut cursor = get_cominf0_cursor(&mut manage);

        let count = cursor.read_u32_le().unwrap();
        cursor.set_position(0x40);

        for _ in 0..count {
            let start_pos = cursor.position();
            let (common, letter_key) = cursor.read_cominf0_entry().unwrap();
            let sender_pid = common.sender_pid;
            let end_pos = cursor.position();

            if let Some(&new_pid) = mapping.get(&sender_pid) {
                cursor.set_position(start_pos + 24);
                cursor.write_all(&u32::to_le_bytes(new_pid)).unwrap();
                cursor.set_position(end_pos);

                let mut letter =
                    BPK1Blocks::new_from_bpk1_bytes(&extdata.read_letter_index(letter_key))
                        .unwrap();
                let common_block = match letter.iter_mut().find(|k| k.name == c"COMMON1") {
                    Some(k) => k,
                    None => continue,
                };

                common_block.data[24..28].copy_from_slice(&u32::to_le_bytes(new_pid));

                let out = BPK1Blocks::bytes_from_bpk1_blocks(letter).unwrap();

                extdata.write_letter_index(letter_key, &out);
            }
        }

        println!("Rewriting manage.bin...");
        extdata.write_file(
            "/letter/manage.bin",
            &(BPK1Blocks::bytes_from_bpk1_blocks(manage).unwrap()),
        );
        println!("Rewrote manage.bin.");
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
