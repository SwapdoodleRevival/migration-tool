use std::{ffi::CStr, mem};

use citro2d_sys::{
    C2D_Text, C2D_TextBuf, C2D_TextBufDelete, C2D_TextBufNew, C2D_TextBufResize, C2D_TextOptimize,
    C2D_TextParse,
};

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
