/* Same FLPR region Zephyr uses on the nRF54LM20A: the top 96K of RAM01, ending just below the VPR
 * saved context at 0x2007FD40. The 4K below it (0x20066C00) is app<->FLPR IPC memory, so the app
 * core's RAM must end at 0x20066C00 once both cores run. */
MEMORY
{
    RAM : ORIGIN = 0x20067C00, LENGTH = 96K
}

REGION_ALIAS("REGION_TEXT", RAM);
REGION_ALIAS("REGION_RODATA", RAM);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);
