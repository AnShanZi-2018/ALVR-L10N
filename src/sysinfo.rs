//! Minimal hardware probing through Win32 / registry, no extra crates.

#[derive(Clone, Debug, Default)]
pub struct Gpu {
    pub name: String,
    pub vram_mb: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Hardware {
    pub cpu: String,
    pub ram_gb: u64,
    pub gpus: Vec<Gpu>,
    pub disk_free_gb: u64,
    pub headset: Option<String>,
}

const REG_SZ: u32 = 0x0000_0002; // RRF_RT_REG_SZ
const REG_QWORD: u32 = 0x0000_0040; // RRF_RT_REG_QWORD

#[cfg(windows)]
mod ffi {
    #[link(name = "advapi32")]
    extern "system" {
        pub fn RegGetValueW(
            key: isize,
            subkey: *const u16,
            value: *const u16,
            flags: u32,
            kind: *mut u32,
            data: *mut u8,
            size: *mut u32,
        ) -> i32;
        pub fn RegOpenKeyExW(
            key: isize,
            subkey: *const u16,
            options: u32,
            rights: u32,
            result: *mut isize,
        ) -> i32;
        pub fn RegCloseKey(key: isize) -> i32;
        pub fn RegEnumKeyExW(
            key: isize,
            index: u32,
            name: *mut u16,
            size: *mut u32,
            reserved: *mut u32,
            class: *mut u16,
            class_len: *mut u32,
            last_write: *mut u64,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GlobalMemoryStatusEx(buffer: *mut MemoryStatus) -> i32;
    }

    #[repr(C)]
    #[derive(Default)]
    pub struct MemoryStatus {
        pub length: u32,
        pub memory_load: u32,
        pub total_phys: u64,
        pub avail_phys: u64,
        pub total_page_file: u64,
        pub avail_page_file: u64,
        pub total_virtual: u64,
        pub avail_virtual: u64,
        pub avail_extended_virtual: u64,
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Read a registry value under HKEY_LOCAL_MACHINE.
/// kind is an RRF_RT_* flag (2 = REG_SZ, 64 = REG_QWORD).
#[cfg(windows)]
fn reg_value(path: &str, name: &str, kind: u32) -> Option<Vec<u8>> {
    let path_w = wide(path);
    let name_w = wide(name);
    // HKEY_LOCAL_MACHINE = 0x80000002
    let mut buffer = vec![0u8; 512];
    let mut size = buffer.len() as u32;
    let mut actual_kind = 0u32;
    let status = unsafe {
        ffi::RegGetValueW(
            0x8000_0002u32 as isize,
            path_w.as_ptr(),
            name_w.as_ptr(),
            kind,
            &mut actual_kind,
            buffer.as_mut_ptr(),
            &mut size,
        )
    };
    if status != 0 || size == 0 || size as usize > buffer.len() {
        return None;
    }
    buffer.truncate(size as usize);
    Some(buffer)
}

#[cfg(windows)]
fn reg_string(path: &str, name: &str) -> Option<String> {
    let bytes = reg_value(path, name, REG_SZ)?;
    let words: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|word| *word != 0)
        .collect();
    let text = String::from_utf16_lossy(&words).trim().to_owned();
    (!text.is_empty()).then_some(text)
}

#[cfg(windows)]
fn reg_qword(path: &str, name: &str) -> Option<u64> {
    let bytes = reg_value(path, name, REG_QWORD)?;
    if bytes.len() < 8 {
        return None;
    }
    Some(u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

#[cfg(windows)]
fn subkeys(path: &str) -> Vec<String> {
    let mut names = Vec::new();
    let path_w = wide(path);
    let mut key: isize = 0;
    // KEY_READ
    if unsafe {
        ffi::RegOpenKeyExW(
            0x8000_0002u32 as isize,
            path_w.as_ptr(),
            0,
            0x20019,
            &mut key,
        )
    } != 0
    {
        return names;
    }
    let mut index = 0u32;
    loop {
        let mut name = [0u16; 256];
        let mut len = name.len() as u32;
        let status = unsafe {
            ffi::RegEnumKeyExW(
                key,
                index,
                name.as_mut_ptr(),
                &mut len,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if status != 0 {
            break;
        }
        names.push(String::from_utf16_lossy(&name[..len as usize]));
        index += 1;
    }
    unsafe { ffi::RegCloseKey(key) };
    names
}

#[cfg(windows)]
fn gpus() -> Vec<Gpu> {
    const CLASS: &str =
        "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}";
    let mut list = Vec::new();
    for subkey in subkeys(CLASS) {
        if !subkey.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let path = format!("{CLASS}\\{subkey}");
        let Some(name) = reg_string(&path, "DriverDesc") else {
            continue;
        };
        let vram = reg_qword(&path, "HardwareInformation.qwMemorySize")
            .map(|bytes| bytes / (1024 * 1024))
            .unwrap_or(0);
        list.push(Gpu {
            name,
            vram_mb: vram,
        });
    }
    list
}

#[cfg(windows)]
fn memory_status() -> u64 {
    let mut status = ffi::MemoryStatus {
        length: std::mem::size_of::<ffi::MemoryStatus>() as u32,
        ..Default::default()
    };
    let ok = unsafe { ffi::GlobalMemoryStatusEx(&mut status) };
    if ok == 0 {
        0
    } else {
        status.total_phys / (1024 * 1024 * 1024)
    }
}

#[cfg(windows)]
fn free_space_gb(path: &std::path::Path) -> u64 {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetDiskFreeSpaceExW(
            directory: *const u16,
            free_to_caller: *mut u64,
            total: *mut u64,
            total_free: *mut u64,
        ) -> i32;
    }
    let mut free = 0u64;
    let mut total = 0u64;
    let mut total_free = 0u64;
    let wide_path = wide(&path.to_string_lossy());
    let ok =
        unsafe { GetDiskFreeSpaceExW(wide_path.as_ptr(), &mut free, &mut total, &mut total_free) };
    if ok == 0 {
        0
    } else {
        total_free / (1024 * 1024 * 1024)
    }
}

pub fn probe(alvr_root: Option<&std::path::Path>, headset: Option<String>) -> Hardware {
    #[cfg(windows)]
    {
        let cpu = reg_string(
            "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0",
            "ProcessorNameString",
        )
        .unwrap_or_else(|| "unknown CPU".to_owned());
        Hardware {
            cpu,
            ram_gb: memory_status(),
            gpus: gpus(),
            disk_free_gb: alvr_root.map(free_space_gb).unwrap_or(0),
            headset,
        }
    }
    #[cfg(not(windows))]
    {
        Hardware {
            cpu: "unknown".to_owned(),
            ram_gb: 0,
            gpus: Vec::new(),
            disk_free_gb: 0,
            headset,
        }
    }
}
