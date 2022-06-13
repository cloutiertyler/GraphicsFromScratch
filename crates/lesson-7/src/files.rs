use std::{fs, io::Write};
use byteorder::WriteBytesExt;

pub fn write_image_file(width: usize, height: usize, image_bytes: impl AsRef<[u8]>) {
    let image_bytes = image_bytes.as_ref();
    let byte_len = image_bytes.len();

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open("image.bmp")
        .unwrap();

    // File header
    file.write_u8('B' as u8).unwrap();
    file.write_u8('M' as u8).unwrap();
    file.write_u32::<byteorder::LE>(14 + 40 + byte_len as u32).unwrap(); // file size
    file.write_u32::<byteorder::LE>(0).unwrap(); // reserved ??
    file.write_u32::<byteorder::LE>(14 + 40 as u32).unwrap(); // content offset

    // Info header
    file.write_u32::<byteorder::LE>(40 as u32).unwrap(); // info header size
    file.write_u32::<byteorder::LE>(width as u32).unwrap(); // width
    file.write_u32::<byteorder::LE>(height as u32).unwrap(); // height
    file.write_u16::<byteorder::LE>(1).unwrap(); // number of color planes???
    file.write_u16::<byteorder::LE>(24).unwrap(); // bits per pixel
    file.write_u32::<byteorder::LE>(0).unwrap(); // compression
    file.write_u32::<byteorder::LE>(byte_len as u32).unwrap(); // image size
    file.write_u32::<byteorder::LE>(0).unwrap(); // horizontal res
    file.write_u32::<byteorder::LE>(0).unwrap(); // vertical res
    file.write_u32::<byteorder::LE>(0).unwrap(); // colors in color table
    file.write_u32::<byteorder::LE>(0).unwrap(); // important color count

    file.write_all(image_bytes).unwrap();
}