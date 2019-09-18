use super::spinlock::SpinLock;
use super::x86;

static mut lock: SpinLock = SpinLock::new();
static mut locking: u32 = 0;

pub fn init() {
    lock
}
pub fn print(&mut self, s: &str) {
    unimplemented!();
}

/*

void
panic(char *s)
{
  int i;
  uint pcs[10];

  cli();
  cons.locking = 0;
  // use lapiccpunum so that we can call panic from mycpu()
  cprintf("lapicid %d: panic: ", lapicid());
  cprintf(s);
  cprintf("\n");
  getcallerpcs(&s, pcs);
  for(i=0; i<10; i++)
    cprintf(" %p", pcs[i]);
  panicked = 1; // freeze other CPU
  for(;;)
    ;
}
*/
