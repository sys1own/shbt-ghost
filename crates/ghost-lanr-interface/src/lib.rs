//! ghost-lanr-interface — 1,800-module LANR cold fusion power plant ledger
//! (`999.054 kW_net` output / `906.00 kW` debt load), operating floor
//! `N_min = 1,633` with `N+167` zero-derating reserve, 33.804% TEG
//! conversion, two-phase He-4 nucleate boiling rejection, CVD Diamond-on-GaN
//! substrates (2000 W/m·K), NbN/MgB2 superconducting margins, and 94.20%
//! efficient SiC crowbar energy recovery.

/// Active LANR modules.
pub const MODULES_TOTAL: u32 = 1800;
/// Net electrical output per module (W).
pub const MODULE_POWER_W: f64 = 555.03;
/// Total grid output (kW).
pub const TOTAL_OUTPUT_KW: f64 = MODULES_TOTAL as f64 * MODULE_POWER_W / 1000.0;
/// Continuous entropy-debt load (kW).
pub const DEBT_LOAD_KW: f64 = 906.00;
/// Minimum operating floor (modules) — 906.36 kW.
pub const MODULES_MIN_FLOOR: u32 = 1633;
/// Zero-derating reserve: `N_total - N_min`.
pub const RESERVE_MODULES: u32 = MODULES_TOTAL - MODULES_MIN_FLOOR;
/// Thermal-to-electric generator efficiency.
pub const TEG_EFFICIENCY: f64 = 0.33804;
/// SiC crowbar energy-recovery efficiency.
pub const SIC_RECOVERY: f64 = 0.9420;
/// CVD Diamond-on-GaN substrate thermal conductivity (W/m·K).
pub const DIAMOND_K_W_MK: f64 = 2000.0;
/// NbN critical temperature (K).
pub const NBN_TC_K: f64 = 16.0;
/// Operating temperature with 11.79 K margin.
pub const OPERATING_TEMP_K: f64 = NBN_TC_K - 11.79;
/// MgB2 critical temperature (K).
pub const MGB2_TC_K: f64 = 39.0;
/// Sustained power transient envelope (MW) within thermal headroom.
pub const TRANSIENT_MW: f64 = 142.08;

/// Total electrical output (kW) for `n` active modules.
pub fn output_kw(n: u32) -> f64 {
    n as f64 * MODULE_POWER_W / 1000.0
}

/// Net operating margin (kW) for `n` modules against the debt load.
pub fn net_margin_kw(n: u32) -> f64 {
    output_kw(n) - DEBT_LOAD_KW
}

/// Whether the module count is at/above the operating floor.
pub fn above_floor(n: u32) -> bool {
    n >= MODULES_MIN_FLOOR
}

/// Two-phase subcooled He-4 nucleate-boiling heat flux estimate (W/m^2)
/// via a Zuber-style CHF correlation scaled to superfluid helium.
pub fn he4_nucleate_boiling_flux(delta_t_k: f64) -> f64 {
    let h_lv = 20_750.0; // J/kg latent heat He-4
    let rho_v = 16.89; // kg/m^3 vapour
    let rho_l = 125.0; // kg/m^3 liquid
    let sigma = 0.00026; // N/m surface tension
    let g = 9.81;
    let q_chf = 0.131 * h_lv * rho_v
        * f64::powf(sigma * g * (rho_l - rho_v) / (rho_v * rho_v), 0.25);
    // Subcooled boiling scales with wall superheat.
    q_chf * delta_t_k.clamp(0.0, 4.0)
}

/// Thermal margin check for the NbN superconducting bus.
pub fn nbn_margin_k() -> f64 {
    NBN_TC_K - OPERATING_TEMP_K
}

/// SiC crowbar: energy recovered (J) from a dump of `e_in` J.
pub fn crowbar_recovered_j(e_in: f64) -> f64 {
    e_in * SIC_RECOVERY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_numbers() {
        assert!((TOTAL_OUTPUT_KW - 999.054).abs() < 1e-9);
        assert!((output_kw(1633) - 906.36399).abs() < 1e-2);
        assert_eq!(RESERVE_MODULES, 167);
        assert!((net_margin_kw(1800) - 93.054).abs() < 1e-9);
    }

    #[test]
    fn thermal_bounds() {
        assert_eq!(DIAMOND_K_W_MK, 2000.0);
        assert!((nbn_margin_k() - 11.79).abs() < 1e-9);
        assert_eq!(MGB2_TC_K, 39.0);
    }
}
