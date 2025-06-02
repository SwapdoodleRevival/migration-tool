pub mod intro;
pub mod mapping;

pub use intro::*;
pub use mapping::*;

fn print_center(a: &str) {
    println!("{:^1$}", a, 50);
}