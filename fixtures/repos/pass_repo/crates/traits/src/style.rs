pub enum Mode {
    Enabled,
    Disabled,
}

pub struct PublicShape {
    pub count: u32,
    pub timeout_ms: u32,
    pub size_bytes: u32,
}

const RETRY_BACKOFF_MS_MAX: u32 = 3;

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

#[must_use]
pub fn public_shape_count(shape: &PublicShape) -> u32 {
    descend_once(shape.count)
}
