#![no_std]
#![no_main]

extern crate alloc;

mod dos_tests;

use rust_dos::{
    dos::file::{set_verify_writes, verify_writes, AccessCode, AccessMode, File, SharingMode},
    *,
};

use rust_dos::bios::video;
use rust_dos::bios::video::VesaMode;

entry!(main);

#[allow(dead_code)]
const PICTURE_DATA: [[u8; 8]; 8] = [
    [000, 000, 000, 000, 000, 000, 000, 0],
    [000, 000, 128, 000, 000, 128, 000, 0],
    [000, 000, 128, 000, 000, 128, 000, 0],
    [000, 000, 128, 000, 000, 128, 000, 0],
    [000, 000, 000, 000, 000, 000, 000, 0],
    [000, 128, 000, 000, 000, 000, 128, 0],
    [000, 000, 128, 128, 128, 128, 000, 0],
    [000, 000, 000, 000, 000, 000, 000, 0],
];

fn main() {
    // Set resolution to 800x600x8
    let mode = VesaMode::new(0x103, false, true, false);

    video::set_video_vesa(mode).unwrap();

    set_verify_writes(true);

    let backup_file = File::open(
        "C:\\AC2023.BAK\0",
        AccessMode::new(AccessCode::Write, SharingMode::Compatibility, false),
    )
    .unwrap();

    let fix_bytes: [u8; 2] = [0x41, 0x43];

    let written_bytes = backup_file.write(&fix_bytes).unwrap();

    println!("Wrote {} bytes", written_bytes);

    backup_file.close().unwrap();
    println!("Modification complete. Please confirm modifications manually.");

    println!("Write verification status: {}", verify_writes());
}
