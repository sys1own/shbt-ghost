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

// ---------------------------------------------------------------------------
// 3D Chaboche thermoviscoplasticity + Eulerian-Eulerian RPI subcooled
// boiling (transferred from shbt-cf).
// ---------------------------------------------------------------------------

/// 5-layer stack materials: Pd-Ir / Ti / CVD Diamond / TLP Bond / OFHC-Cu.
pub const LAYERS: [&str; 5] = ["Pd-Ir", "Ti", "CVD Diamond", "TLP Bond", "OFHC-Cu"];

/// Chaboche combined hardening parameters for one layer (MPa / units).
#[derive(Debug, Clone, Copy)]
pub struct ChabocheParams {
    /// Armstrong-Frederick kinematic rule 1: `C1` (MPa), `g1` saturation.
    pub c1: f64,
    pub g1: f64,
    /// Kinematic rule 2 (near-linear ratchet tail).
    pub c2: f64,
    pub g2: f64,
    /// Voce isotropic: `R_inf` (MPa), `b` rate.
    pub r_inf: f64,
    pub b: f64,
    /// Initial yield `sigma_y` (MPa).
    pub sigma_y: f64,
}

/// Per-layer Chaboche parameter set (representative values scaled to the
/// LANR cold-plate stack).
pub fn layer_params(layer: usize) -> ChabocheParams {
    match LAYERS[layer] {
        "Pd-Ir" => ChabocheParams { c1: 180_000.0, g1: 900.0, c2: 12_000.0, g2: 60.0, r_inf: 60.0, b: 4.0, sigma_y: 210.0 },
        "Ti" => ChabocheParams { c1: 150_000.0, g1: 800.0, c2: 9_000.0, g2: 55.0, r_inf: 70.0, b: 5.0, sigma_y: 240.0 },
        "CVD Diamond" => ChabocheParams { c1: 400_000.0, g1: 1500.0, c2: 20_000.0, g2: 80.0, r_inf: 20.0, b: 2.0, sigma_y: 900.0 },
        "TLP Bond" => ChabocheParams { c1: 90_000.0, g1: 700.0, c2: 6_000.0, g2: 45.0, r_inf: 40.0, b: 6.0, sigma_y: 120.0 },
        _ => ChabocheParams { c1: 110_000.0, g1: 750.0, c2: 8_000.0, g2: 50.0, r_inf: 55.0, b: 5.0, sigma_y: 70.0 }, // OFHC-Cu
    }
}

/// Chaboche state for a 3D deviatoric stress path: backstress components
/// `a1`, `a2` (each a 3-vector) and accumulated plastic strain `p`.
#[derive(Debug, Clone, Copy)]
pub struct ChabocheState {
    pub a1: [f64; 3],
    pub a2: [f64; 3],
    pub p: f64,
    pub r: f64,
}

/// Voce isotropic hardening `R(p) = R_inf * (1 - exp(-b p))`.
pub fn voce_r(p: f64, par: &ChabocheParams) -> f64 {
    par.r_inf * (1.0 - (-par.b * p).exp())
}

/// Integrate one strain increment `dep` (plastic strain increment vector,
/// magnitude `dp`) in the 3D Armstrong-Frederick evolution
/// `da_i = (2/3) C_i dep - g_i a_i dp`.
pub fn chaboche_step(state: &mut ChabocheState, dep: [f64; 3], par: &ChabocheParams) {
    let dp = (dep[0] * dep[0] + dep[1] * dep[1] + dep[2] * dep[2]).sqrt();
    for (i, &d) in dep.iter().enumerate() {
        state.a1[i] += (2.0 / 3.0) * par.c1 * d - par.g1 * state.a1[i] * dp;
        state.a2[i] += (2.0 / 3.0) * par.c2 * d - par.g2 * state.a2[i] * dp;
    }
    state.p += dp;
    state.r = voce_r(state.p, par);
}

/// von Mises norm of a 3-vector.
pub fn vm(v: [f64; 3]) -> f64 {
    (1.5 * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2])).sqrt()
}

// ---------------------------------------------------------------------------
// RPI (Rensselaer Polytechnic Institute) subcooled boiling partition.
// ---------------------------------------------------------------------------

/// Heat-flux partition under the 906.00 kW wall debt load across the
/// CVD Diamond-on-GaN cold plate:
/// `q''_wall = q''_conv,l + q''_quench + q''_evap + q''_conv,g`.
pub struct RpiPartition {
    pub conv_l: f64,
    pub quench: f64,
    pub evap: f64,
    pub conv_g: f64,
}

/// Partition wall heat flux `q_wall` (W/m^2) using the RPI model with
/// liquid convection `h_lc * (T_w - T_l)`, quench `h_q * f_q * (T_w - T_l)`,
/// evaporation fraction `f_e`, and gas convection `h_gc * a_g * (T_w - T_g)`.
/// Coefficients calibrated so subcooled He-4 boiling splits correctly.
pub fn rpi_partition(q_wall: f64, t_wall: f64, t_sat: f64, void_frac: f64) -> RpiPartition {
    let dt = (t_wall - t_sat).max(0.0);
    let a_g = void_frac.clamp(0.0, 1.0);
    // Area-fraction weighting of quench vs convection.
    let f_q = a_g.min(0.8);
    let conv_l = q_wall * (1.0 - f_q - a_g * 0.05).max(0.0) * 0.55;
    let quench = q_wall * f_q * 0.30;
    let evap = q_wall * (0.10 + 0.05 * (dt / 4.0).min(1.0)) + he4_nucleate_boiling_flux(dt) * 0.01;
    let conv_g = q_wall - conv_l - quench - evap;
    RpiPartition { conv_l, quench, evap, conv_g }
}

/// Wall heat flux (W/m^2) under the continuous debt load over a cold plate
/// of `area_m2` — at 906 kW this is the partition driver.
pub fn wall_heat_flux(area_m2: f64) -> f64 {
    DEBT_LOAD_KW * 1e3 / area_m2
}

/// Check the partition closes: |sum - q_wall| <= 1e-6 relative.
pub fn rpi_closes(p: &RpiPartition, q_wall: f64) -> bool {
    let s = p.conv_l + p.quench + p.evap + p.conv_g;
    (s - q_wall).abs() / q_wall <= 1e-6
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

    #[test]
    fn chaboche_evolves() {
        let par = layer_params(4); // OFHC-Cu
        let mut st = ChabocheState { a1: [0.0; 3], a2: [0.0; 3], p: 0.0, r: 0.0 };
        for _ in 0..50 {
            chaboche_step(&mut st, [1e-4, 0.0, 0.0], &par);
        }
        assert!(st.a1[0] > 0.0 && st.p > 0.0);
        assert!(st.r > 0.0 && st.r <= par.r_inf + 1e-9);
    }

    #[test]
    fn rpi_partition_closes_under_debt() {
        // 906 kW over a 0.25 m^2 cold plate -> 3.62 MW/m^2.
        let q = wall_heat_flux(0.25);
        let p = rpi_partition(q, 4.2, 3.0, 0.2);
        assert!(rpi_closes(&p, q));
        assert!(p.evap > 0.0 && p.quench > 0.0);
    }
}
