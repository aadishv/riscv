MEMORY
{
    FLASH : ORIGIN = 0x00000000, LENGTH = 2036K
    RAM   : ORIGIN = 0x20000000, LENGTH = 0x66c00
    RAM_SHARED : ORIGIN = 0x20066C00, LENGTH = 4K
}

SECTIONS
{
    /* shared memory, initialized by the application core */
    .shared_mem (NOLOAD) : {
        __shared_mem = .;
    } > RAM_SHARED
}