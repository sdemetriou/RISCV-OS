#risc-v 
**Physical Page Number:** The supervisor physical address of a page table / 4 KiB

why?

Well, if each page is 4096 Bytes: 

By the name, **(HYPOTHESIS)** this is likely meant to define the number of page table entries in the page table.

In that case, the only way: addr of page table / 4 KiB can yield the number of page table entries, is if address `0x0` is where the page table entries begin, right?

If **(HYPOTHESIS)** this is meant to define an index of a page table, then:

say arbitrarily that the root page table sits at `0x10000000`

Therefore `0x10000000 / 0x1000 = 0x10000` = `65536`

If we start with `0x80200000` then:
`0x80200000 / 0x1000 = 0x80200` = `524800`

We are essentially taking an address space from 0x0 to 0xwhatever, and we are dividing it into chunks of 4 KiB each. so the higher the address space that the root page table starts at, the more chunks we end up creating.

This is nonsensical to me. Why not take end address of the page table, and subtract from it the start address of the page table, AND THEN do the division?

I'm missing something. Will continue reading the Sv32 section for now. Perhaps I'm over-thinking something without the necessary context.
**(CONTEXT REQUIRED)**