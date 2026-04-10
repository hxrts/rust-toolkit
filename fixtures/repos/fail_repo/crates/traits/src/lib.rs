pub mod helper;
pub mod style;

pub trait SampleTrait {
    fn compute(&self) -> Result<u8, ()>;
}
