//! Carpetas conocidas del sistema. En Windows se preguntan a Shell por su ruta
//! real: Descargas, Escritorio y Documentos pueden estar redirigidos a OneDrive.

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnownFolder {
    Downloads,
    Desktop,
    Documents,
}

pub fn path(folder: KnownFolder) -> Option<PathBuf> {
    platform_path(folder).filter(|path| path.is_dir())
}

#[cfg(windows)]
fn platform_path(folder: KnownFolder) -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    #[repr(C)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHGetKnownFolderPath(
            id: *const Guid,
            flags: u32,
            token: *mut core::ffi::c_void,
            path: *mut *mut u16,
        ) -> i32;
    }

    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoTaskMemFree(pointer: *mut core::ffi::c_void);
    }

    const DOWNLOADS: Guid = Guid {
        data1: 0x374d_e290,
        data2: 0x123f,
        data3: 0x4565,
        data4: [0x91, 0x64, 0x39, 0xc4, 0x92, 0x5e, 0x46, 0x7b],
    };
    const DESKTOP: Guid = Guid {
        data1: 0xb4bf_cc3a,
        data2: 0xdb2c,
        data3: 0x424c,
        data4: [0xb0, 0x29, 0x7f, 0xe9, 0x9a, 0x87, 0xc6, 0x41],
    };
    const DOCUMENTS: Guid = Guid {
        data1: 0xfdd3_9ad0,
        data2: 0x238f,
        data3: 0x46af,
        data4: [0xad, 0xb4, 0x6c, 0x85, 0x48, 0x03, 0x69, 0xc7],
    };

    let id = match folder {
        KnownFolder::Downloads => &DOWNLOADS,
        KnownFolder::Desktop => &DESKTOP,
        KnownFolder::Documents => &DOCUMENTS,
    };

    let mut raw: *mut u16 = std::ptr::null_mut();
    let result =
        unsafe { SHGetKnownFolderPath(id, 0, std::ptr::null_mut(), &mut raw as *mut *mut u16) };
    if result < 0 || raw.is_null() {
        return None;
    }

    let mut length = 0usize;
    unsafe {
        while *raw.add(length) != 0 {
            length += 1;
        }
    }
    let text = unsafe { std::slice::from_raw_parts(raw, length) };
    let path = PathBuf::from(OsString::from_wide(text));
    unsafe { CoTaskMemFree(raw.cast()) };
    Some(path)
}

#[cfg(not(windows))]
fn platform_path(folder: KnownFolder) -> Option<PathBuf> {
    match folder {
        KnownFolder::Downloads => dirs::download_dir(),
        KnownFolder::Desktop => dirs::desktop_dir(),
        KnownFolder::Documents => dirs::document_dir(),
    }
}
