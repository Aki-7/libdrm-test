use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Write};

extern crate libc;
use libc::{rand, srand, time, usleep, printf};

mod drm;

fn modeset_open<'a>(card: &String) -> Result<File, String> {
    let file = match OpenOptions::new().read(true).write(true).open(card) {
        Err(e) => Err(format!("cannot open '{}': {}", card, e))?,
        Ok(f) => f,
    };

    match drm::xf86drm::get_cap(&file, drm::CAP_DUMB_BUFFER) {
        Ok(value) if value != 0 => return Ok(file),
        _ => Err(format!("drm device '{}' down not support dumb buffers", card))?,
    }
}

#[derive(Debug)]
struct ModesetDev<'a> {
    width: u16,
    height: u16,
    stride: u32,
    size: u64,
    handle: u32,
    map_offset: u64,
    mode: drm::xf86drmMode::DrmModeModeInfo,
    fb: u32,
    conn: u32,
    crtc: u32,
    saved_crtc: Option<Box<drm::xf86drmMode::DrmModeCrtc<'a>>>,
}

fn modeset_prepare(file: &File) -> Result<Vec<Box<ModesetDev>>, String> {
    let mut dev_list: Vec<Box<ModesetDev>> = Vec::new();

    let res = match drm::xf86drmMode::get_resources(&file) {
        None => Err(format!("cannot retrieve DRM resources"))?,
        Some(b) => b,
    };

    for connector_id in (*res).connectors {
        let conn = match drm::xf86drmMode::get_connector(&file, *connector_id) {
            None => Err(format!("cannot retrieve DRM connector {}", connector_id))?,
            Some(c) => c,
        };
        let dev = match modeset_setup_dev(&file, &res, &conn, &dev_list) {
            Err(e) => {
                println!("{}", e);
                drm::xf86drmMode::free_connector(conn);
                continue;
            },
            Ok(d) => d,
        };

        drm::xf86drmMode::free_connector(conn);
        dev_list.push(dev);
    }

    drm::xf86drmMode::free_resources(res);

    return Ok(dev_list)
}

fn modeset_setup_dev<'a>(file: &File, res: &drm::xf86drmMode::DrmModeRes,
    conn: &drm::xf86drmMode::DrmModeConnector, dev_list: &Vec<Box<ModesetDev>>) -> Result<Box<ModesetDev<'a>>, String> {
    if let drm::xf86drmMode::DrmModeConnection::Connected = conn.connection {} else {
        return Err(format!("ignoring unused connector {}", conn.connector_id));
    }

    if conn.modes.len() == 0 {
        return Err(format!("no valid mode for connector {}", conn.connector_id));
    }

    let mode = conn.modes[0].clone();
    let crtc = modeset_find_crtc(&file, conn, dev_list)?;
    let (stride, handle, size, fb, map_offset) = modeset_create_fb(&file, mode.hdisplay as u32, mode.vdisplay as u32)?;
    let dev = ModesetDev{
        width: mode.hdisplay,
        height: mode.vdisplay,
        stride,
        size: size as u64,
        handle,
        map_offset,
        mode: mode.clone(),
        fb,
        conn: conn.connector_id,
        crtc: crtc,
        saved_crtc: None,
    };

    println!("mode for connector {} is {}x{}", conn.connector_id, dev.mode.hdisplay, dev.mode.vdisplay);

    return Ok(Box::new(dev));
}

fn modeset_find_crtc(file: &File, conn: &drm::xf86drmMode::DrmModeConnector, dev_list: &Vec<Box<ModesetDev>>) -> Result<u32, String> {
    let enc_op;
    if conn.encoder_id != 0 {
        enc_op = drm::xf86drmMode::get_encoder(&file, conn.encoder_id);
    } else {
        enc_op = None;
    }

    if let Some(enc_b) = enc_op {
        let enc = *enc_b.clone();
        if enc.crtc_id != 0 {
            let mut crtc: i64 = enc.crtc_id as i64;
            for dev in dev_list {
                if dev.crtc == crtc as u32 {
                    crtc = -1;
                    break;
                }
            }

            if crtc >= 0 {
                drm::xf86drmMode::free_encoder(enc_b);
                return Ok(crtc as u32);
            };
        };
    };
    return Err(format!("cannot find suitable CRTC for connector {}", conn.connector_id));
}

fn modeset_create_fb(file: &File, width: u32, height: u32) -> Result<(u32, u32, u32, u32, u64), String>{
    let stride: u32;
    let size: u32;
    let handle: u32;

    let creq = Box::new(drm::mode::drm_mode_create_dumb{
        height: height,
        width: width,
        bpp: 32,
        flags: 0,
        handle: 0,
        pitch: 0,
        size: 0,
    });
    let creq = match drm::xf86drm::ioctl(file, drm::xf86drm::DRM_IOCTL_MODE_CREATE_DUMB, creq) {
        Err(e) => Err(format!("cannot create dumb buffer ({})", e))?,
        Ok(req) => *req,
    };

    stride = creq.pitch;
    size = creq.size as u32;
    handle = creq.handle;

    let fb = match drm::xf86drmMode::add_fb(file, width, height, 32, 32, stride, handle) {
        Err(e) => {
            err_destroy(file, handle);
            return Err(format!("cannot create frame buffer ({})", e))
        },
        Ok(fb) => *fb,
    };

    let mreq = Box::new(drm::mode::drm_mode_map_dumb{
        handle: handle,
        pad: 0,
        offset: 0,
    });

    let mreq = match drm::xf86drm::ioctl(file, drm::xf86drm::DRM_IOCTL_MODE_MAP_DUMB, mreq) {
        Err(e) => {
            err_fb(file, handle, fb);
            return Err(format!("cannot map dumb buffer ({})", e))
        },
        Ok(req) => *req,
    };

    let m = match unsafe { memmap::MmapOptions::new().len(size as usize).offset(mreq.offset).map_mut(file) } {
        Err(e) => {
            err_fb(file, handle, fb);
            return Err(format!("cannot mmap dump buffer: {}", e))
        }
        Ok(m) => m,
    };

    let p = m.as_ptr() as *mut u8;
    (0..size).for_each(|x| unsafe { std::ptr::write_volatile(p.offset(x as isize), 0) });
    Ok((stride, handle, size, fb, mreq.offset))
}

fn err_fb(file: &File, handle: u32, fb: u32) {
    let _ = drm::xf86drmMode::rm_fb(file, fb);
    err_destroy(file, handle);
}

fn err_destroy(file: &File, handle: u32) {
    let dreq = Box::new(drm::mode::drm_mode_destroy_dumb{
        handle: handle,
    });
    let _ = drm::xf86drm::ioctl(file, drm::xf86drm::DRM_IOCTL_MODE_DESTROY_DUMB, dreq);
}

fn main() {

    match _main() {
        Err(e) => println!("{}", e),
        Ok(_) => println!("success")
    }
    println!("done")
}

fn _main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let card;
    if args.len() > 1 {
        card = args[1].clone();
    } else {
        card = String::from("/dev/dri/card0");
    }

    println!("using card '{}'", card);

    let file = modeset_open(&card)?;

    let mut dev_list = modeset_prepare(&file)?;

    for dev in &mut dev_list {
        dev.saved_crtc = drm::xf86drmMode::get_crtc(&file, dev.crtc);
        // println!("crtc:{}, fb: {}, conn: {}, mode: {:?}", dev.crtc, dev.fb, dev.conn, dev.mode);
        match drm::xf86drmMode::set_crtc_one(&file, dev.crtc, dev.fb, 0 as u32, 0 as u32, dev.conn, &dev.mode) {
            Err(e) => {
                println!("cannot set CRTC for connector {} ({})", dev.conn, e);
            },
            Ok(_) => {}
        };
    }

    modest_draw(&dev_list, &file);

    for dev in &mut dev_list {
        if let Some(crtc_box) = &dev.saved_crtc {
            let crtc = &**crtc_box;
            let _ = drm::xf86drmMode::set_crtc_one(&file, crtc.crtc_id, crtc.buffer_id, crtc.x, crtc.y, dev.conn, crtc.mode);
        }
    }

    Ok(())
}

fn next_color(up: bool, cur: i16, md: i32) -> (i16, bool) {
    let m: i16 = (unsafe{rand()} % md) as i16;
    let mut next: i16 = cur + m * if up {1 as i16} else {-1 as i16} as i16;
    let mut ret: bool = up;
    if (up && next < cur) || (!up && next > cur) {
        ret = !up;
        next = cur;
    };
    return (next, ret);
}

fn modest_draw(dev_list: &Vec<Box<ModesetDev>>, file: &File) -> Result<(), String> {
    unsafe { srand(time(std::ptr::null_mut()) as u32) }
    let mut r: i16 = (unsafe { rand() } % 0xff) as i16;
    let mut g: i16 = (unsafe { rand() } % 0xff) as i16;
    let mut b: i16 = (unsafe { rand() } % 0xff) as i16;
    let mut r_up = true;
    let mut g_up = true;
    let mut b_up = true;
    let mut off: isize = 0;

    let dev = &dev_list[0];
    let m = match unsafe { memmap::MmapOptions::new().len(dev.size as usize).offset(dev.map_offset).map_mut(file)} {
        Err(e) => {
            err_fb(file, dev.handle, dev.fb);
            return Err(format!("cannot mmap dumb buffer: {}", e));
        },
        Ok(m) => m,
    };
    let p = m.as_ptr() as *mut u32;

    println!("size: {}, h: {}, w: {}, stride: {}", dev.size, dev.height, dev.width, dev.stride);
    let st = (dev.stride >> 2) as isize;
    let width = dev.width as isize;
    let height = dev.height as isize;
    (0..50).for_each(|i| {
        let vals = next_color(r_up, r, 20); r = vals.0; r_up = vals.1;
        let vals = next_color(g_up, g, 10); g = vals.0; g_up = vals.1;
        let vals = next_color(b_up, b,  5); b = vals.0; b_up = vals.1;

        for y in 0..height {
            let k = st * y;
            for x in 0..width {
                off = k + x;
                if cross(x, y, width, height) { continue }
                unsafe { std::ptr::write_volatile(p.offset(off), ((r as u32) << 16)  | ((g as u32) << 8) | b as u32 )};
            }
        }
        unsafe { usleep(100000); }
    });
    return Ok(());
}

fn cross(x: isize, y: isize, width: isize, height: isize) -> bool {
    if height / 16 * 3 < y &&  y < height / 16 * 4 { return true }
    if width / 32 * 5 < x && x < width / 32 * 6 { return true}
    return false;
}
