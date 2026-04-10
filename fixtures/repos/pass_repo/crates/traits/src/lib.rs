#![forbid(unsafe_code)]

pub mod style;

#[must_use]
pub trait SampleTrait {
    #[must_use]
    fn compute(&self) -> Result<u8, ()>;
}
