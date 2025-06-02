use std::fmt::Display;

pub struct Menu<B: MenuBehavior, O: Display> {
    behavior: B,
    selection: usize,
    options: Vec<O>,
}

impl<B: MenuBehavior, O: Display> Menu<B, O> {}

pub trait MenuBehavior {
    fn get_options(&self) -> Vec<Box<dyn Display>>;
}
