
// Capability
pub type Capability = u64;
pub const CAP_DUMB_BUFFER: Capability = 1;

pub mod xf86drm;
pub mod xf86drmMode;
pub mod mode;