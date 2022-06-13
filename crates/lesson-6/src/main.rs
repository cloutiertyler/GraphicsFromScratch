use std::{fs, time::Instant};
use byteorder::WriteBytesExt;

const HEIGHT: usize = 512;
const WIDTH: usize = 512;
const BYTES_PER_PIXEL: usize = 3;
const ARRAY_LENGTH: usize = HEIGHT * WIDTH * BYTES_PER_PIXEL;

fn main() {
    // [0,0,0,0,0,0,0,0,0,0,0 ...]
    // [0 0 0   0 0 0   0 0 0 ...]
    //  ^         ^
    //  0         4
    let mut image_bytes: [u8; ARRAY_LENGTH] = [0; ARRAY_LENGTH];

    let now = Instant::now();
    for row in 0..HEIGHT {
        for column in 0..WIDTH {
            // [0 0 0   0 0 0   0 0 0 ...]
            //          ^ ^ ^
            //         pr g b
            let pixel_pointer: usize = WIDTH * BYTES_PER_PIXEL * row + BYTES_PER_PIXEL * column;
            let blue_pointer = pixel_pointer;
            let green_pointer = pixel_pointer + 1;
            let red_pointer = pixel_pointer + 2;

            if is_pixel_in_circle(row, column, WIDTH, HEIGHT) {
                image_bytes[red_pointer] = 0;
                image_bytes[green_pointer] = 0;
                image_bytes[blue_pointer] = 255;
            } else if is_pixel_in_triangle(row, column, WIDTH, HEIGHT) {
                image_bytes[red_pointer] = 255;
                image_bytes[green_pointer] = 0;
                image_bytes[blue_pointer] = 0;
            } else {
                image_bytes[red_pointer] = 0;
                image_bytes[green_pointer] = 0;
                image_bytes[blue_pointer] = 0;
            }
        }
    }
    println!("{}", now.elapsed().as_micros());

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

struct Vector {
    x: f32,
    y: f32,
    z: f32,
}

fn distance(p1: Vector, p2: Vector) -> f32 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    let dz = p1.z - p2.z;

    (dx*dx + dy*dy + dz*dz).sqrt()
}

struct Circle {
    center: Vector,
    radius: f32,
}

fn is_pixel_in_circle(row: usize, column: usize, row_size: usize, col_size: usize) -> bool {
    let x = (column as f32 - col_size as f32 / 2.0) / (0.5 * col_size as f32);
    let y = (row as f32 - row_size as f32 / 2.0) / (0.5 * row_size as f32);
    let z = 0.0;
    let point = Vector {x, y, z};

    let center = Vector {x: 0.0, y: 0.0, z: 0.0};
    let circle = Circle { center, radius: 0.3 };
    is_point_in_circle(circle, point)
}

fn is_point_in_circle(circle: Circle, point: Vector) -> bool {
    let dist = distance(circle.center, point);
    dist <= circle.radius
}

/*
   center: 0, 0, 0

   (-1, 1, 0)
   .____. (1, 1, 0)
   |   /
   | / 
   . (-1, -1, 0)
*/
struct Triangle {
    p0: Vector,
    p1: Vector,
    p2: Vector,
}

fn is_pixel_in_triangle(row: usize, column: usize, row_size: usize, col_size: usize) -> bool {
    let x = (column as f32 - col_size as f32 / 2.0) / (0.5 * col_size as f32);
    let y = (row as f32 - row_size as f32 / 2.0) / (0.5 * row_size as f32);
    let z = 0.0;
    let point = Vector {x, y, z};

    // let triangle = Triangle {
    //     p0: Vector { x: -0.5, y: 0.5, z: 0.0 },
    //     p1: Vector { x: 0.5, y: 0.5, z: 0.0 },
    //     p2: Vector { x: -0.5, y: -0.5, z: 0.0 },
    // };

    let triangle = Triangle {
        p0: Vector { x: -0.4, y: 0.4, z: 0.0 },
        p1: Vector { x: 0.8, y: 0.5, z: 0.0 },
        p2: Vector { x: -0.75, y: -0.75, z: 0.0 },
    };

    is_point_in_triangle(triangle, point)
}

fn is_point_in_triangle(t: Triangle, p: Vector) -> bool {
    // See: https://stackoverflow.com/a/2049712/1445441
    // use barycentric coordinates, who knew

    let s = (t.p0.x - t.p2.x) * (p.y - t.p2.y) - (t.p0.y - t.p2.y) * (p.x - t.p2.x);
    let v = (t.p1.x - t.p0.x) * (p.y - t.p0.y) - (t.p1.y - t.p0.y) * (p.x - t.p0.x);

    if (s < 0.0) != (v < 0.0) && s != 0.0 && v != 0.0 {
        return false;
    }

    let d = (t.p2.x - t.p1.x) * (p.y - t.p1.y) - (t.p2.y - t.p1.y) * (p.x - t.p1.x);
    return d == 0.0 || (d < 0.0) == (s + v <= 0.0);
}