MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* TODO Adjust these memory regions to match your device memory layout */
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* You may want to use this variable to locate the call stack and static
   variables in different memory regions. Below is shown the default value */
/* _stack_start = ORIGIN(RAM) + LENGTH(RAM); */

/* You can use this symbol to customize the location of the .text section */
/* If omitted the .text section will be placed right after the .vector_table
   section */
/* This is required only on microcontrollers that store some configuration right
   after the vector table */
/* _stext = ORIGIN(FLASH) + 0x400; */

/* Example of putting non-initialized variables into custom RAM locations. */
/* This assumes you have defined a region RAM2 above, and in the Rust
   sources added the attribute `#[link_section = ".ram2bss"]` to the data
   you want to place there. */

/* Align stacks to double word see:
   https://community.arm.com/developer/ip-products/processors/f/cortex-m-forum/6344/what-is-the-meaning-of-a-64-bit-aligned-stack-pointer-address */
/* Note that the section will not be zero-initialized by the runtime! */
   SECTIONS {
     .taskstack (NOLOAD) : ALIGN(8) {
       *(.taskstack);
       . = ALIGN(8);
     } > RAM
   } INSERT AFTER .bss;

   SECTIONS {
        .test_secret (NOLOAD) : ALIGN(4) {
          *(.test_secret);
          . = ALIGN(4);
        } > RAM
      } INSERT AFTER .bss;