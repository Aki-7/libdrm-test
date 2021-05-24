use std::fs::File;
use std::os::unix::io::AsRawFd;
use ioctl_sys::iorw;
extern crate libc;
use libc::{c_int, c_void};

use super::Capability;

const DRM_IOCTL_BASE: char =  'd';
pub const DRM_IOCTL_MODE_CREATE_DUMB: u64
    = iorw!(DRM_IOCTL_BASE, 0xb2, std::mem::size_of::<super::mode::drm_mode_create_dumb>()) as u64;
pub const DRM_IOCTL_MODE_MAP_DUMB: u64
    = iorw!(DRM_IOCTL_BASE, 0xb3, std::mem::size_of::<super::mode::drm_mode_map_dumb>()) as u64;
pub const DRM_IOCTL_MODE_DESTROY_DUMB: u64
    = iorw!(DRM_IOCTL_BASE, 0xb4, std::mem::size_of::<super::mode::drm_mode_destroy_dumb>()) as u64;

#[link(name = "drm")]
extern "C" {
  fn drmGetCap(fd: c_int, capability: u64, value: *const u64) -> c_int;
  fn drmIoctl(fd: c_int, request: u64, arg: *const c_void) -> c_int;
}

pub fn get_cap(file: &File, capability: Capability) -> Result<u64, i32> {
    let value: u64 = 0;
    unsafe {
        let result = drmGetCap(file.as_raw_fd(), capability, &value);
        if result < 0 {
            return Err(result)
        }
    };
    return Ok(value)
}

pub fn ioctl<T>(file: &File, request: u64, arg: Box<T>) -> Result<Box<T>, i32> {
    let raw_ptr = Box::into_raw(arg);
    let res = unsafe { drmIoctl(file.as_raw_fd(),request, raw_ptr as *const c_void) };
    if res < 0 { return Err(-res)};
    let b = unsafe { Box::from_raw(raw_ptr)};
    return Ok(b);
}
