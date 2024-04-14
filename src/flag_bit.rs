pub struct FlagBits;
impl FlagBits {
    pub fn is_bit_active(flag: u8, index: u8) -> bool {
        ((flag >> index) & 1) == 1
    }
}