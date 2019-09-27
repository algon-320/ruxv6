use super::x86::{self, EFlags};

#[derive(Default, Debug, Copy, Clone)]
pub struct CPU {
    pub apicid: u8,
}

impl CPU {
    pub const fn new() -> Self {
        CPU { apicid: 0 }
    }
}

// Must be called with interrupts disabled to avoid the caller being
// rescheduled between reading lapicid and running through the loop.
pub fn mycpu<'a>() -> Option<&'a CPU> {
    if x86::readflags().contains(EFlags::IF) {
        panic!("mycpu called with interrupts enabled\n");
    }
    unimplemented!();
    //   int apicid, i;

    //   if(readeflags()&FL_IF)
    //     panic("mycpu called with interrupts enabled\n");

    //   apicid = lapicid();
    //   // APIC IDs are not guaranteed to be contiguous. Maybe we should have
    //   // a reverse map, or reserve a register to store &cpus[i].
    //   for (i = 0; i < ncpu; ++i) {
    //     if (cpus[i].apicid == apicid)
    //       return &cpus[i];
    //   }
    //   panic("unknown apicid\n");
}
