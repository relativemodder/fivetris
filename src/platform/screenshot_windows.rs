use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::thread;
use web_time::{Duration, Instant};

use crossbeam_channel::Sender;

use super::{PlatformEvent, ScreenshotImage, ScreenshotRequester};

#[derive(Debug, Clone)]
pub enum ScreenshotError {
    Portal(String),
    Clipboard(String),
    Capture(String),
}

impl std::fmt::Display for ScreenshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Portal(m) => write!(f, "snipping tool: {m}"),
            Self::Clipboard(m) => write!(f, "clipboard: {m}"),
            Self::Capture(m) => write!(f, "capture: {m}"),
        }
    }
}

impl std::error::Error for ScreenshotError {}

pub struct ScreenshotPortalService {
    tx: Sender<PlatformEvent>,
}

impl ScreenshotPortalService {
    pub fn new(tx: Sender<PlatformEvent>) -> Result<Self, std::io::Error> {
        Ok(Self { tx })
    }

    pub fn unavailable(tx: Sender<PlatformEvent>) -> Self {
        Self { tx }
    }

    pub fn is_available(&self) -> bool {
        true
    }
}

impl ScreenshotRequester for ScreenshotPortalService {
    fn request_interactive_capture(&self) {
        let tx = self.tx.clone();
        thread::spawn(move || capture_on_thread(tx));
    }
}

fn capture_on_thread(tx: Sender<PlatformEvent>) {
    let _ = tx.send(PlatformEvent::ScreenshotReady(capture_interactive()));
}

fn capture_interactive() -> Result<ScreenshotImage, ScreenshotError> {
    if launch_ms_screenclip() {
        let timeout = Instant::now() + Duration::from_secs(60);
        let seq = unsafe { ffi::GetClipboardSequenceNumber() };
        while Instant::now() < timeout {
            if unsafe { ffi::GetClipboardSequenceNumber() } != seq {
                if let Some(img) = read_clipboard_dib()? {
                    return Ok(img);
                }
            }
            unsafe { ffi::Sleep(200) };
        }
    }
    gdi_capture()
}

fn launch_ms_screenclip() -> bool {
    unsafe {
        let op = wide("open");
        let uri = wide("ms-screenclip:");
        let r = ffi::ShellExecuteW(
            ptr::null_mut(),
            op.as_ptr(),
            uri.as_ptr(),
            ptr::null(),
            ptr::null(),
            ffi::SW_SHOWNORMAL,
        );
        (r as isize) > 32
    }
}

fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn read_clipboard_dib() -> Result<Option<ScreenshotImage>, ScreenshotError> {
    unsafe {
        if ffi::OpenClipboard(ptr::null_mut()) == 0 {
            return Err(ScreenshotError::Clipboard("OpenClipboard failed".into()));
        }
        let handle = ffi::GetClipboardData(ffi::CF_DIB);
        if handle.is_null() {
            ffi::CloseClipboard();
            return Ok(None);
        }
        match dib_to_image(handle) {
            Ok(img) => {
                ffi::CloseClipboard();
                Ok(Some(img))
            }
            Err(e) => {
                ffi::CloseClipboard();
                Err(e)
            }
        }
    }
}

unsafe fn dib_to_image(handle: ffi::HANDLE) -> Result<ScreenshotImage, ScreenshotError> {
    let ptr = unsafe { ffi::GlobalLock(handle) };
    if ptr.is_null() {
        return Err(ScreenshotError::Clipboard("GlobalLock failed".into()));
    }

    let size = unsafe { ffi::GlobalSize(handle) };
    if size < std::mem::size_of::<ffi::BITMAPINFOHEADER>() {
        unsafe { ffi::GlobalUnlock(handle) };
        return Err(ScreenshotError::Clipboard("small DIB".into()));
    }

    let data = unsafe { std::slice::from_raw_parts(ptr as *const u8, size) };
    let hdr = unsafe { &*(ptr as *const ffi::BITMAPINFOHEADER) };
    if hdr.biBitCount != 32 {
        unsafe { ffi::GlobalUnlock(handle) };
        return Err(ScreenshotError::Clipboard(format!(
            "{}bpp unsupported",
            hdr.biBitCount
        )));
    }

    let w = hdr.biWidth as u32;
    let h = hdr.biHeight.unsigned_abs();
    let row_bytes = w as usize * 4;
    let pixels = &data[std::mem::size_of::<ffi::BITMAPINFOHEADER>()..];
    let mut rgba = Vec::with_capacity((w * h) as usize * 4);

    if hdr.biHeight > 0 {
        for y in (0..h as usize).rev() {
            let row = &pixels[y * row_bytes..][..row_bytes];
            for px in row.chunks_exact(4) {
                rgba.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
            }
        }
    } else {
        for px in pixels[..(w * h) as usize * 4].chunks_exact(4) {
            rgba.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
        }
    }

    unsafe { ffi::GlobalUnlock(handle) };
    Ok(ScreenshotImage {
        width: w,
        height: h,
        rgba,
    })
}

fn gdi_capture() -> Result<ScreenshotImage, ScreenshotError> {
    unsafe {
        let display = wide("DISPLAY");
        let dc = ffi::CreateDCW(
            display.as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        if dc.is_null() {
            return Err(ScreenshotError::Capture("CreateDCW".into()));
        }

        let w = ffi::GetDeviceCaps(dc, ffi::HORZRES);
        let h = ffi::GetDeviceCaps(dc, ffi::VERTRES);
        if w <= 0 || h <= 0 {
            ffi::DeleteDC(dc);
            return Err(ScreenshotError::Capture("bad size".into()));
        }

        let mem = ffi::CreateCompatibleDC(dc);
        if mem.is_null() {
            ffi::DeleteDC(dc);
            return Err(ScreenshotError::Capture("CreateCompatibleDC".into()));
        }

        let bm = ffi::CreateCompatibleBitmap(dc, w, h);
        if bm.is_null() {
            ffi::DeleteDC(mem);
            ffi::DeleteDC(dc);
            return Err(ScreenshotError::Capture("CreateCompatibleBitmap".into()));
        }

        let old = ffi::SelectObject(mem, bm);
        if ffi::BitBlt(mem, 0, 0, w, h, dc, 0, 0, ffi::SRCCOPY) == 0 {
            ffi::SelectObject(mem, old);
            ffi::DeleteObject(bm);
            ffi::DeleteDC(mem);
            ffi::DeleteDC(dc);
            return Err(ScreenshotError::Capture("BitBlt".into()));
        }

        let mut bmi = std::mem::zeroed::<ffi::BITMAPINFO>();
        bmi.bmiHeader.biSize = std::mem::size_of::<ffi::BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = w;
        bmi.bmiHeader.biHeight = -h;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = ffi::BI_RGB;

        let mut pixels = vec![0u8; (w * h) as usize * 4];
        if ffi::GetDIBits(
            mem,
            bm,
            0,
            h as u32,
            pixels.as_mut_ptr() as *mut std::ffi::c_void,
            &mut bmi,
            ffi::DIB_RGB_COLORS,
        ) == 0
        {
            ffi::SelectObject(mem, old);
            ffi::DeleteObject(bm);
            ffi::DeleteDC(mem);
            ffi::DeleteDC(dc);
            return Err(ScreenshotError::Capture("GetDIBits".into()));
        }

        for px in pixels.chunks_exact_mut(4) {
            px.swap(0, 2);
        }

        ffi::SelectObject(mem, old);
        ffi::DeleteObject(bm);
        ffi::DeleteDC(mem);
        ffi::DeleteDC(dc);

        Ok(ScreenshotImage {
            width: w as u32,
            height: h as u32,
            rgba: pixels,
        })
    }
}

mod ffi {
    #![allow(non_camel_case_types, non_snake_case, dead_code)]

    pub type HANDLE = *mut std::ffi::c_void;
    pub type LPVOID = *mut std::ffi::c_void;
    pub type BOOL = i32;
    pub type UINT = u32;
    pub type DWORD = u32;
    pub type LONG = i32;
    pub type WORD = u16;

    pub const SW_SHOWNORMAL: i32 = 1;
    pub const CF_DIB: UINT = 8;
    pub const SRCCOPY: DWORD = 0x00CC0020;
    pub const DIB_RGB_COLORS: UINT = 0;
    pub const BI_RGB: DWORD = 0;
    pub const HORZRES: i32 = 8;
    pub const VERTRES: i32 = 10;

    #[repr(C)]
    pub struct BITMAPINFOHEADER {
        pub biSize: DWORD,
        pub biWidth: LONG,
        pub biHeight: LONG,
        pub biPlanes: WORD,
        pub biBitCount: WORD,
        pub biCompression: DWORD,
        pub biSizeImage: DWORD,
        pub biXPelsPerMeter: LONG,
        pub biYPelsPerMeter: LONG,
        pub biClrUsed: DWORD,
        pub biClrImportant: DWORD,
    }

    #[repr(C)]
    pub struct BITMAPINFO {
        pub bmiHeader: BITMAPINFOHEADER,
        pub bmiColors: [u32; 1],
    }

    unsafe extern "system" {
        pub fn ShellExecuteW(
            HWND: LPVOID,
            op: *const u16,
            file: *const u16,
            params: *const u16,
            dir: *const u16,
            show: i32,
        ) -> HANDLE;
        pub fn OpenClipboard(hwnd: LPVOID) -> BOOL;
        pub fn CloseClipboard() -> BOOL;
        pub fn GetClipboardData(fmt: UINT) -> HANDLE;
        pub fn GetClipboardSequenceNumber() -> u32;
        pub fn GlobalLock(mem: HANDLE) -> LPVOID;
        pub fn GlobalUnlock(mem: HANDLE) -> BOOL;
        pub fn GlobalSize(mem: HANDLE) -> usize;
        pub fn Sleep(ms: u32);
        pub fn CreateDCW(
            driver: *const u16,
            device: LPVOID,
            output: LPVOID,
            init: LPVOID,
        ) -> HANDLE;
        pub fn CreateCompatibleDC(dc: HANDLE) -> HANDLE;
        pub fn CreateCompatibleBitmap(dc: HANDLE, w: i32, h: i32) -> HANDLE;
        pub fn SelectObject(dc: HANDLE, obj: HANDLE) -> HANDLE;
        pub fn DeleteDC(dc: HANDLE) -> BOOL;
        pub fn DeleteObject(obj: HANDLE) -> BOOL;
        pub fn BitBlt(
            dst: HANDLE,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
            src: HANDLE,
            sx: i32,
            sy: i32,
            rop: DWORD,
        ) -> BOOL;
        pub fn GetDIBits(
            dc: HANDLE,
            bm: HANDLE,
            start: UINT,
            lines: UINT,
            bits: LPVOID,
            info: *mut BITMAPINFO,
            usage: UINT,
        ) -> i32;
        pub fn GetDeviceCaps(dc: HANDLE, index: i32) -> i32;
    }
}
