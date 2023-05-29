
fn main() {
    // [0,1,2,3,4,5,6,0,0,0,0,0,0,0 ...]
    let mut bytes: [u8; 128] = [0; 128];

    let mut i: u8 = 0;
    loop {
        if i == 128 {
            break;
        }
        bytes[i as usize] = i;
        i = i + 1;
    }

    println!("{:?}", bytes);
}
