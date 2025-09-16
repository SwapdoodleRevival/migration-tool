use std::{cmp::min, ffi::CStr, mem, ops::Div, ptr::null};

use citro2d_sys::{
    C2D_AlignCenter, C2D_AtBaseline, C2D_Color32, C2D_CreateScreenTarget, C2D_DEFAULT_MAX_OBJECTS,
    C2D_DrawRectSolid, C2D_DrawText, C2D_Init, C2D_Prepare, C2D_SceneBegin, C2D_TargetClear,
    C2D_Text, C2D_TextBuf, C2D_TextBufDelete, C2D_TextBufNew, C2D_TextBufResize, C2D_TextOptimize,
    C2D_TextParse, C2D_WithColor, C3D_RenderTarget,
};
use citro3d_sys::{
    C3D_DEFAULT_CMDBUF_SIZE, C3D_FRAME_SYNCDRAW, C3D_FrameBegin, C3D_FrameEnd, C3D_Init,
};
use ctru::{GFX_LEFT, GFX_TOP};
use ctru_sys as ctru;

pub struct Gui {
    pub screen: *mut C3D_RenderTarget,
    pub textbuf: TextBufferManager,
    pub bg: u32,
    pub fg: u32,
    pub red: u32,
    pub blue: u32,
    pub dialog_overlay: u32,
    pub highlight_color: u32,
    pub dark_purple: u32,
    pub dark_green: u32,
}

pub const TOP_SCREEN_WIDTH: f32 = 400.0;
pub const TOP_SCREEN_HEIGHT: f32 = 240.0;

impl Gui {
    pub fn init() -> Self {
        unsafe {
            C3D_Init(C3D_DEFAULT_CMDBUF_SIZE as usize);
            C2D_Init(C2D_DEFAULT_MAX_OBJECTS as usize);

            let screen = C2D_CreateScreenTarget(GFX_TOP, GFX_LEFT);

            C2D_Prepare();
            Self {
                screen,
                textbuf: TextBufferManager::init(512),
                fg: C2D_Color32(255, 255, 255, 255),
                red: C2D_Color32(255, 0, 0, 255),
                blue: C2D_Color32(0, 40, 199, 255),
                dialog_overlay: C2D_Color32(0, 0, 0, 120),
                dark_purple: C2D_Color32(31, 16, 42, 255),
                dark_green: C2D_Color32(16, 42, 16, 255),
                highlight_color: C2D_Color32(255, 255, 255, 90),
                bg: C2D_Color32(20, 20, 20, 255),
            }
        }
    }

    pub fn begin_frame(&self) {
        unsafe {
            C3D_FrameBegin(C3D_FRAME_SYNCDRAW);
            C2D_TargetClear(self.screen, self.bg);
            C2D_SceneBegin(self.screen);
        }
    }

    pub fn header(&self, text: &C2D_Text) {
        self.blue_rect(0.0, 0.0, TOP_SCREEN_WIDTH, 30.0);
        self.text(
            text,
            TOP_SCREEN_WIDTH / 2.0,
            22.0,
            C2D_AlignCenter | C2D_AtBaseline,
            0.7,
        );
    }

    pub fn header_small(&self, text: &C2D_Text) {
        self.blue_rect(0.0, 0.0, TOP_SCREEN_WIDTH, 20.0);
        self.text(
            text,
            TOP_SCREEN_WIDTH / 2.0,
            15.0,
            C2D_AlignCenter | C2D_AtBaseline,
            0.7,
        );
    }

    pub fn blue_rect(&self, x: f32, y: f32, width: f32, height: f32) {
        self.rect(x, y, width, height, self.blue);
    }

    pub fn rect(&self, x: f32, y: f32, width: f32, height: f32, color: u32) {
        unsafe {
            C2D_DrawRectSolid(x, y, 0.0, width, height, color);
        }
    }

    pub fn dialog(&self) {
        unsafe {
            C2D_DrawRectSolid(
                0.0,
                0.0,
                0.0,
                TOP_SCREEN_WIDTH,
                TOP_SCREEN_HEIGHT,
                self.dialog_overlay,
            );
        }
    }

    pub fn highlight(&self, x: f32, y: f32, width: f32, height: f32) {
        unsafe {
            C2D_DrawRectSolid(x, y, 0.0, width, height, self.highlight_color);
        }
    }

    pub fn text(&self, text: &C2D_Text, x: f32, y: f32, flags: u8, scale: f32) {
        self._draw_text(text, x, y, flags | C2D_WithColor, self.fg, scale);
    }

    pub fn text_danger(&self, text: &C2D_Text, x: f32, y: f32, flags: u8, scale: f32) {
        self._draw_text(text, x, y, flags | C2D_WithColor, self.red, scale);
    }

    fn _draw_text(&self, text: &C2D_Text, x: f32, y: f32, flags: u8, color: u32, scale: f32) {
        unsafe {
            C2D_DrawText(
                text as *const _,
                flags as u32,
                x,
                y,
                0.0,
                scale,
                scale,
                color,
            );
        }
    }

    pub fn end_frame(&self) {
        unsafe {
            C3D_FrameEnd(0);
        }
    }
}

pub struct TextBufferManager {
    buf: C2D_TextBuf,
    glyph_count: usize,
}

impl TextBufferManager {
    pub fn init(size: usize) -> Self {
        unsafe {
            Self {
                buf: C2D_TextBufNew(size),
                glyph_count: size,
            }
        }
    }

    pub fn make_static_text(&mut self, content: &CStr) -> C2D_Text {
        unsafe {
            let mut text: C2D_Text = mem::zeroed();
            let end_of_parsing = C2D_TextParse(&mut text as *mut _, self.buf, content.as_ptr());
            if *end_of_parsing != b'\0' {
                self.glyph_count *= 2;
                self.buf = C2D_TextBufResize(self.buf, self.glyph_count);
                C2D_TextParse(&mut text as *mut _, self.buf, content.as_ptr());
            }
            C2D_TextOptimize(&mut text as *mut _);
            text
        }
    }
}

impl Drop for TextBufferManager {
    fn drop(&mut self) {
        unsafe {
            C2D_TextBufDelete(self.buf);
        }
    }
}

pub struct ScrollableView<'a, T: ScrollableViewData> {
    data: &'a T,
    highlighted_item: usize,
    x: f32,
    y: f32,
    height: f32,
    width: f32,
    item_height: f32,
}

impl<'a, T: ScrollableViewData> ScrollableView<'a, T> {
    pub fn new(data: &'a T, x: f32, y: f32, height: f32, width: f32, item_height: f32) -> Self {
        Self {
            data,
            highlighted_item: 0,
            x,
            y,
            height,
            width,
            item_height,
        }
    }

    fn max_items_on_screen(&self) -> usize {
        self.height.div(self.item_height).floor() as usize
    }

    pub fn up(&mut self) {
        if self.highlighted_item == 0 {
            self.highlighted_item = self.data.count_items();
        }
        self.highlighted_item = self.highlighted_item.saturating_sub(1);
    }

    pub fn down(&mut self) {
        self.highlighted_item = self.highlighted_item.saturating_add(1);
        if self.highlighted_item == self.data.count_items() {
            self.highlighted_item = 0;
        }
    }

    pub fn current(&self) -> usize {
        self.highlighted_item
    }

    pub fn render(&self, gui: &Gui) {
        let max_items = self.max_items_on_screen();
        let page_start = (self.highlighted_item / max_items) * max_items;
        let mut y = self.y;
        for index in page_start..page_start + min(self.data.count_items() - page_start, max_items) {
            if index == self.highlighted_item {
                gui.rect(self.x, y, self.width, self.item_height, gui.highlight_color);
            }
            self.data
                .render_line(gui, index, self.x, y, self.width, self.item_height);
            y += self.item_height;
        }
    }
}

pub trait ScrollableViewData {
    fn render_line(&self, gui: &Gui, index: usize, x: f32, y: f32, width: f32, height: f32);
    fn count_items(&self) -> usize;
}
