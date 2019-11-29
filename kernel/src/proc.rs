use super::file;
use super::lapic;
use super::mmu;
use super::mp;
use super::param;
use super::utils::address::{vaddr, vaddr_raw};
use super::vm;
use super::x86::{self, EFlags};

#[derive(Debug)]
pub struct CPU {
    pub id: usize,
    pub apicid: u8,                         // Local APIC ID
    pub scheduler: *const context,          // swtch() here to enter scheduler
    pub ts: mmu::taskstate,                 // Used by x86 to find stack for interrupt
    pub gdt: [mmu::SegDesc; mmu::seg::NUM], // x86 global descriptor table
    pub started: bool,                      // Has the CPU started?
    pub ncli: i32,                          // Depth of pushcli nesting.
    pub intena: bool,                       // Were interrupts enabled before pushcli?
    pub proc: *const proc,                  // The process running on this cpu or null
}

impl CPU {
    pub fn new(id: usize, apicid: u8) -> Self {
        CPU {
            id,
            apicid,
            scheduler: core::ptr::null(),
            ts: mmu::taskstate::new(),
            gdt: [mmu::SegDesc::zero(); mmu::seg::NUM],
            started: false,
            ncli: 0,
            intena: false,
            proc: core::ptr::null(),
        }
    }
    pub fn cpuid(&self) -> usize {
        self.id
    }
}

// Saved registers for kernel context switch
// Don't need to save all the segment registers (%cs, etc),
// because they are constant across kernel contexts.
// Don't need to save %eax, %ecx, %edx, because the
// x86 convention is that the caller has saved them.
// Contexts are stored at the bottom of the stack they
// describe; the stack pointer is the address of the context.
// The layout of the context matches the layout of the stack in swtch,S
// at the "Switch stacks" comment. Switch doesn't save eip explicitly,
// but it is on the stack and allocproc() manipulates it.
#[derive(Debug)]
pub struct context {
    edi: u32,
    esi: u32,
    ebx: u32,
    ebp: u32,
    eip: u32,
}

enum procstate {
    UNUSED,
    EMBRYO,
    SLEEPING,
    RUNNABLE,
    RUNNING,
    ZOMBIE,
}

// Per-process status
pub struct proc {
    sz: usize,                          // Size of process memory (bytes)
    pgdir: *const vm::PageDirEntry,     // Page table
    kstack: *const u8,                  // Bottom of kernel stack for this process
    state: procstate,                   // Process state
    pid: i32,                           // Process ID
    parent: *const proc,                // Parent process
    tf: *const x86::trapframe,          // Trap frame for current syscall
    context: *const context,            // swtch() here to run process
    chan: vaddr,                        // If non-zero, sleeping on chan
    killed: bool,                       // If true, have been killed
    ofile: [file::File; param::NOFILE], // Open files
    cwd: *const file::Inode,            // Current directory
    name: [u8; 16],                     // Process name (debugging)
}

pub fn pinit() {
    // initilock
}

// Must be called with interrupts disabled to avoid the caller being
// rescheduled between reading lapicid and running through the loop.
pub fn mycpu() -> &'static CPU {
    if x86::readflags().contains(EFlags::IF) {
        panic!("mycpu called with interrupts enabled\n");
    }

    let apicid = unsafe { lapic::lapicid() };
    // APIC IDs are not guaranteed to be contiguous. Maybe we should have
    // a reverse map, or reserve a register to store &cpus[i].
    unsafe {
        for c in mp::CPU_ARRAY.slice().iter() {
            if let Some(c) = c {
                if c.apicid == apicid {
                    return c;
                }
            }
        }
    }
    panic!("unknown apicid");
}
