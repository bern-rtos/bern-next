MEMORY {
    FLASH : ORIGIN = 0x08000000, LENGTH = 512K
    RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* Align stacks to double word see:
   https://community.arm.com/developer/ip-products/processors/f/cortex-m-forum/6344/what-is-the-meaning-of-a-64-bit-aligned-stack-pointer-address */
SECTIONS {
    .task_stack (NOLOAD) : ALIGN(8) {
        *(.task_stack);
        . = ALIGN(8);
    } > RAM
} INSERT AFTER .bss;

SECTIONS {
    /*### .shared */
    .shared : ALIGN(4)
    {
        . = ALIGN(4);
        __sshared = .;
        *(.shared);
        . = ALIGN(4);
        __eshared = .;
    } > RAM
    __sishared = LOADADDR(.shared);
} INSERT AFTER .task_stack
