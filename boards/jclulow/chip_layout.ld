/* Memory Spaces Definitions, 512K flash, 1024K ram */
MEMORY
{
  rom (rx)  : ORIGIN = 0x00000000, LENGTH = 0x00040000
  prog (rx) : ORIGIN = 0x00040000, LENGTH = 0x00040000
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00100000
}

MPU_MIN_ALIGN = 8K; /* XXX */
