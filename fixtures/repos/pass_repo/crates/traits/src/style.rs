#[public_model]
/// Operating mode.
pub enum Mode {
    /// Feature is active.
    Enabled,
    /// Feature is inactive.
    Disabled,
}

/// Shape with explicitly-sized public fields.
pub struct PublicShape {
    /// Item count.
    pub count: u32,
    /// Timeout in milliseconds.
    pub timeout_ms: u32,
    /// Size in bytes.
    pub size_bytes: u32,
}

const RETRY_BACKOFF_MS_MAX: u32 = 3;

/// Compute the status for the given mode and timeout.
#[must_use]
pub fn compute_status(mode: Mode, timeout_ms: u32) -> Result<u32, ()> {
    assert!(timeout_ms > 0);
    Ok(match mode {
        | Mode::Enabled => timeout_ms + RETRY_BACKOFF_MS_MAX,
        | Mode::Disabled => timeout_ms,
    })
}

fn descend_once(depth: u32) -> u32 {
    if depth == 0 {
        0
    } else {
        depth - 1
    }
}

/// Return the count from a shape.
#[must_use]
pub fn public_shape_count(shape: &PublicShape) -> u32 {
    descend_once(shape.count)
}

// clone-allowed: lightweight value type used as a key
#[derive(Clone)]
/// A safe-to-clone value type.
pub struct SafeClone {
    /// The value.
    pub value: u32,
}
