type BOOL = i32;

#[link(name = "Kernel32")]
extern "system" {

    pub(crate) fn GetSystemInfo(lpSystemInfo: *mut SystemInfo);

    // internal
    pub(crate) fn VirtualAlloc(
        lpAddress: *const ::core::ffi::c_void,
        dwSize: usize,
        flAllocationType: u32,
        flProtect: u32,
    ) -> *mut ::core::ffi::c_void;

    pub(crate) fn VirtualFree(
        lpAddress: *mut ::core::ffi::c_void,
        dwSize: usize,
        dwFreeType: u32,
    ) -> BOOL;

    pub(crate) fn VirtualQuery(
        lpAddress: *const ::core::ffi::c_void,
        lpBuffer: *mut MemoryBasicInformation,
        dwLength: usize,
    ) -> usize;

}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MemoryBasicInformation {
    pub(crate) base_address: *mut ::core::ffi::c_void,
    pub(crate) allocation_base: *mut ::core::ffi::c_void,
    pub(crate) allocation_protect: u32,
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    pub(crate) partition_id: u16,
    pub(crate) region_size: usize,
    pub(crate) state: u32,
    pub(crate) protect: u32,
    pub(crate) type_: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct SystemInfo {
    pub(crate) dummy_union: SystemInfoDummyUnion,
    pub(crate) dw_page_size: u32,
    pub(crate) lp_minimum_application_address: *mut ::core::ffi::c_void,
    pub(crate) lp_maximum_application_address: *mut ::core::ffi::c_void,
    pub(crate) dw_active_processor_mask: usize,
    pub(crate) dw_number_of_processors: u32,
    pub(crate) dw_processor_type: u32,
    pub(crate) dw_allocation_granularity: u32,
    pub(crate) w_processor_level: u16,
    pub(crate) w_processor_revision: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) union SystemInfoDummyUnion {
    pub(crate) dw_oem_id: u32,
    pub(crate) dummy_struct: SystemInfoDummyStruct,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SystemInfoDummyStruct {
    pub(crate) w_processor_architecture: u16,
    pub(crate) w_reserved: u16,
}

#[doc = "Constant collection"]
pub mod mem_protect {
    pub const ENCLAVE_DECOMMIT: u32 = 0x1000_0000;

    pub const ENCLAVE_THREAD_CONTROL: u32 = 0x8000_0000;

    pub const ENCLAVE_UNVALIDATED: u32 = 0x2000_0000;

    pub const EXECUTE: u32 = 0x10;

    pub const EXECUTE_READ: u32 = 0x20;

    pub const EXECUTE_READ_WRITE: u32 = 0x40;

    pub const EXECUTE_WRITECOPY: u32 = 0x80;

    pub const GUARD: u32 = 0x100;

    pub const NOACCESS: u32 = 0x01;

    pub const NOCACHE: u32 = 0x200;

    pub const READONLY: u32 = 0x02;

    pub const READ_WRITE: u32 = 0x04;

    pub const TARGETS_INVALID: u32 = 0x4000_0000;

    pub const TARGETS_NO_UPDATE: u32 = 0x4000_0000;

    pub const WRITECOMBINE: u32 = 0x400;

    pub const WRITECOPY: u32 = 0x08;
}

#[doc = "Constant collection"]
pub mod mem_alloc {
    pub const COMMIT: u32 = 0x0000_1000;

    pub const LARGE_PAGES: u32 = 0x2000_0000;

    pub const PHYSICAL: u32 = 0x0040_0000;

    pub const RESERVE: u32 = 0x0000_2000;

    pub const RESET: u32 = 0x0008_0000;

    pub const RESET_UNDO: u32 = 0x0100_0000;

    pub const TOP_DOWN: u32 = 0x0010_0000;

    pub const WRITE_WATCH: u32 = 0x0020_0000;
}

#[doc = "Constant collection"]
pub mod mem_free {
    pub const COALESCE_PLACEHOLDERS: u32 = 0x0000_0001;

    pub const DECOMMIT: u32 = 0x0000_4000;

    pub const PRESERVE_PLACEHOLDER: u32 = 0x0000_0002;

    pub const RELEASE: u32 = 0x00008000;
}
