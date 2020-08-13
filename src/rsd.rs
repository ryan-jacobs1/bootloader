use core::fmt::Write;
use crate::printer;

#[derive(Debug)]
#[repr(C, packed)]
pub struct RSD {
    signature: [u8; 8],
    checksum: u8,
    oemID: [u8; 6],
    revision: u8,
    root_sdt: *const SDT,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct SDT {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oemID: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

pub(crate) unsafe fn find_rsd_in_range(start: u64, end: u64) -> Option<*const RSD> {
    for i in (start..end).step_by(16) {
        let rsd = i as *const RSD;
        let sig: &[u8] = &((*rsd).signature);
        if sig == "RSD PTR ".as_bytes() {
            write!(&mut printer::Printer, "sig: {}, found: {:?}", core::str::from_utf8_unchecked(sig), *rsd);
            return Some(rsd)
        }
    }
    return None
}

pub(crate) fn find_rsd(phys_mem_offset: u64) -> *const RSD {
    let rsd = unsafe {find_rsd_in_range(phys_mem_offset + 0x000E0000,phys_mem_offset + 0x00100000)};
    match rsd {
        Some(rsd) => rsd,
        None => {
            let base = unsafe {(*(0x40E as *const u16) as u64) << 4};
            match unsafe {find_rsd_in_range(base, base + (1 << 10))} {
                Some(rsd) => rsd,
                None => panic!("No RSD found")
            }
        }
    }
}

