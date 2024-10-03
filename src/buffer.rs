use std::ffi::c_void;
use std::mem;
use std::mem::size_of;
use std::mem::zeroed;
use std::ptr;

use crate::ffi;
use crate::ffi::mem_alloc;
use crate::ffi::mem_protect;
use crate::ffi::GetSystemInfo;
use crate::ffi::MemoryBasicInformation;
use crate::ffi::SystemInfo;
use crate::ffi::VirtualQuery;

#[cfg(target_arch = "x86_64")]
const MEMORY_SLOT_SIZE: usize = 64;
#[cfg(target_arch = "x86")]
const MEMORY_SLOT_SIZE: usize = 32;

const MEMORY_BLOCK_SIZE: usize = 0x1000;

const MAX_MEMORY_RANGE: usize = 0x4000_0000;

const PAGE_EXECUTE_FLAGS: u32 = 0x10 | 0x20 | 0x40 | 0x80;

#[repr(C)]
pub struct MemorySlot {
    pub data: MemorySlotUnion,
}

#[repr(C)]
pub union MemorySlotUnion {
    pub next_p: *mut MemorySlot,
    pub buffer: [u8; MEMORY_SLOT_SIZE],
}

#[repr(C)]
pub struct MemoryBlock {
    pub next_p: *mut MemoryBlock,
    pub free_p: *mut MemorySlot,
    pub count_used: u32,
}

static mut MEMORY_BLOCKS_P: *mut MemoryBlock = ::core::ptr::null_mut();

pub unsafe fn uninitialize_buffer() {
    let mut block_p: *mut MemoryBlock = MEMORY_BLOCKS_P;

    MEMORY_BLOCKS_P = ::core::ptr::null_mut();

    while !block_p.is_null() {
        let next_p: *mut MemoryBlock = (*block_p).next_p;

        crate::ffi::VirtualFree(block_p.cast(), 0, ffi::mem_free::RELEASE);

        block_p = next_p;
    }
}

#[cfg(target_arch = "x86_64")]
unsafe fn find_prev_free_region(
    address: *mut ::core::ffi::c_void,
    min_addr_p: *mut ::core::ffi::c_void,
    allocation_granularity: u32,
) -> *mut ::core::ffi::c_void {
    let mut try_addr: usize = address as usize;

    try_addr -= try_addr % allocation_granularity as usize;

    try_addr -= allocation_granularity as usize;

    while try_addr >= min_addr_p as usize {
        let mut mbi = ::core::mem::zeroed::<crate::ffi::MemoryBasicInformation>();

        if VirtualQuery(
            try_addr as *mut ::core::ffi::c_void,
            &mut mbi,
            ::core::mem::size_of::<MemoryBasicInformation>(),
        ) == 0
        {
            break;
        }

        if mbi.state == 0x0001_0000 {
            return try_addr as *mut ::core::ffi::c_void;
        }

        if (mbi.allocation_base as usize) < allocation_granularity as usize {
            break;
        }

        try_addr = mbi.allocation_base as usize - (allocation_granularity as usize);
    }

    return ::core::ptr::null_mut();
}

#[cfg(target_arch = "x86_64")]
unsafe fn find_next_free_region(
    address: *mut ::core::ffi::c_void,
    max_addr_p: *mut ::core::ffi::c_void,
    allocation_granularity: u32,
) -> *mut ::core::ffi::c_void {
    let mut try_addr: usize = address as usize;

    try_addr -= try_addr % (allocation_granularity as usize);

    try_addr += allocation_granularity as usize;

    while try_addr <= (max_addr_p as usize) {
        let mut mbi = ::core::mem::zeroed::<crate::ffi::MemoryBasicInformation>();

        if crate::ffi::VirtualQuery(
            try_addr as *mut ::core::ffi::c_void,
            &mut mbi,
            ::core::mem::size_of_val(&mbi),
        ) == 0
        {
            break;
        }

        if mbi.state == 0x0001_0000 {
            return try_addr as *mut ::core::ffi::c_void;
        }

        try_addr = (mbi.base_address as usize) + mbi.region_size;

        try_addr += (allocation_granularity as usize) - 1;
        try_addr -= try_addr % (allocation_granularity as usize);
    }

    ::core::ptr::null_mut()
}

unsafe fn get_memory_block(origin_p: *mut core::ffi::c_void) -> *mut MemoryBlock {
    let mut block_p: *mut MemoryBlock = ptr::null_mut();

    let mut si: SystemInfo = std::mem::zeroed();

    GetSystemInfo(&mut si);

    let mut min_addr = si.lp_minimum_application_address as usize;
    let mut max_addr = si.lp_maximum_application_address as usize;

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        if (origin_p as usize) > MAX_MEMORY_RANGE
            && min_addr < (origin_p as usize) - MAX_MEMORY_RANGE
        {
            min_addr = (origin_p as usize) - MAX_MEMORY_RANGE;
        }

        if max_addr > (origin_p as usize) + MAX_MEMORY_RANGE {
            max_addr = (origin_p as usize) + MAX_MEMORY_RANGE;
        }

        max_addr -= MEMORY_BLOCK_SIZE - 1;
    }

    let mut p_temp_block = MEMORY_BLOCKS_P;
    while !p_temp_block.is_null() {
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        {
            if (p_temp_block as usize) < min_addr || (p_temp_block as usize) >= max_addr {
                p_temp_block = (*p_temp_block).next_p;
                continue;
            }
        }

        if !(*p_temp_block).free_p.is_null() {
            return p_temp_block;
        }

        p_temp_block = (*p_temp_block).next_p;
    }

    let mut p_alloc = origin_p;
    while (p_alloc as usize) >= min_addr {
        p_alloc = find_prev_free_region(
            p_alloc,
            min_addr as *mut core::ffi::c_void,
            si.dw_allocation_granularity,
        );
        if p_alloc.is_null() {
            break;
        }

        block_p = crate::ffi::VirtualAlloc(
            p_alloc,
            MEMORY_BLOCK_SIZE,
            mem_alloc::COMMIT | ffi::mem_alloc::RESERVE,
            mem_protect::EXECUTE_READ_WRITE,
        ) as *mut MemoryBlock;
        if !block_p.is_null() {
            break;
        }
    }

    if block_p.is_null() {
        let mut p_alloc = origin_p;
        while (p_alloc as usize) <= max_addr {
            p_alloc = find_next_free_region(
                p_alloc,
                max_addr as *mut core::ffi::c_void,
                si.dw_allocation_granularity,
            );
            if p_alloc.is_null() {
                break;
            }

            block_p = crate::ffi::VirtualAlloc(
                p_alloc,
                MEMORY_BLOCK_SIZE,
                mem_alloc::COMMIT | ffi::mem_alloc::RESERVE,
                mem_protect::EXECUTE_READ_WRITE,
            ) as *mut MemoryBlock;
            if !block_p.is_null() {
                break;
            }
        }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        p_block = VirtualAlloc(ptr::null_mut(), MEMORY_BLOCK_SIZE, 0x1000)
    }

    if !block_p.is_null() {
        let p_slot = (block_p as *mut MemorySlot).offset(1);
        (*block_p).free_p = p_slot;
        (*block_p).count_used = 0;
        let mut p_temp_slot = p_slot;
        while ((p_temp_slot as usize) - (block_p as usize)) <= MEMORY_BLOCK_SIZE - MEMORY_SLOT_SIZE
        {
            (*p_temp_slot).data.next_p = (*block_p).free_p;
            (*block_p).free_p = p_temp_slot;
            p_temp_slot = p_temp_slot.offset(1);
        }

        (*block_p).next_p = MEMORY_BLOCKS_P;
        MEMORY_BLOCKS_P = block_p;
    }

    block_p
}

pub unsafe fn allocate_buffer(p_origin: *mut core::ffi::c_void) -> *mut core::ffi::c_void {
    let p_slot: *mut MemorySlot;
    let p_block: *mut MemoryBlock = get_memory_block(p_origin);
    if p_block.is_null() {
        return ptr::null_mut();
    }

    p_slot = (*p_block).free_p;
    (*p_block).free_p = (*p_slot).data.next_p;
    (*p_block).count_used += 1;

    #[cfg(debug_assertions)]
    {
        std::ptr::write_bytes(p_slot as *mut u8, 0xCC, mem::size_of::<MemorySlot>());
    }

    p_slot as *mut core::ffi::c_void
}

pub unsafe fn is_executable_address(address: *mut c_void) -> bool {
    let mut mbi: MemoryBasicInformation = zeroed();

    VirtualQuery(address, &mut mbi, size_of::<MemoryBasicInformation>());

    return mbi.state == mem_alloc::COMMIT && (mbi.protect & PAGE_EXECUTE_FLAGS != 0);
}
