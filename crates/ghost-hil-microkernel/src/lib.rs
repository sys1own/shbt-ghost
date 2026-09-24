//! ghost-hil-microkernel — bare-metal C11 FFI bridge to
//! `kernel/shbt_ghost_kernel.c`: the 56-byte `SHBT-MMIO-1` register block at
//! `0x70000000`, the 2,112-byte `UnifiedStinespringFrame` SRAM arena,
//! SECDED Hamming(72,64) ECC, and the sub-2.50 ns GaN current-shunt quench
//! interlock (`tau_quench <= 2.18 ns`) with 94.20% SiC energy recovery.

/// Physical MMIO base of the SHBT-MMIO-1 register block.
pub const MMIO_BASE: u64 = 0x7000_0000;
/// Register block size in bytes.
pub const MMIO_BYTES: usize = 56;
/// UnifiedStinespringFrame size (bytes).
pub const SRAM_FRAME_BYTES: usize = 2112;
/// Active residual arena (bytes, eta_A = 10/33).
pub const ACTIVE_ARENA_BYTES: usize = 640;
/// Dark ledger QEC segment (bytes, eta_D = 23/33).
pub const DARK_LEDGER_BYTES: usize = 1472;
/// Fibonacci braid descriptor slots in the dark ledger.
pub const FIBONACCI_BRAIDS: usize = 124;
/// GaN quench interlock latency bound (ns).
pub const QUENCH_LATENCY_NS: f64 = 2.18;
/// SiC crowbar energy-recovery efficiency.
pub const SIC_RECOVERY: f64 = 0.9420;

/// Host-side mirror of `UnifiedStinespringFrame`.
#[repr(C, align(64))]
pub struct UnifiedStinespringFrame {
    /// 640-byte active residual state arena.
    pub active_arena: [u8; ACTIVE_ARENA_BYTES],
    /// 1,472-byte dark ledger QEC storage.
    pub dark_ledger: [u8; DARK_LEDGER_BYTES],
}

impl UnifiedStinespringFrame {
    /// Zeroed frame (2,112 bytes).
    pub fn new() -> Self {
        Self { active_arena: [0; ACTIVE_ARENA_BYTES], dark_ledger: [0; DARK_LEDGER_BYTES] }
    }
}

impl Default for UnifiedStinespringFrame {
    fn default() -> Self {
        Self::new()
    }
}

extern "C" {
    fn shbt_ghost_kernel_init();
    fn shbt_ghost_kernel_step();
    fn shbt_compute_secded_ecc(data: u64) -> u8;
    fn shbt_verify_and_correct_secded(data: *mut u64, stored_ecc: u8) -> i32;
    fn shbt_trigger_quench_interlock();
    fn shbt_sys_status() -> u32;
    fn shbt_avx512_givens_remapping(c: f64, s: f64);
}

/// Initialize the kernel: clear the MMIO control/status block and zero the
/// SRAM Stinespring frame.
pub fn kernel_init() {
    unsafe { shbt_ghost_kernel_init() }
}

/// Run one microkernel control step (power guardrails -> ECC -> engage).
pub fn kernel_step() {
    unsafe { shbt_ghost_kernel_step() }
}

/// SECDED Hamming(72,64) parity byte for a 64-bit word.
pub fn secded_ecc(data: u64) -> u8 {
    unsafe { shbt_compute_secded_ecc(data) }
}

/// Verify ECC; corrects single-bit errors in place. Returns 0 clean, 1
/// corrected, -1 uncorrectable.
pub fn secded_verify_correct(data: &mut u64, ecc: u8) -> i32 {
    unsafe { shbt_verify_and_correct_secded(data as *mut u64, ecc) }
}

/// Trigger the GaN current-shunt quench interlock.
pub fn trigger_quench() {
    unsafe { shbt_trigger_quench_interlock() }
}

/// Raw `REG_SYS_STATUS` bitfield.
pub fn sys_status() -> u32 {
    unsafe { shbt_sys_status() }
}

/// AVX-512 Givens rotation remap over the active residual arena.
pub fn givens_remap(c: f64, s: f64) {
    unsafe { shbt_avx512_givens_remapping(c, s) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_layout_exact() {
        assert_eq!(core::mem::size_of::<UnifiedStinespringFrame>(), SRAM_FRAME_BYTES);
        assert_eq!(ACTIVE_ARENA_BYTES, 640);
        assert_eq!(DARK_LEDGER_BYTES, 1472);
        assert_eq!(DARK_LEDGER_BYTES * 10, ACTIVE_ARENA_BYTES * 23); // eta ratios
    }

    #[test]
    fn ecc_corrects_single_bit() {
        let data = 0xDEAD_BEEF_CAFE_1234u64;
        let ecc = secded_ecc(data);
        let mut corrupt = data ^ (1 << 37);
        let r = secded_verify_correct(&mut corrupt, ecc);
        assert_eq!(r, 1);
        assert_eq!(corrupt, data);
    }

    #[test]
    fn ecc_clean_word() {
        let data = 0x0123_4567_89AB_CDEFu64;
        let ecc = secded_ecc(data);
        let mut w = data;
        assert_eq!(secded_verify_correct(&mut w, ecc), 0);
    }

    #[test]
    fn quench_sets_status() {
        kernel_init();
        trigger_quench();
        assert_ne!(sys_status() & (1 << 3), 0); // QUENCH_ACTIVE
    }
}
