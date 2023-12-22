#![no_std]
#![no_main]

extern crate alloc;

mod dos_tests;

use day5_payload::{
    dos::file::{File, SeekFrom},
    entry,
};

entry!(main);

fn main() {
    let filename: &str = "AC2023.BAK";

    let file_to_modify: File = File::open(filename).expect("Check file name and try again?");

    let bytes: [u8; 2] = [0x41, 0x43];
    file_to_modify.seek(SeekFrom::Start(0)).unwrap();
    file_to_modify.write(&bytes).unwrap();
}
