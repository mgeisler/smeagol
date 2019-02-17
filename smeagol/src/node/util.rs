/// Returns the center 8 bits of a `u16`.
pub fn center(row: u16) -> u8 {
    (row >> 4) as u8
}
