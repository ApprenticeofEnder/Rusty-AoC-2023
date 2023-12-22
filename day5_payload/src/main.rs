#![no_std]
#![no_main]

extern crate alloc;

mod dos_tests;

use alloc::string::ToString;
use day5_payload::{*, dos::file::File};

entry!(main);

fn main() {
    let filename_src: &str = "TEST.TXT";

    let filename_out: &str = &(filename_src.to_string() + ".BAK");

    let fix_bytes: [u8; 2] = [0x41, 0x43];

    println!("Opening: {}", filename_src);

    let src_file: File = File::open(filename_src).expect("Check file name and try again?");

    println!("Opening for write: {}", filename_out);

    let out_file: File = File::open(filename_out).expect("Error opening outfile.");

    println!("Opened files.");

    println!("Reading...");

    let mut buf = [0; 13000]; //Target file is a little over 12,700 bytes

    let bytes_read = src_file.read(&mut buf).unwrap();

    println!("Read {} bytes", bytes_read);

    src_file.close().unwrap();
    
    fix_bytes.iter().enumerate().for_each(|(index, byte)| {
        buf[index] = *byte;
    });

    println!("First two bytes of buffer: {} {}", buf[0], buf[1]);
    println!("First two bytes of patch buffer: {} {}", fix_bytes[0], fix_bytes[1]);

    assert_eq!((buf[0], buf[1]), (fix_bytes[0], fix_bytes[1]));

    let bytes_written = out_file.write(&buf).unwrap();
    println!("Wrote {} bytes", bytes_written);

    out_file.close().unwrap();

    println!("Modification complete. Please confirm modifications manually.");
}
