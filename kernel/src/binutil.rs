use bytemuck::NoUninit;

pub fn checksum_bytes<T: Sized + NoUninit>(value: &T) -> u8 {
    let mut sum: u8 = 0;
    let bytes = bytemuck::bytes_of(value);
    for i in 0..size_of::<T>() {
        sum = sum.wrapping_add(bytes[i]);
    }
    sum
}