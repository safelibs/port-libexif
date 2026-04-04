use std::panic::{catch_unwind, AssertUnwindSafe};

#[inline]
pub fn call_or<T: Copy>(default: T, body: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(value) => value,
        Err(_) => default,
    }
}

#[inline]
pub fn call_void(body: impl FnOnce()) {
    let _ = catch_unwind(AssertUnwindSafe(body));
}
