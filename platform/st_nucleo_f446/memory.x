MEMORY {
    FLASH : ORIGIN = 0x08000000, LENGTH = 512K
    RAM : ORIGIN = 0x20000000, LENGTH = 127K
}

SECTIONS {
    /*### .kernel */
    _kernel_size = 2K;

    .kernel : ALIGN(4)
    {
        /* Kernel static memory */
        . = ALIGN(4);
        __smkernel = .;
        *(.kernel);
        *(.kernel.process);
        . = ALIGN(4);
        __emkernel = .;

        /* Kernel heap */
        . = ALIGN(4);
        __shkernel = .;
        . = __smkernel + _kernel_size;
        __ehkernel = .;

        ASSERT(__emkernel <= __ehkernel, "Error: No room left in bern kernel.");
    } > RAM
    __sikernel = LOADADDR(.kernel);
} INSERT AFTER .bss;