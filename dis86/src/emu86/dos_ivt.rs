
// CPU Exceptions
pub const DOS_IVT_DIVIDE_BY_ZERO:          u8 = 0x00;
pub const DOS_IVT_SINGLE_STEP:             u8 = 0x01;
pub const DOS_IVT_NMI:                     u8 = 0x02;
pub const DOS_IVT_BREAKPOINT:              u8 = 0x03;
pub const DOS_IVT_OVERFLOW:                u8 = 0x04;
pub const DOS_IVT_BOUND_EXCEEDED:          u8 = 0x05;
pub const DOS_IVT_INVALID_OPCODE:          u8 = 0x06;
pub const DOS_IVT_COPROC_UNAVAILABLE:      u8 = 0x07;

// Hardware IRQs (first PIC, IRQ0-IRQ7)
pub const DOS_IVT_IRQ0_TIMER:              u8 = 0x08;
pub const DOS_IVT_IRQ1_KEYBOARD:           u8 = 0x09;
pub const DOS_IVT_IRQ2_CASCADE:            u8 = 0x0A;
pub const DOS_IVT_IRQ3_COM2:               u8 = 0x0B;
pub const DOS_IVT_IRQ4_COM1:               u8 = 0x0C;
pub const DOS_IVT_IRQ5_LPT2:               u8 = 0x0D;
pub const DOS_IVT_IRQ6_FLOPPY:             u8 = 0x0E;
pub const DOS_IVT_IRQ7_LPT1:               u8 = 0x0F;

// BIOS Services
pub const DOS_IVT_VIDEO:                   u8 = 0x10;
pub const DOS_IVT_EQUIPMENT_LIST:          u8 = 0x11;
pub const DOS_IVT_MEMORY_SIZE:             u8 = 0x12;
pub const DOS_IVT_DISK:                    u8 = 0x13;
pub const DOS_IVT_SERIAL:                  u8 = 0x14;
pub const DOS_IVT_MISC_SYSTEM:             u8 = 0x15;
pub const DOS_IVT_KEYBOARD:                u8 = 0x16;
pub const DOS_IVT_PRINTER:                 u8 = 0x17;
pub const DOS_IVT_ROM_BASIC:               u8 = 0x18;
pub const DOS_IVT_BOOTSTRAP:               u8 = 0x19;
pub const DOS_IVT_TIME_OF_DAY:             u8 = 0x1A;
pub const DOS_IVT_CTRL_BREAK:              u8 = 0x1B;
pub const DOS_IVT_TIMER_TICK:              u8 = 0x1C;

// BIOS Data Pointers
pub const DOS_IVT_VIDEO_PARAMS:            u8 = 0x1D;
pub const DOS_IVT_FLOPPY_PARAMS:           u8 = 0x1E;
pub const DOS_IVT_GRAPHICS_CHARS:          u8 = 0x1F;

// DOS
pub const DOS_IVT_TERMINATE:               u8 = 0x20;
pub const DOS_IVT_DOS_FUNCTIONS:           u8 = 0x21;
pub const DOS_IVT_TERMINATE_ADDRESS:       u8 = 0x22;
pub const DOS_IVT_CTRL_C_HANDLER:          u8 = 0x23;
pub const DOS_IVT_CRITICAL_ERROR:          u8 = 0x24;
pub const DOS_IVT_ABS_DISK_READ:           u8 = 0x25;
pub const DOS_IVT_ABS_DISK_WRITE:          u8 = 0x26;
pub const DOS_IVT_TSR:                     u8 = 0x27;
pub const DOS_IVT_IDLE:                    u8 = 0x28;
pub const DOS_IVT_FAST_CON_OUT:            u8 = 0x29;
pub const DOS_IVT_NETWORK:                 u8 = 0x2A;
pub const DOS_IVT_COMMAND_EXECUTE:         u8 = 0x2E;
pub const DOS_IVT_MULTIPLEX:               u8 = 0x2F;

// Hardware IRQs (second PIC, IRQ8-IRQ15)
pub const DOS_IVT_IRQ8_RTC:                u8 = 0x70;
pub const DOS_IVT_IRQ9_REDIRECT:           u8 = 0x71;
pub const DOS_IVT_IRQ10:                   u8 = 0x72;
pub const DOS_IVT_IRQ11:                   u8 = 0x73;
pub const DOS_IVT_IRQ12_PS2_MOUSE:         u8 = 0x74;
pub const DOS_IVT_IRQ13_FPU:               u8 = 0x75;
pub const DOS_IVT_IRQ14_IDE_PRIMARY:       u8 = 0x76;
pub const DOS_IVT_IRQ15_IDE_SECONDARY:     u8 = 0x77;

// Other
pub const DOS_IVT_MOUSE:                   u8 = 0x33;
pub const DOS_IVT_HDD_PARAMS_0:            u8 = 0x41;
pub const DOS_IVT_HDD_PARAMS_1:            u8 = 0x46;
pub const DOS_IVT_RTC_ALARM:               u8 = 0x4A;
pub const DOS_IVT_EMS:                     u8 = 0x67;
