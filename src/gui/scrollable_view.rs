use std::{cmp::min, ops::Div};

use crate::gui::Gui;

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
        let count_pages = self.data.count_items() / max_items + 1;
        let current_page = self.highlighted_item / max_items;
        let page_start = current_page * max_items;

        let mut y = self.y;
        for index in page_start..page_start + min(self.data.count_items() - page_start, max_items) {
            if index == self.highlighted_item {
                gui.rect(self.x, y, self.width, self.item_height, gui.highlight_color);
            }
            self.data
                .render_line(gui, index, self.x, y, self.width, self.item_height);
            y += self.item_height;
        }

        gui.rect(self.x, self.y, 5.0, self.height, gui.dialog_overlay);
        let indicator_height = self.height / count_pages as f32;
        gui.rect(
            self.x,
            self.y + indicator_height * current_page as f32,
            5.0,
            indicator_height,
            gui.highlight_color,
        );
    }
}

pub trait ScrollableViewData {
    fn render_line(&self, gui: &Gui, index: usize, x: f32, y: f32, width: f32, height: f32);
    fn count_items(&self) -> usize;
}
