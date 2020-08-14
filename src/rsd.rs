use core::fmt::Write;
use crate::printer;

#[derive(Debug)]
#[repr(C, packed)]
pub struct RSDP {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    root_sdt_phys_addr: u32,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct RSDT {
    header: ACPISDTHeader,
    other_sdt_ptr: u32,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct ACPISDTHeader {
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

#[derive(Debug)]
pub struct MADTIter<'a> {
    madt: &'a MADT,
    pos: usize,
    bytes_remaining: usize,
    curr_addr: usize
}

impl<'a> MADTIter<'a> {
    pub fn new(madt: &'a MADT) -> Self {
        MADTIter {
            madt: madt,
            pos: 0,
            bytes_remaining: madt.header.length as usize - core::mem::size_of::<MADT>(),
            curr_addr: madt.entry_start()
        }
    }
}

impl<'a> Iterator for MADTIter<'a>   {
    type Item = MADTEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes_remaining <= 0 {
            return None
        }
        let mut curr = self.curr_addr as *const MADTEntryHeader;
        unsafe {
            self.bytes_remaining -= (*curr).len as usize;
            write!(&mut printer::Printer, "bytes remaining: {}", self.bytes_remaining);
            self.curr_addr += (*curr).len as usize;
            let typ = (*curr).typ;
            match typ {
                0 => Some(MADTEntry::APICEntry(&*(curr as *const APICEntry))),
                1 => Some(MADTEntry::IOAPICEntry(&*(curr as *const IOAPICEntry))),
                2 | 4 | 5 => Some(MADTEntry::OtherEntry(typ)),
                _ => panic!("unknown MADT entry {}", (typ)),

            }
        }

    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct MADT {
    header: ACPISDTHeader,
    local_addr: u32,
    flags: u32,
}

impl MADT {
    pub unsafe fn iter(&self) -> MADTIter {
        MADTIter::new(&self)
    }

    pub fn entry_start(&self) -> usize {
        self as *const MADT as usize + core::mem::size_of::<MADT>()
    }
}

#[derive(Debug)]
pub enum MADTEntry<'a> {
    APICEntry(&'a APICEntry),
    IOAPICEntry(&'a IOAPICEntry),
    OtherEntry(u8),
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct MADTEntryHeader {
    typ: u8,
    len: u8
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct APICEntry {
    header: MADTEntryHeader,
    processor_id: u8,
    apic_id: u8,
    flags: u32
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct IOAPICEntry {
    header: MADTEntryHeader,
    apic_id: u8,
    reserved: u8,
    address: u32,
    base: u32,
}

pub(crate) unsafe fn init(phys_mem_offset: u64) {
    let rsd = match find_rsd(phys_mem_offset) {
        Some(rsd) => rsd,
        None => panic!("no rsd found")
    };
    let madt = match find_sdt(rsd, "APIC", phys_mem_offset) {
        Some(rsd) => rsd as *const MADT,
        None => panic!("no madt found")
    };
    writeln!(&mut printer::Printer, "lapic addr: {:x}", (*madt).local_addr);
    for entry in (*madt).iter() {
        write!(&mut printer::Printer, "apic entry: {:?}", entry);
    }
    

    
}

pub(crate) unsafe fn find_rsd_in_range(start: u64, end: u64) -> Option<*const RSDP> {
    for i in (start..end).step_by(16) {
        let rsd = i as *const RSDP;
        let sig: &[u8] = &((*rsd).signature);
        if sig == "RSD PTR ".as_bytes() {
            write!(&mut printer::Printer, "sig: {}, found: {:?}", core::str::from_utf8_unchecked(sig), *rsd);
            return Some(rsd)
        }
    }
    return None
}

pub(crate) fn find_rsd(phys_mem_offset: u64) -> Option<*const RSDP> {
    let rsd = unsafe {find_rsd_in_range(phys_mem_offset + 0x000E0000,phys_mem_offset + 0x00100000)};
    match rsd {
        Some(rsd) => Some(rsd),
        None => {
            let base = unsafe {(*(0x40E as *const u16) as u64) << 4};
            match unsafe {find_rsd_in_range(base, base + (1 << 10))} {
                Some(rsd) => Some(rsd),
                None => None
            }
        }
    }
}

pub(crate) unsafe fn find_sdt(rsdp: *const RSDP, signature: &str, phys_mem_offset: u64) -> Option<*const ACPISDTHeader> {
    let rsdt = ((*rsdp).root_sdt_phys_addr as u64 + phys_mem_offset) as *const RSDT;
    let entries = ((*rsdt).header.length as usize - core::mem::size_of::<ACPISDTHeader>()) / 4;
    let other_sdts = core::slice::from_raw_parts(&(*rsdt).other_sdt_ptr as *const u32, entries);
    for sdt_addr in other_sdts {
        let sdt = ((*sdt_addr as u64) + phys_mem_offset) as *const ACPISDTHeader;
        if (*sdt).signature == signature.as_bytes() {
            return Some(sdt)
        }
    }
    None
}

