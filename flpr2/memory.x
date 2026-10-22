/*
 * just follow zephyr conventions
 * 4k  @ 0x20066C00 for ipc
 * 96k @ 0x20067C00 for flpr ram
*/
MEMORY
{
    RAM : ORIGIN = 0x20067C00, LENGTH = 96K
    RAM_SHARED : ORIGIN = 0x20066C00, LENGTH = 4K
}

SECTIONS
{
    /* shared memory, initialized by the application core */
    .shared_mem (NOLOAD) : {
        __shared_mem = .;
    } > RAM_SHARED
}

REGION_ALIAS("REGION_TEXT", RAM);
REGION_ALIAS("REGION_RODATA", RAM);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);
