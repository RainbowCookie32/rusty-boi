pub fn check_bit(value: u8, bit: u8) -> bool {
    (value & (1 << bit)) != 0
}