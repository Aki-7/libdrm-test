use std::fs::{File};
use std::os::unix::io::{AsRawFd};
use std::slice;
use std::ffi::CStr;
use std::io::{stdout, Write};

extern crate libc;
use libc::{c_int, c_char, c_void, printf};

pub const DRM_DISPLAY_MODE_LEN: usize = 32;

#[repr(C)]
#[derive(Debug)]
pub enum DrmModeConnection {
    Connected         = 1,
    Disconnected      = 2,
    UnknownConnection = 3,
}

#[repr(C)]
#[derive(Debug)]
pub enum DrmModeSubPixel {
    Unknown       = 1,
    HorizontalRGB = 2,
    HorizontalBGR = 3,
    VerticalRGB   = 4,
    VerticalBGR   = 5,
    None          = 6,
}

#[repr(C)]
struct _DrmModeRes {
    count_fbs: c_int,
    fbs: *const u32,

    count_crtcs: c_int,
    crtcs: *const u32,

    count_connectors: c_int,
    connectors: *const u32,

    count_encoders: c_int,
    encoders: *const u32,

    min_width: u32,
    max_width: u32,
    min_height: u32,
    max_height: u32,
}

#[derive(Debug)]
pub struct DrmModeRes<'a> {
    pub fbs: &'a [u32],
    pub crtcs: &'a [u32],
    pub connectors: &'a [u32],
    pub encoders: &'a [u32],
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
    ptr: *const _DrmModeRes,
}

unsafe fn drmModeResConvert<'a>(ptr: *const _DrmModeRes) -> Box<DrmModeRes<'a>> {
    let raw = &*ptr;
    Box::new(DrmModeRes{
        fbs: slice::from_raw_parts(raw.fbs, raw.count_fbs as usize),
        crtcs: slice::from_raw_parts(raw.crtcs, raw.count_crtcs as usize),
        connectors: slice::from_raw_parts(raw.connectors, raw.count_connectors as usize),
        encoders: slice::from_raw_parts(raw.encoders, raw.count_encoders as usize),
        min_width: raw.min_width,
        max_width: raw.max_width,
        min_height: raw.min_height,
        max_height: raw.max_height,
        ptr,
    })
}

#[repr(C)]
struct _DrmModeConnector {
    connector_id: u32,
    encoder_id: u32,
    connector_type: u32,
    connector_type_id: u32,
    connection: DrmModeConnection,
    mmWidth: u32,
    mmHeight: u32,
    subpixel: DrmModeSubPixel,

    count_modes: c_int,
    modes: *mut DrmModeModeInfo,

    count_props: c_int,
    props: *const u32,
    prop_values: *const u64,

    count_encoders: c_int,
    encoders: *const u32
}

#[derive(Debug)]
pub struct DrmModeConnector<'a>{
    pub connector_id: u32,
    pub encoder_id: u32,
    pub connector_type: u32,
    pub connector_type_id: u32,
    pub connection: &'a DrmModeConnection,
    pub mm_width: u32,
    pub mm_height: u32,
    pub subpixel: &'a DrmModeSubPixel,

    pub modes: &'a [DrmModeModeInfo],
    pub props: &'a [u32],
    pub prop_values: &'a [u64],
    pub encoders: &'a [u32],
    ptr: *const _DrmModeConnector,

}

unsafe fn drmModeConnectorConvert<'a>(ptr: *mut _DrmModeConnector) -> Box<DrmModeConnector<'a>> {
    let raw = &*ptr;
    Box::new(DrmModeConnector{
        connector_id: raw.connector_id,
        encoder_id: raw.encoder_id,
        connector_type: raw.connector_type,
        connector_type_id: raw.connector_type_id,
        connection: &raw.connection,
        mm_width: raw.mmWidth,
        mm_height: raw.mmHeight,
        subpixel: &raw.subpixel,
        modes: slice::from_raw_parts(raw.modes, raw.count_modes as usize),
        props: slice::from_raw_parts(raw.props, raw.count_props as usize),
        prop_values: slice::from_raw_parts(raw.prop_values, raw.count_props as usize),
        encoders: slice::from_raw_parts(raw.encoders, raw.count_encoders as usize),
        ptr,
    })
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct DrmModeModeInfo {
    pub clock: u32,
    pub hdisplay: u16,
    pub hsync_start: u16,
    pub hsync_end: u16,
    pub htotal: u16,
    pub hskew: u16,

    pub vdisplay: u16,
    pub vsync_start: u16,
    pub vsync_end: u16,
    pub vtotal: u16,
    pub vscan: u16,

    pub vrefresh: u32,

    pub flags: u32,
    pub r#type: u32,
    pub name: [c_char; DRM_DISPLAY_MODE_LEN]
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct DrmModeEncoder {
    pub encoder_id: u32,
    pub encoder_type: u32,
    pub crtc_id: u32,
    pub possible_crtcs: u32,
    pub possible_clones: u32,
}

#[repr(C)]
struct _DrmModeCrtc {
    crtc_id: u32,
    buffer_id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    mode_valid: c_int,
    mode: DrmModeModeInfo,
    gamma_size: c_int
}

#[derive(Debug, Clone)]
pub struct DrmModeCrtc<'a> {
    pub crtc_id: u32,
    pub buffer_id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub mode_valid: i32,
    pub mode: &'a DrmModeModeInfo,
    pub gamma_size: i32
}

unsafe fn drmModeCrtcConvert<'a>(ptr: *const _DrmModeCrtc) -> Box<DrmModeCrtc<'a>> {
    let raw = &*ptr;
    Box::new(DrmModeCrtc{
        crtc_id: raw.crtc_id,
        buffer_id: raw.buffer_id,
        x: raw.x,
        y: raw.y,
        width: raw.width,
        height: raw.height,
        mode_valid: raw.mode_valid,
        // mode: *drmModeModeInfoConvert(&raw.mode),
        mode: &raw.mode,
        gamma_size: raw.gamma_size,
    })
}

#[link(name = "drm")]
extern "C" {
    fn drmModeAddFB(fd: c_int, width: u32, height: u32, depth: u8, bpp: u32, pitch: u32, bo_handle: u32, buf_id: *const u32) -> c_int;
    fn drmModeRmFB(fb: c_int, bufferId: u32) -> c_int;
    fn drmModeFreeEncoder(ptr: *const DrmModeEncoder ) -> c_void;
    fn drmModeFreeConnector(ptr: *const _DrmModeConnector ) -> c_void;
    fn drmModeFreeResources(ptr: *const _DrmModeRes) -> c_void;
    fn drmModeGetResources(fd: c_int) -> *mut _DrmModeRes;
    fn drmModeGetConnector(fd: c_int, connectorId: u32) -> *mut _DrmModeConnector;
    fn drmModeGetCrtc(fd: c_int, crtcId: u32) -> *mut _DrmModeCrtc;
    fn drmModeGetEncoder(fd: c_int, encoderId: u32) -> *mut DrmModeEncoder;
    fn drmModeSetCrtc(fd: c_int, crtcId: u32, bufferId: u32, x: u32, y: u32,
        connectors: *const u32, count: c_int, mode: *const DrmModeModeInfo) -> c_int;
}

pub fn add_fb(file: &File, width: u32, height: u32, depth: u8, bpp: u32, pitch: u32, bo_handle: u32) -> Result<Box<u32>, i32> {
    let b = Box::new(0 as u32);
    let p = Box::into_raw(b);
    let res = unsafe { drmModeAddFB(file.as_raw_fd(), width, height, depth, bpp, pitch, bo_handle, p) };
    if res != 0 {
        return Err(res);
    };
    return unsafe { Ok(Box::from_raw(p)) };
}

pub fn rm_fb(file: &File, buf_id: u32) -> Result<(), i32> {
    let res = unsafe { drmModeRmFB(file.as_raw_fd(), buf_id) };
    if res != 0 {
        return Err(res);
    };
    return Ok(());
}

pub fn free_encoder(enc: Box<DrmModeEncoder>) {
    unsafe { drmModeFreeEncoder(Box::into_raw(enc)) };
}

pub fn free_connector(conn: Box<DrmModeConnector>) {
    unsafe { drmModeFreeConnector(conn.ptr) };
}

pub fn free_resources(res: Box<DrmModeRes>) {
    unsafe { drmModeFreeResources(res.ptr)};
}

pub fn get_resources(file: &File) -> Option<Box<DrmModeRes>> {
    unsafe {
        let _res_ptr = drmModeGetResources(file.as_raw_fd());
        if _res_ptr.is_null() {
            return None
        }
        let res = drmModeResConvert(_res_ptr);
        return Some(res);
    }
}

pub fn get_connector<'a>(file: &File, connector_id: u32) -> Option<Box<DrmModeConnector<'a>>> {
    unsafe {
        let _res_ptr = drmModeGetConnector(file.as_raw_fd(), connector_id);
        if _res_ptr.is_null() { return None };
        let res = drmModeConnectorConvert(_res_ptr);
        return Some(res);
    }
}

pub fn get_crtc(file: &File, crtc_id: u32) -> Option<Box<DrmModeCrtc>> {
    let _res_ptr = unsafe { drmModeGetCrtc(file.as_raw_fd(), crtc_id) };
    if _res_ptr.is_null() { return None };
    return unsafe { Some(drmModeCrtcConvert(_res_ptr)) };
}

pub fn get_encoder(file: &File, encoder_id: u32) -> Option<Box<DrmModeEncoder>> {
    let _res_ptr = unsafe { drmModeGetEncoder(file.as_raw_fd(), encoder_id) };
    if _res_ptr.is_null() { return None };
    return  unsafe { Some(Box::from_raw(_res_ptr)) };
}

pub fn set_crtc_one(file: &File, crtc_id: u32, buffer_id: u32, x: u32, y: u32, connector: u32, mode: &DrmModeModeInfo) -> Result<(), i32> {
    let b = Box::new(connector);
    let res = unsafe { drmModeSetCrtc(file.as_raw_fd(), crtc_id, buffer_id, x, y, &connector, 1, mode)};
    if res != 0 {
        return Err(res);
    };
    return Ok(())
}
