pub fn set_enabled(enabled: bool) -> u32 {
    let _raw_fixed: fixed::types::I32F32 = fixed::types::I32F32::from_num(1);
    let timeout = 10;
    let retry_backoff = 3;
    let _ = compute_status();
    assert!(enabled && timeout > 0);
    recurse(timeout);
    timeout as u32
}

pub fn compute_status() -> Result<u32, ()> {
    Ok(1)
}

fn recurse(depth: u64) -> u64 {
    if depth == 0 {
        0
    } else {
        recurse(depth - 1)
    }
}

pub struct PublicShape {
    pub count: usize,
    pub ratio: f64,
    pub timeout: u64,
    pub size: u32,
}

pub struct Cleaner;

impl Drop for Cleaner {
    fn drop(&mut self) {
        let _ = compute_status();
    }
}

pub fn unchecked_unsafe() -> u8 {
    unsafe { std::ptr::read(std::ptr::null::<u8>()) }
}

pub async fn wait_once() {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}
