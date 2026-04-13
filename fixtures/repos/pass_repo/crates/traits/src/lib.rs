#![forbid(unsafe_code)]

/// Style utilities for the fixture crate.
pub mod style;

/// Sample trait demonstrating must_use on trait methods.
#[must_use]
pub trait SampleTrait {
    /// Compute a value.
    #[must_use]
    fn compute(&self) -> Result<u8, ()>;
}
