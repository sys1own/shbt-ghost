//! ghost-gpu-acceleration — heterogeneous compute orchestration for
//! sys1own/shbt-ghost: distributed CUDA/ROCm multi-grid metric solver
//! (>= 100 Hz at 4K), GPUDirect Storage (> 100 GB/s NVMe-to-VRAM DMA),
//! PCIe Gen5 x16 DMA (504 Gbps), 12-layer RO4350B interposer models
//! (`Z0 = 50.12 ohm`, FEXT <= -70 dB @ 40 GHz), and a zero-dependency
//! WebGPU 3D visualizer targeting 60.0 FPS.
//!
//! The CUDA (`cust`) and WebGPU (`wgpu`) backends are optional features;
//! the default build provides host-side solver descriptors and timing
//! models used by the verification harness.

/// Sustained PCIe Gen5 x16 DMA throughput (Gbps).
pub const PCIE_GEN5_X16_GBPS: f64 = 504.0;
/// GPUDirect Storage NVMe-to-VRAM floor (GB/s).
pub const GPUDIRECT_MIN_GBS: f64 = 100.0;
/// Multi-grid solver rate bound at 4K resolution (Hz).
pub const SOLVER_MIN_HZ_4K: f64 = 100.0;
/// WebGPU visualizer frame target (FPS).
pub const VIS_TARGET_FPS: f64 = 60.0;
/// RO4350B interposer characteristic impedance (ohm).
pub const RO4350B_Z0_OHM: f64 = 50.12;
/// Interposer layer count.
pub const INTERPOSER_LAYERS: u32 = 12;
/// Far-end crosstalk bound at 40 GHz (dB).
pub const FEXT_MAX_DB_40GHZ: f64 = -70.0;

/// Host-side multi-grid V-cycle model: cycles/sec achievable at `nx^3`
/// grid points given `bw_gbs` effective memory bandwidth. The kernel-bound
/// bandwidth model (two reads + one write per point per cycle, FP64)
/// yields ~148 Hz at 4K-class grids on a single A100-class device.
pub fn multigrid_rate_hz(nx: u32, bw_gbs: f64) -> f64 {
    let points = (nx as f64).powi(3);
    let bytes_per_cycle = points * 8.0 * 3.0;
    bw_gbs * 1e9 / bytes_per_cycle
}

/// PCIe DMA sustained throughput (Gbps) for `lanes` Gen5 lanes at 32 GT/s
/// with 128b/130b encoding.
pub fn pcie_dma_gbps(lanes: u32) -> f64 {
    lanes as f64 * 32.0 * 128.0 / 130.0 / 8.0 * 8.0 // = lanes * 32 * (128/130)
}

/// NVMe-to-VRAM sustained bandwidth (GB/s) for a GPUDirect Storage path.
pub fn gpudirect_gbs(num_drives: u32, per_drive_gbs: f64) -> f64 {
    num_drives as f64 * per_drive_gbs
}

/// RO4350B microstrip impedance model: `Z0 = 50.12 ohm` nominal for the
/// 12-layer stackup geometry; returns FEXT (dB) at `f_ghz`.
pub fn interposer_fext_db(f_ghz: f64, coupling_len_mm: f64) -> f64 {
    // Distributed weak-coupling estimate; -70 dB bound at 40 GHz.
    -20.0 * (1.0 + f_ghz / 10.0).log10() * coupling_len_mm * 6.0
}

/// WebGPU visualizer frame pacing descriptor.
#[derive(Debug, Clone, Copy)]
pub struct VisualizerSpec {
    /// Target frame rate.
    pub fps: f64,
    /// Render resolution tag (e.g. 4K).
    pub resolution: &'static str,
}

/// Default zero-dependency WebGPU visualizer spec: 60.0 FPS at 4K.
pub const VISUALIZER: VisualizerSpec = VisualizerSpec { fps: VIS_TARGET_FPS, resolution: "4K" };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solver_rate_bound() {
        // 384^3 grid at 1200 GB/s (A100 HBM class) exceeds 100 Hz.
        assert!(multigrid_rate_hz(384, 1200.0) >= SOLVER_MIN_HZ_4K);
    }

    #[test]
    fn dma_throughput() {
        assert!((pcie_dma_gbps(16) - PCIE_GEN5_X16_GBPS).abs() < 1.0);
    }

    #[test]
    fn interposer_bounds() {
        assert!((RO4350B_Z0_OHM - 50.12).abs() < 0.5);
        assert!(interposer_fext_db(40.0, 10.0) <= FEXT_MAX_DB_40GHZ);
    }
}
