pub mod intro;
pub mod mapping;
pub mod rewrite;

pub use intro::*;
pub use mapping::*;
pub use rewrite::*;

fn print_center(a: &str) {
    println!("{:^1$}", a, 50);
}
