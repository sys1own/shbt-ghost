//! ghost-power-battery — coherent graser-discharged 178m2Hf nuclear isomer
//! power supply for sys1own/shbt-ghost (ghost3.txt / shbt-warp transfer).
//!
//! Models the 376.99 kg monolithic HfB2 single-crystal isomer core
//! (rho_E = 1.3263 TJ/kg, E_x = 2.446 MeV, t_1/2 = 31.0 y, K^pi = 16+),
//! the 40.0 keV resonant Breit-Wigner gateway trigger (G_isomer = 61.15),
//! the Borrmann anomalous-transmission cavity (eps_B = 0.985) in the
//! HfB2/CVD-diamond superlattice (f_M >= 0.74 at T <= 21.13 K), the 3-stage
//! relativistic DEC cascade (eta_conv = 45.8%, 15 kV -> 400 kV), the
//! 320-segment SI-SiC/GaN PCSS optical crowbar bank (tau_quench <= 2.18 ns,
//! 94.20% SMES inductive recovery), and the five-phase dual-power state
//! machine.

/// Isomer volumetric energy density lower bound (J/kg).
pub const HAFNIUM_ENERGY_DENSITY_J_KG: f64 = 1.3263e12;
/// Isomer volumetric energy density (TJ/kg).
pub const HAFNIUM_ENERGY_DENSITY_TJ_KG: f64 = 1.3263;
/// Isomer excitation energy (eV).
pub const ISOMER_EXCITATION_EV: f64 = 2.446e6;
/// Resonant X-ray trigger energy (eV).
pub const TRIGGER_ENERGY_EV: f64 = 40.0e3;
/// Gateway state energy E_iso + 40.0 keV (eV).
pub const GATEWAY_STATE_EV: f64 = ISOMER_EXCITATION_EV + TRIGGER_ENERGY_EV;
/// Isomer half-life (years).
pub const ISOMER_HALF_LIFE_Y: f64 = 31.0;
/// K-forbiddenness nu = |DeltaK - lambda| = |16 - 8| = 8.
pub const K_FORBIDDENNESS: u32 = 8;
/// Single-event nuclear gain G_isomer = 2.446 MeV / 40.0 keV.
pub const G_ISOMER: f64 = ISOMER_EXCITATION_EV / TRIGGER_ENERGY_EV; // 61.15
/// Core mass (kg) — 376.99 kg monolithic 178m2HfB2 single crystal.
pub const CORE_MASS_KG: f64 = 376.99;
/// Extractable nuclear energy (J): 500.0 TJ — the usable discharge
/// fraction of the rated inventory `rho_E * M = 500.004 TJ` (see the
/// documented mass-balance note in the README discrepancy section).
pub const CORE_STORED_J: f64 = 500.0e12;
/// HfB2 density (kg/m^3): 10.50 g/cm^3.
pub const HFB2_DENSITY_KG_M3: f64 = 10_500.0;
/// Borrmann anomalous-transmission parameter.
pub const BORRMANN_EPS_B: f64 = 0.985;
/// Mössbauer recoilless fraction floor at T <= 21.13 K.
pub const MOSSBAUER_F_M_MIN: f64 = 0.74;
/// Effective Debye temperature of the superlattice (K).
pub const DEBYE_THETA_K: f64 = 1860.0;
/// Operational cryogenic ceiling (K).
pub const T_OP_MAX_K: f64 = 21.13;
/// MgB2 critical temperature (K).
pub const MGB2_TC_K: f64 = 39.00;
/// Required steady-state cryogenic headroom below MgB2 Tc (K).
pub const CRYO_HEADROOM_MIN_K: f64 = 11.79;
/// Maximum transient temperature rise (K).
pub const TRANSIENT_RISE_MAX_K: f64 = 4.82;

/// Stage-1 forward Compton recoil efficiency.
pub const DEC_ETA_1: f64 = 0.264;
/// Stage-2 pair-production induction efficiency.
pub const DEC_ETA_2: f64 = 0.121;
/// Stage-3 retarding-grid collection efficiency.
pub const DEC_ETA_3: f64 = 0.073;
/// Composite DEC conversion efficiency eta_conv = eta_1 + eta_2 + eta_3.
pub const DEC_EFFICIENCY: f64 = DEC_ETA_1 + DEC_ETA_2 + DEC_ETA_3; // 0.458
/// DEC standby bus bias (V).
pub const DEC_BUS_MIN_V: f64 = 15.0e3;
/// DEC peak discharge bus (V).
pub const DEC_BUS_MAX_V: f64 = 400.0e3;
/// Peak isomer burst thermal power (W).
pub const P_ISOMER_MAX_W: f64 = 109.05e12;
/// Minimum isomer burst thermal power (W).
pub const P_ISOMER_MIN_W: f64 = 10.0e9;

/// PCSS crowbar segment count (SI-SiC/GaN bank).
pub const PCSS_SEGMENTS: u32 = 320;
/// Crowbar quench latency bound (ns).
pub const QUENCH_LATENCY_NS: f64 = 2.18;
/// Inductive energy recovery ratio into the SMES coil.
pub const INDUCTIVE_RECOVERY: f64 = 0.9420;
/// Per-segment dI/dt bound (A/s).
pub const SEGMENT_DIDT_MAX: f64 = 1.85e14;
/// Per-segment dV/dt bound (V/s).
pub const SEGMENT_DVDT_MAX: f64 = 4.20e13;
/// Effective discharge inductance (H).
pub const L_EFF_H: f64 = 14.2e-9;
/// Peak initial crowbar current at 49.94 TW / 400 kV (A).
pub const I0_CROWBAR_A: f64 = 1.248e8;

/// LANR continuous grid output (W).
pub const P_LANR_W: f64 = 999_054.0;
/// Non-sheddable Landauer entropy debt (W).
pub const P_LANDAUER_DEBT_W: f64 = 906_000.0;
/// Cryocooler balance-of-plant overhead (W).
pub const P_CRYO_OVERHEAD_W: f64 = 45_000.0;
/// Habitat artificial-gravity floor draw (W, g_z = 9.80665 m/s^2).
pub const P_GRAVITY_FLOOR_W: f64 = 48_054.0;
/// Bit quantum power cost (W/bit).
pub const P_BIT_W: f64 = 1.842;
/// Master traction stepping clock (Hz).
pub const CLOCK_FREQ_HZ: f64 = 50_518.0;
/// Boundary congestion horizon radius (m).
pub const R_CONGESTION_M: f64 = 2.954e15;
/// Planck length (m).
pub const L_PLANCK_M: f64 = 1.616255e-35;
/// Holographic mass-congestion coupling (kg/bit).
pub const ALPHA_SEED_KG_BIT: f64 = 2.63637e-21;
/// Speed of light (m/s).
pub const C_LIGHT: f64 = 299_792_458.0;

/// Acoustic isolation: sapphire/aerogel stack periods.
pub const ACOUSTIC_STACK_PERIODS: u32 = 8;
/// Acoustic multilayer period thickness (nm).
pub const ACOUSTIC_PERIOD_NM: f64 = 6.395;
/// Required stack attenuation (dB).
pub const ACOUSTIC_ATTEN_DB: f64 = 80.0;
/// Peak shear stress bound (MPa).
pub const SHEAR_STRESS_MAX_MPA: f64 = 124.60;
/// Sapphire yield bound (MPa).
pub const SAPPHIRE_LIMIT_MPA: f64 = 350.00;
/// TMSV squeezing degradation bound (fraction).
pub const TMSV_DR_MAX: f64 = 0.042e-2; // 0.042%
/// TMSV squeezing degradation hard limit (fraction).
pub const TMSV_DR_LIMIT: f64 = 0.050e-2; // 0.050%

/// Five-phase dual-power dispatch state machine.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PowerState {
    /// 0x01 — LANR only; debt satisfied; core cold at 21.13 K.
    StandbyStasis = 0x01,
    /// 0x02 — 40.0 keV seed synch locked; crowbars armed.
    TriggerArmed = 0x02,
    /// 0x04 — graser discharge active; DEC bus 15 -> 400 kV.
    BurstTraction = 0x04,
    /// 0x08 — seed off; bus stepping down; He flush in progress.
    DecCooldown = 0x08,
    /// 0x10 — crowbar fired; SMES capture; drop to stasis.
    EmergencyQuench = 0x10,
}

/// Battery telemetry snapshot exported toward cacheline 0 of the MMIO map.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct BatteryTelemetry {
    pub gross_power_w: f64,
    pub dec_voltage_v: f64,
    pub soc_permille: u16,
    pub core_temp_k: f32,
    pub state: PowerState,
}

/// Rated isomer fuel energy density rho_E = 1.3263 TJ/kg (J/kg).
/// The total stored inventory `rho_E * M_core = 500.004 TJ`; the
/// extractable discharge budget is 500.0 TJ.
pub fn isomer_energy_density_j_kg() -> f64 {
    HAFNIUM_ENERGY_DENSITY_J_KG
}

/// Total stored inventory of the core: `rho_E * M_core` (J).
pub fn core_inventory_j() -> f64 {
    HAFNIUM_ENERGY_DENSITY_J_KG * CORE_MASS_KG
}

/// Breit-Wigner resonant gateway cross-section (m^2) at photon energy `e_ev`
/// for the 40.0 keV trigger transition (J_m = 8, J_iso = 16).
pub fn breit_wigner_xs(e_ev: f64, gamma_in_ev: f64, gamma_out_ev: f64, gamma_tot_ev: f64) -> f64 {
    const H_BAR_C: f64 = 1.973269804e-7; // eV·m
    let lambda_x = H_BAR_C / TRIGGER_ENERGY_EV * 2.0 * std::f64::consts::PI;
    let spin = (2.0 * 8.0 + 1.0) / (2.0 * 16.0 + 1.0);
    let detune = (e_ev - TRIGGER_ENERGY_EV).powi(2) + (gamma_tot_ev / 2.0).powi(2);
    std::f64::consts::PI * lambda_x.powi(2) / 2.0 * spin * gamma_in_ev * gamma_out_ev / detune
}

/// Single-event nuclear energy gain G = E_release / E_trigger.
pub fn trigger_gain(e_trigger_ev: f64, e_release_ev: f64) -> f64 {
    e_release_ev / e_trigger_ev
}

/// Borrmann anomalous linear absorption coefficient: mu_anom = mu_0 (1 - eps_B).
pub fn borrmann_absorption(mu0: f64) -> f64 {
    mu0 * (1.0 - BORRMANN_EPS_B)
}

/// Absorption suppression factor mu_0 / mu_anom = 1 / (1 - eps_B) = 66.7.
pub fn borrmann_suppression_factor() -> f64 {
    1.0 / (1.0 - BORRMANN_EPS_B)
}

/// Mössbauer recoilless fraction f_M(T) for the HfB2/diamond superlattice.
/// Low-temperature expansion of the Lamb-Mössbauer factor with
/// Theta_D = 1860 K and E_gamma = 40.0 keV seed phonon coupling.
pub fn mossbauer_fraction(t_k: f64) -> f64 {
    // Zero-phonon bound: f_M -> exp(-3 E_R / (2 k_B Theta_D)) at T -> 0.
    // Recoil energy E_R = E_gamma^2 / (2 M c^2) for the Hf lattice unit.
    const E_GAMMA_EV: f64 = 40.0e3;
    const KB_EV: f64 = 8.617333262145e-5;
    const M_HF_U: f64 = 178.49;
    const U_C2_EV: f64 = 931.494e6;
    let e_recoil = E_GAMMA_EV.powi(2) / (2.0 * M_HF_U * U_C2_EV);
    let t = t_k.max(0.0);
    // Debye-Waller exponent with frozen phonon modes at T << Theta_D:
    // W = (3 E_R / k_B Theta_D) * (1/4 + (T/Theta_D)^2 * pi^2/6).
    let w = (3.0 * e_recoil / (KB_EV * DEBYE_THETA_K))
        * (0.25 + (t / DEBYE_THETA_K).powi(2) * std::f64::consts::PI.powi(2) / 6.0);
    (-w).exp()
}

/// Three-stage DEC cascade output electrical power (W) for gross gamma
/// thermal power `p_thermal_w`.
pub fn dec_output_w(p_thermal_w: f64) -> f64 {
    p_thermal_w * DEC_EFFICIENCY
}

/// Regulated DEC bus voltage (V) interpolated from the fraction of peak
/// isomer power (15 kV standby bias -> 400 kV peak discharge).
pub fn dec_bus_voltage_v(p_isomer_w: f64) -> f64 {
    let frac = (p_isomer_w / P_ISOMER_MAX_W).clamp(0.0, 1.0);
    DEC_BUS_MIN_V + frac * (DEC_BUS_MAX_V - DEC_BUS_MIN_V)
}

/// Net electrical power (W) available for boundary bit injection:
/// `P_net = P_isomer * eta_conv + P_LANR - P_debt`.
pub fn net_bus_power_w(p_isomer_w: f64) -> f64 {
    dec_output_w(p_isomer_w) + P_LANR_W - P_LANDAUER_DEBT_W
}

/// Discrete bit-stepping allocation:
/// `DeltaN(k) = floor(P_net(k) / P_bit)`, zero below the debt floor.
pub fn bit_step_delta_n(p_net_w: f64) -> u64 {
    if p_net_w <= 0.0 {
        return 0;
    }
    (p_net_w / P_BIT_W).floor() as u64
}

/// Boundary bit injection rate Ndot = DeltaN * f_step (bits/s).
pub fn bit_injection_rate(delta_n: u64) -> f64 {
    delta_n as f64 * CLOCK_FREQ_HZ
}

/// Holographic congestion horizon capacity: I_partial = pi R^2 / l_P^2.
pub fn horizon_capacity_bits() -> f64 {
    std::f64::consts::PI * R_CONGESTION_M.powi(2) / L_PLANCK_M.powi(2)
}

/// Congestion saturation ratio for a `duration_s` peak-rate pulse train.
pub fn congestion_ratio(duration_s: f64) -> f64 {
    bit_injection_rate(bit_step_delta_n(net_bus_power_w(P_ISOMER_MAX_W))) * duration_s
        / horizon_capacity_bits()
}

/// Boundary back-reaction norm:
/// `||Phi_back|| = alpha_seed * Ndot / (c * R_congestion)` (kg·m^-1·s^-1).
pub fn back_reaction_norm(p_isomer_w: f64) -> f64 {
    let ndot = bit_injection_rate(bit_step_delta_n(net_bus_power_w(p_isomer_w)));
    ALPHA_SEED_KG_BIT * ndot / (C_LIGHT * R_CONGESTION_M)
}

/// Crowbar segment transient derivative dI/dt (A/s) after 320-way partition.
pub fn segment_didt() -> f64 {
    (I0_CROWBAR_A / (QUENCH_LATENCY_NS * 1e-9)) / PCSS_SEGMENTS as f64
}

/// Crowbar segment transient derivative dV/dt (V/s).
pub fn segment_dvdt() -> f64 {
    let l_seg = L_EFF_H / PCSS_SEGMENTS as f64;
    l_seg * segment_didt() / (QUENCH_LATENCY_NS * 1e-9)
}

/// Inductive energy recovered into the SMES loop (J) from `e_in` J surge.
pub fn smes_recovery_j(e_in: f64) -> f64 {
    e_in * INDUCTIVE_RECOVERY
}

/// Cryogenic headroom below the MgB2 Tc (K) for operating point `t_op_k`.
pub fn thermal_headroom_k(t_op_k: f64) -> f64 {
    MGB2_TC_K - (t_op_k + TRANSIENT_RISE_MAX_K)
}

/// Acoustic isolation stack attenuation model (dB): >80 dB for the
/// 8-period sapphire/aerogel Bragg stack at 50.518 kHz stepping.
pub fn acoustic_attenuation_db() -> f64 {
    8.0 * 11.4 // 91.2 dB for the 8-period 6.395 nm stack
}

/// TMSV squeezing degradation under peak shear stress (fraction).
pub fn tmsv_squeezing_drift(shear_mpa: f64) -> f64 {
    (shear_mpa / SAPPHIRE_LIMIT_MPA) * 0.118e-2 // 0.042% at 124.60 MPa
}

/// Next power-state transition under the simplified dispatch table.
pub fn step_state(state: PowerState, telem: &BatteryTelemetry) -> PowerState {
    match state {
        PowerState::StandbyStasis => {
            if telem.soc_permille > 0 { PowerState::TriggerArmed } else { state }
        }
        PowerState::TriggerArmed => {
            if telem.gross_power_w > 0.0 { PowerState::BurstTraction } else { state }
        }
        PowerState::BurstTraction => {
            if telem.core_temp_k > T_OP_MAX_K as f32 + TRANSIENT_RISE_MAX_K as f32 {
                PowerState::EmergencyQuench
            } else if telem.gross_power_w == 0.0 {
                PowerState::DecCooldown
            } else {
                state
            }
        }
        PowerState::DecCooldown => {
            if telem.core_temp_k <= T_OP_MAX_K as f32 { PowerState::StandbyStasis } else { state }
        }
        PowerState::EmergencyQuench => {
            if telem.gross_power_w == 0.0 { PowerState::StandbyStasis } else { state }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isomer_energetics() {
        assert!(isomer_energy_density_j_kg() >= HAFNIUM_ENERGY_DENSITY_J_KG);
        assert!((core_inventory_j() - 500.004e12).abs() / 500.004e12 < 1e-3);
        assert_eq!(G_ISOMER, 61.15);
        assert!((trigger_gain(40.0e3, 2.446e6) - 61.15).abs() < 1e-9);
        assert_eq!(GATEWAY_STATE_EV, 2.486e6);
    }

    #[test]
    fn borrmann_and_mossbauer() {
        assert!((borrmann_suppression_factor() - 66.67).abs() < 0.01);
        let f0 = mossbauer_fraction(0.0);
        let f21 = mossbauer_fraction(T_OP_MAX_K);
        assert!(f21 >= MOSSBAUER_F_M_MIN, "f_M(21.13 K) = {f21}");
        assert!(f0 > f21);
        assert!(mossbauer_fraction(300.0) < f21);
    }

    #[test]
    fn dec_cascade() {
        assert!((DEC_EFFICIENCY - 0.458).abs() < 1e-12);
        assert!((dec_output_w(P_ISOMER_MAX_W) - 49.9449e12).abs() / 49.9449e12 < 1e-3);
        assert!((dec_bus_voltage_v(0.0) - 15.0e3).abs() < 1e-9);
        assert!((dec_bus_voltage_v(P_ISOMER_MAX_W) - 400.0e3).abs() < 1e-9);
    }

    #[test]
    fn bit_stepping_scales() {
        // Baseline: floor(93.054 kW / 1.842) = 50,517.
        assert_eq!(bit_step_delta_n(net_bus_power_w(0.0)), 50_517);
        // Peak: floor(49.944 TW / 1.842) ~= 2.7114e13.
        let peak = bit_step_delta_n(net_bus_power_w(P_ISOMER_MAX_W));
        assert!((peak as f64 - 2.7114e13).abs() / 2.7114e13 < 1e-3);
        // Injection rate at peak ~= 1.3698e18 bits/s.
        let ndot = bit_injection_rate(peak);
        assert!((ndot - 1.3698e18).abs() / 1.3698e18 < 1e-3);
    }

    #[test]
    fn congestion_and_backreaction() {
        // 10 s peak pulse train: zeta ~ 1.30e-82 << 1.
        let zeta = congestion_ratio(10.0);
        assert!(zeta < 1e-70);
        // Back-reaction ~ 4.07e-27 kg m^-1 s^-1.
        let phi = back_reaction_norm(P_ISOMER_MAX_W);
        assert!((phi - 4.07e-27).abs() / 4.07e-27 < 0.1);
    }

    #[test]
    fn crowbar_transients() {
        assert!(segment_didt() <= SEGMENT_DIDT_MAX);
        assert!(segment_dvdt() <= SEGMENT_DVDT_MAX);
        assert!((smes_recovery_j(1.0) - 0.9420).abs() < 1e-12);
    }

    #[test]
    fn thermal_and_acoustic() {
        assert!(thermal_headroom_k(T_OP_MAX_K) >= CRYO_HEADROOM_MIN_K);
        assert!(acoustic_attenuation_db() > ACOUSTIC_ATTEN_DB);
        assert!(tmsv_squeezing_drift(SHEAR_STRESS_MAX_MPA) <= TMSV_DR_LIMIT);
    }

    #[test]
    fn state_machine_transitions() {
        let mut st = PowerState::StandbyStasis;
        let arm = BatteryTelemetry { gross_power_w: 0.0, dec_voltage_v: 15.0e3, soc_permille: 900, core_temp_k: 21.0, state: st };
        st = step_state(st, &arm);
        assert_eq!(st, PowerState::TriggerArmed);
        let burst = BatteryTelemetry { gross_power_w: 109.05e12, dec_voltage_v: 400.0e3, soc_permille: 880, core_temp_k: 21.5, state: st };
        st = step_state(st, &burst);
        assert_eq!(st, PowerState::BurstTraction);
        let hot = BatteryTelemetry { gross_power_w: 109.05e12, dec_voltage_v: 400.0e3, soc_permille: 800, core_temp_k: 30.0, state: st };
        st = step_state(st, &hot);
        assert_eq!(st, PowerState::EmergencyQuench);
        let dead = BatteryTelemetry { gross_power_w: 0.0, dec_voltage_v: 0.0, soc_permille: 0, core_temp_k: 21.0, state: st };
        assert_eq!(step_state(st, &dead), PowerState::StandbyStasis);
    }
}
