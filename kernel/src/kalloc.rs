// Initialization happens in two phases.
// 1. main() calls kinit1() while still using entrypgdir to place just
// the pages mapped by entrypgdir on free list.
// 2. main() calls kinit2() with the rest of the physical pages
// after installing a full page table that maps them on all cores.

fn kinit1() {}

// void
// kinit1(void *vstart, void *vend)
// {
//   initlock(&kmem.lock, "kmem");
//   kmem.use_lock = 0;
//   freerange(vstart, vend);
// }

// void
// kinit2(void *vstart, void *vend)
// {
//   freerange(vstart, vend);
//   kmem.use_lock = 1;
// }
