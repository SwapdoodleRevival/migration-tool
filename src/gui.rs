use std::{ffi::CStr, mem};

use citro2d_sys::{
    C2D_CreateScreenTarget, C2D_Init, C2D_Prepare, C2D_Text, C2D_TextBuf, C2D_TextBufDelete, C2D_TextBufNew, C2D_TextOptimize, C2D_TextParse, C3D_RenderTarget, C2D_DEFAULT_MAX_OBJECTS
};
use citro3d_sys::{C3D_DEFAULT_CMDBUF_SIZE, C3D_Init};
use ctru::{GFX_BOTTOM, GFX_LEFT, GFX_TOP};
use ctru_sys as ctru;

pub struct GUI {
    pub screen: *mut C3D_RenderTarget,
}

pub const TOP_SCREEN_WIDTH: f32 = 400.0;

impl GUI {
    pub fn init() -> Self {
        unsafe {
            C3D_Init(C3D_DEFAULT_CMDBUF_SIZE as usize);
            C2D_Init(C2D_DEFAULT_MAX_OBJECTS as usize);

            let screen = C2D_CreateScreenTarget(GFX_TOP, GFX_LEFT);

            C2D_Prepare();
            Self { screen }
        }
    }

    pub fn make_static_text(textbuf: C2D_TextBuf, content: &CStr) -> C2D_Text {
        unsafe {
            let mut text: C2D_Text = mem::zeroed();
            C2D_TextParse(&mut text as *mut _, textbuf, content.as_ptr());
            C2D_TextOptimize(&mut text as *mut _);
            text
        }
    }
}

pub struct TextBuffer {
    buf: C2D_TextBuf,
}

impl TextBuffer {
    pub fn init(size: usize) -> Self {
        unsafe {
            Self {
                buf: C2D_TextBufNew(size),
            }
        }
    }

    pub fn make_static_text(&self, content: &CStr) -> C2D_Text {
        unsafe {
            let mut text: C2D_Text = mem::zeroed();
            C2D_TextParse(&mut text as *mut _, self.buf, content.as_ptr());
            C2D_TextOptimize(&mut text as *mut _);
            text
        }
    }
}

impl Drop for TextBuffer {
    fn drop(&mut self) {
        unsafe {
            C2D_TextBufDelete(self.buf);
        }
    }
}
