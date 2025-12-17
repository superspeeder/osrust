use bitflags::bitflags;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CPU_FLAGS: CPUFlags = {
        let res = unsafe { core::arch::x86_64::__cpuid(1) };
        CPUFlags::from_bits_retain(res.ecx as u64 | ((res.edx as u64) << 32))
    };
}

lazy_static! {
    pub static ref CPU_FLAGS_EXT: EXTCPUFlags = {
        let res = unsafe { core::arch::x86_64::__cpuid(1) };
        EXTCPUFlags::from_bits_retain(res.edx as u64 | ((res.ecx as u64) << 32))
    };
}

bitflags! {
    pub struct CPUFlags : u64 {
        const SSE3 = 1 << 0;
        const MONITOR = 1 << 3;
        const DS_CPL = 1 << 4;
        const VMX = 1 << 5;
        const SMX = 1 << 6;
        const EST = 1 << 7;
        const TM2 = 1 << 8;
        const SSSE3 = 1 << 9;
        const CNXT_ID = 1 << 10;
        const CMPXCHG16B = 1 << 13;
        const xTPR_UPDATE = 1 << 14;
        const PDCM = 1 << 15;
        const DCA = 1 << 18;
        const SSE4_1 = 1 << 19;
        const SSE4_2 = 1 << 20;
        const x2APIC = 1 << 21;
        const MOVBE = 1 << 22;
        const POPCNT = 1 << 23;
        const XSAVE = 1 << 26;
        const OSXSAVE = 1 << 27;

        const x87 = 1 << (32 + 0);
        const VME = 1 << (32 + 1);
        const DE = 1 << (32 + 2);
        const PSE = 1 << (32 + 3);
        const TSC = 1 << (32 + 4);
        const MSR = 1 << (32 + 5);
        const PAE = 1 << (32 + 6);
        const MCE = 1 << (32 + 7);
        const CX8 = 1 << (32 + 8);
        const APIC = 1 << (32 + 9);
        const SEP = 1 << (32 + 11);
        const MTRR = 1 << (32 + 12);
        const PGE = 1 << (32 + 13);
        const MCA = 1 << (32 + 14);
        const CMOV = 1 << (32 + 15);
        const PAT = 1 << (32 + 16);
        const PSE36 = 1 << (32 + 17);
        const PSN = 1 << (32 + 18);
        const CLFSH = 1 << (32 + 19);
        const DS = 1 << (32 + 21);
        const ACPI = 1 << (32 + 22);
        const MMX = 1 << (32 + 23);
        const FXSR = 1 << (32 + 24);
        const SSE = 1 << (32 + 25);
        const SSE2 = 1 << (32 + 26);
        const SS = 1 << (32 + 27);
        const HTT = 1 << (32 + 28);
        const TM = 1 << (32 + 29);
        const PBE = 1 << (32 + 31);
    }
}

bitflags! {
    pub struct EXTCPUFlags : u64 {
        #[deprecated]
        const FPU = 1 << 0;

        #[deprecated]
        const VME = 1 << 1;

        #[deprecated]
        const DE = 1 << 2;

        #[deprecated]
        const PSE = 1 << 3;

        #[deprecated]
        const TSC = 1 << 4;

        #[deprecated]
        const MSR = 1 << 5;

        #[deprecated]
        const PAE = 1 << 6;

        #[deprecated]
        const MCE = 1 << 7;

        #[deprecated]
        const CX8 = 1 << 8;

        #[deprecated]
        const APIC = 1 << 9;

        const SYSCALL = 1 << 11;

        #[deprecated]
        const MTRR = 1 << 12;

        #[deprecated]
        const PGE = 1 << 13;

        #[deprecated]
        const MCA = 1 << 14;

        #[deprecated]
        const CMOV = 1 << 15;

        #[deprecated]
        const PAT = 1 << 16;

        #[deprecated]
        const PSE36 = 1 << 17;

        const ECC = 1 << 19;
        const NX = 1 << 20;
        const MMXEXT = 1 << 22;

        #[deprecated]
        const MMX = 1 << 23;

        #[deprecated]
        const FXSR = 1 << 24;

        const FXSR_OPT = 1 << 25;
        const PDPE1GB = 1 << 26;
        const RDTSCP = 1 << 27;
        const LM = 1 << 29;

        const LAHF_LM = 1 << (32 + 0);
        const CMP_LEGACY = 1 << (32 + 1);
        const SVM = 1 << (32 + 2);
        const EXTAPIC = 1 << (32 + 3);
        const CR8_LEGACY = 1 << (32 + 4);
        const ABM = 1 << (32 + 5);
        const SSE4A = 1 << (32 + 6);
        const MISALIGNSSE = 1 << (32 + 7);
        const PREFETCH = 1 << (32 + 8);
        const OSVW = 1 << (32 + 9);
        const IBS = 1 << (32 + 10);
        const XOP = 1 << (32 + 11);
        const SKINIT = 1 << (32 + 12);
        const WDT = 1 << (32 + 13);
        const LWP = 1 << (32 + 15);
        const FMA4 = 1 << (32 + 16);
        const TCE = 1 << (32 + 17);
        const NODEID_MSR = 1 << (32 + 19);
        const TBM = 1 << (32 + 21);
        const TOPOEXT = 1 << (32 + 22);
        const PERFCTR_CORE = 1 << (32 + 23);
        const PERFCTR_NB = 1 << (32 + 24);
        const STREAMPERFMON = 1 << (32 + 25);
        const DBX = 1 << (32 + 26);
        const PERFTSC = 1 << (32 + 27);
        const PCX_L2I = 1 << (32 + 28);
        const MONITORX = 1 << (32 + 29);
        const ADDR_MASK_EXT = 1 << (32 + 30);
    }
}
