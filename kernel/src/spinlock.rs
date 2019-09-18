use super::proc::CPU;
use super::x86;

pub struct SpinLock {
    pub locked: bool,

    // for debugging
    name: Option<*const u8>,
    cpu: CPU,
    pcs: [u32; 10],
}

impl SpinLock {
    pub fn new() -> Self {
        SpinLock {
            locked: false,
            name: None,
            cpu: CPU::default(),
            pcs: [0; 10],
        }
    }
    pub fn with_name(name: *const u8) -> Self {
        SpinLock {
            locked: false,
            name: Some(name),
            cpu: CPU::default(),
            pcs: [0; 10],
        }
    }

    // Acquire the lock.
    // Loops (spins) until the lock is acquired.
    // Holding a lock for a long time may cause
    // other CPUs to waste time spinning to acquire it.
    pub fn acquire(&mut self) {
        unimplemented!()
    }

    // Release the lock.
    pub fn release(&mut self) {
        unimplemented!()
    }

    // Record the current call stack in pcs[] by following the %ebp chain.
    pub fn getcallerpcs(&mut self, v: *const ()) {
        unimplemented!()
    }

    // Check whether this cpu is holding the lock.
    pub fn holding() -> bool {
        unimplemented!()
    }
}

// Pushcli/popcli are like cli/sti except that they are matched:
// it takes two popcli to undo two pushcli.  Also, if interrupts
// are off, then pushcli, popcli leaves them off.
fn pushcli() {
    let eflags = x86::readflags();
    x86::cli();
    unimplemented!()
}
fn popcli() {
    unimplemented!()
}

// void
// pushcli(void)
// {
//   int eflags;

//   eflags = readeflags();
//   cli();
//   if(mycpu()->ncli == 0)
//     mycpu()->intena = eflags & FL_IF;
//   mycpu()->ncli += 1;
// }

// void
// popcli(void)
// {
//   if(readeflags()&FL_IF)
//     panic("popcli - interruptible");
//   if(--mycpu()->ncli < 0)
//     panic("popcli");
//   if(mycpu()->ncli == 0 && mycpu()->intena)
//     sti();
// }
