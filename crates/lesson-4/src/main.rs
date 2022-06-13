use std::{fs, io::Write};

fn main() {
    // [0,0,0,0,0,0,0,0,0,0,0,0,0,0 ...]
    let mut bytes: [u8; 128] = [0; 128];

    let mut i: u8 = 0;
    loop {
        if i == 128 {
            break;
        }
        bytes[i as usize] = i;
        i = i + 1;
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open("bytes.txt")
        .unwrap();

    for byte in bytes {
        file.write(&[byte]).unwrap();
    }
}
