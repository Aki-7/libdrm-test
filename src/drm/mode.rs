#[repr(C)]
#[derive(Debug)]
pub struct drm_mode_create_dumb {
    pub height: u32,
    pub width: u32,
    pub bpp: u32,
    pub flags: u32,
    pub handle: u32,
    pub pitch: u32,
    pub size: u64,
}

#[repr(C)]
pub struct drm_mode_map_dumb {
    pub handle: u32,
    pub pad: u32,
    pub offset: u64,
}

#[repr(C)]
pub struct drm_mode_destroy_dumb {
    pub handle: u32,
}
