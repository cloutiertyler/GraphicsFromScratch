use std::fs;
use byteorder::WriteBytesExt;

const HEIGHT: usize = 256;
const WIDTH: usize = 256;
const BYTES_PER_PIXEL: usize = 3;
const ARRAY_LENGTH: usize = HEIGHT * WIDTH * BYTES_PER_PIXEL;

fn main() {
    // [0,0,0,0,0,0,0,0,0,0,0 ...]
    // [0 0 0   0 0 0   0 0 0 ...]
    //  ^         ^
    //  0         4
    let mut image_bytes: [u8; ARRAY_LENGTH] = [0; ARRAY_LENGTH];

    for row in 0..HEIGHT {
        for column in 0..WIDTH {
            // [0 0 0   0 0 0   0 0 0 ...]
            //          ^ ^ ^
            //         pr g b
            let pixel_pointer: usize = WIDTH * BYTES_PER_PIXEL * row + BYTES_PER_PIXEL * column;
            let blue_pointer = pixel_pointer;
            let green_pointer = pixel_pointer + 1;
            let red_pointer = pixel_pointer + 2;
        
            image_bytes[red_pointer] = row as u8;
            image_bytes[green_pointer] = column as u8;
            image_bytes[blue_pointer] = 0;
        }
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open("image.bmp")
        .unwrap();

    // File header
    file.write_u8('B' as u8).unwrap();
    file.write_u8('M' as u8).unwrap();
    file.write_u32::<byteorder::LE>(14 + 40 + ARRAY_LENGTH as u32).unwrap(); // file size
    file.write_u32::<byteorder::LE>(0).unwrap(); // reserved ??
    file.write_u32::<byteorder::LE>(14 + 40 as u32).unwrap(); // content offset

    // Info header
    file.write_u32::<byteorder::LE>(40 as u32).unwrap(); // info header size
    file.write_u32::<byteorder::LE>(WIDTH as u32).unwrap(); // width
    file.write_u32::<byteorder::LE>(HEIGHT as u32).unwrap(); // height
    file.write_u16::<byteorder::LE>(1).unwrap(); // number of color planes???
    file.write_u16::<byteorder::LE>(BYTES_PER_PIXEL as u16 * 8).unwrap(); // bits per pixel
    file.write_u32::<byteorder::LE>(0).unwrap(); // compression
    file.write_u32::<byteorder::LE>(ARRAY_LENGTH as u32).unwrap(); // image size
    file.write_u32::<byteorder::LE>(0).unwrap(); // horizontal res
    file.write_u32::<byteorder::LE>(0).unwrap(); // vertical res
    file.write_u32::<byteorder::LE>(0).unwrap(); // colors in color table
    file.write_u32::<byteorder::LE>(0).unwrap(); // important color count

    for byte in image_bytes {
        file.write_u8(byte).unwrap();
    }
}