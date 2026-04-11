// #[public_model] fixture marker
use fixed::types::I32F32;

#[must_use]
pub fn from_bits(bits: i64) -> I32F32 {
    I32F32::from_bits(bits)
}
