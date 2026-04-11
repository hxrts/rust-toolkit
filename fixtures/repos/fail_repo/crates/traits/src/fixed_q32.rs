use fixed::types::I32F32;

pub fn from_bits(bits: i64) -> I32F32 {
    I32F32::from_bits(bits)
}

pub fn visit_f64(value: f64) -> I32F32 {
    I32F32::from_num(value)
}
