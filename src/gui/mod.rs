pub mod colors;
pub mod scrollable_view;
pub mod text;

use citro2d_sys::{
    C2D_AlignCenter, C2D_AtBaseline, C2D_CreateScreenTarget, C2D_DEFAULT_MAX_OBJECTS,
    C2D_DrawRectSolid, C2D_DrawText, C2D_Init, C2D_Prepare, C2D_SceneBegin, C2D_TargetClear,
    C2D_Text, C2D_WithColor, C3D_RenderTarget,
};
use citro3d_sys::{
    C3D_DEFAULT_CMDBUF_SIZE, C3D_FRAME_SYNCDRAW, C3D_FrameBegin, C3D_FrameEnd, C3D_Init,
};
use ctru::{GFX_LEFT, GFX_TOP};
use ctru_sys as ctru;

use crate::gui::text::TextBufferManager;

pub struct Gui {
    pub screen: *mut C3D_RenderTarget,
    pub textbuf: TextBufferManager,
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
            }
        }
    }

    pub fn begin_frame(&self) {
        unsafe {
            C3D_FrameBegin(C3D_FRAME_SYNCDRAW);
            C2D_TargetClear(self.screen, colors::BACKGROUND);
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
        self.rect(x, y, width, height, colors::BLUE);
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
                colors::SHADOW,
            );
        }
    }

    pub fn text(&self, text: &C2D_Text, x: f32, y: f32, flags: u8, scale: f32) {
        self.draw_text(
            text,
            x,
            y,
            flags | C2D_WithColor,
            colors::TEXT_PRIMARY,
            scale,
        );
    }

    pub fn text_danger(&self, text: &C2D_Text, x: f32, y: f32, flags: u8, scale: f32) {
        self.draw_text(
            text,
            x,
            y,
            flags | C2D_WithColor,
            colors::TEXT_DANGER,
            scale,
        );
    }

    fn draw_text(&self, text: &C2D_Text, x: f32, y: f32, flags: u8, color: u32, scale: f32) {
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
