extern crate goblin;

use std::default::Default;

// demonstrates "automagical" elf32/64 switches via cfg on arch and pub use hacks.
// SIZEOF_* will change depending on whether it's an x86_64 system or 32-bit x86, or really any cfg you can think of.
// similarly the printers will be different, since they have different impls. #typepuns4life

#[cfg(target_arch = "x86_64")]
pub use goblin::elf64 as elf;

#[cfg(target_arch = "x86")]
pub use goblin::elf32 as elf;

use elf::{header, sym};

fn main() {
    let header: header::Header = Default::default();
    let sym: sym::Sym = Default::default();
    println!("header: {:?}, sym: {:?}", header, sym);
    println!("sizeof header: {}", elf::header::SIZEOF_EHDR);
    println!("sizeof sym: {}", elf::sym::SIZEOF_SYM);
}
