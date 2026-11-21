//! What the storybook's tests share.

use guido::testing::Headless;

/// The headless app for a test, or `None` without a GPU adapter (or a device
/// on it), so the test skips. With `GUIDO_GPU_REQUIRED` set, as in CI, it
/// panics instead: the variable Guido's own tests honour.
pub fn headless() -> Option<Headless> {
    let h = Headless::new();
    if h.is_none() && std::env::var_os("GUIDO_GPU_REQUIRED").is_some() {
        panic!("GUIDO_GPU_REQUIRED is set and no GPU adapter or device was found");
    }
    h
}
