pub fn set_enabled(enabled: bool) -> u32 {
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

// --- annotation_scope violation: #[public_model] on a function (only allowed on struct/enum) ---
#[public_model]
pub fn wrongly_annotated() {}

// --- unwrap_guard violation: bare unwrap without marker ---
pub fn unwrap_violation() -> u32 {
    let value: Option<u32> = Some(42);
    value.unwrap()
}

// --- allow_attribute_guard violation: bare #[allow] without marker ---
#[allow(dead_code)]
fn allow_violation() {}

// --- cloning_boundary violation: bare #[derive(Clone)] without marker ---
#[derive(Clone)]
pub struct CloneViolation {
    pub data: u32,
}

// --- fn_length violation: function exceeding warn threshold ---
pub fn overly_long_function() -> u32 {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
    let h = 8;
    let i = 9;
    let j = 10;
    let k = 11;
    let l = 12;
    let m = 13;
    let n = 14;
    let o = 15;
    let p = 16;
    let q = 17;
    let r = 18;
    let s = 19;
    let t = 20;
    let u = 21;
    let v = 22;
    let w = 23;
    let x = 24;
    let y = 25;
    let z = 26;
    let aa = 27;
    let bb = 28;
    let cc = 29;
    let dd = 30;
    let ee = 31;
    let ff = 32;
    let gg = 33;
    let hh = 34;
    let ii = 35;
    let jj = 36;
    let kk = 37;
    let ll = 38;
    let mm = 39;
    let nn = 40;
    let oo = 41;
    let pp = 42;
    let qq = 43;
    let rr = 44;
    let ss = 45;
    let tt = 46;
    let uu = 47;
    let vv = 48;
    let ww = 49;
    let xx = 50;
    let yy = 51;
    let zz = 52;
    let aaa = 53;
    let bbb = 54;
    let ccc = 55;
    let ddd = 56;
    let eee = 57;
    let fff = 58;
    let ggg = 59;
    let hhh = 60;
    a + b + c + d + e + f + g + h + i + j + k + l + m + n + o + p + q + r + s
        + t + u + v + w + x + y + z + aa + bb + cc + dd + ee + ff + gg + hh
        + ii + jj + kk + ll + mm + nn + oo + pp + qq + rr + ss + tt + uu + vv
        + ww + xx + yy + zz + aaa + bbb + ccc + ddd + eee + fff + ggg + hhh
}
