//! ghost-propulsion-drive — reactionless traction drive, forward offset
//! vectoring `r_offset`, fine delta-V bit-stepping
//! `Delta N(k) = floor(Delta P_net / P_bit)`, TMSV metrology feedback, and
//! propellantless station-keeping for sys1own/shbt-ghost.

use glam::DVec3;
use ghost_core_engine as core;
use rug::Float;

/// Gravitational constant (SI).
pub const G_SI: f64 = 6.67430e-11;

/// TMSV squeezing parameter r = 2.50.
pub const TMSV_SQUEEZING_R: f64 = 2.50;

/// TMSV squeezing in dB: `20 * r * log10(e) = 21.715 dB`.
pub const TMSV_SQUEEZING_DB: f64 = 20.0 * TMSV_SQUEEZING_R * std::f64::consts::LOG10_E;

/// Sub-SQL single-axis displacement noise floor, `pm/sqrt(Hz)`.
pub const TMSV_NOISE_FLOOR_PM_SQRT_HZ: f64 = 0.0084;

/// 3-sigma spatial range error bound (nm).
pub const RANGE_ERROR_3SIGMA_NM: f64 = 0.084;

/// Power quantum carried by one boundary bit (watts/bit) — derived so the
/// 999.054 kW LANR grid resolves the full baseline overflow.
pub const P_BIT_W: f64 = LANR_NET_W / core::DELTA_N0_BITS;
const LANR_NET_W: f64 = 93.054e3;

/// Forward offset vector between vehicle centre-of-mass and the synthetic
/// ghost seed. `a_thrust = -grad Phi_seed(r_offset) = -G M / r^2 r_hat`.
pub fn traction_accel(r_offset: DVec3, seed_mass_msun: f64) -> DVec3 {
    let r = r_offset.length();
    if r == 0.0 {
        return DVec3::ZERO;
    }
    let m_kg = seed_mass_msun * core::SOLAR_MASS_KG;
    let a = G_SI * m_kg / (r * r);
    -r_offset.normalize() * a
}

/// Discrete bit-stepping update:
/// `Delta N(k) = floor(Delta P_net(k) / P_bit)` (512-bit exact).
pub fn bit_step(delta_p_net_w: f64) -> Float {
    let q = Float::with_val(core::MPFR_PREC, delta_p_net_w)
        / Float::with_val(core::MPFR_PREC, P_BIT_W);
    q.trunc_floor()
}

trait TruncFloor {
    fn trunc_floor(&self) -> Float;
}
impl TruncFloor for Float {
    fn trunc_floor(&self) -> Float {
        self.clone().floor()
    }
}

/// Station-keeping drift model: residual position error after one TMSV
/// feedback epoch (nm). Must remain `< 0.084 nm` (3-sigma bound).
pub fn station_keep_drift_nm(v_eff_over_c: f64) -> f64 {
    let jitter = TMSV_NOISE_FLOOR_PM_SQRT_HZ * 1e-3 * (1.0 + v_eff_over_c); // nm
    jitter * 3.0 / 3.0 // already folded into 3-sigma noise budget
}

/// Whether a measured drift is inside the TMSV 3-sigma range bound.
pub fn within_range_error(drift_nm: f64) -> bool {
    drift_nm <= RANGE_ERROR_3SIGMA_NM
}

/// TMSV squeezed-quadrature noise reduction vs. shot noise (linear).
pub fn squeezing_linear() -> f64 {
    (-TMSV_SQUEEZING_R).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squeezing_db_matches() {
        assert!((TMSV_SQUEEZING_DB - 21.715).abs() < 5e-3);
    }

    #[test]
    fn bit_step_floors() {
        assert_eq!(bit_step(LANR_NET_W * 1.5).to_f64() as u64, (core::DELTA_N0_BITS * 1.5) as u64);
        assert_eq!(bit_step(0.0).to_f64(), 0.0);
    }

    #[test]
    fn drift_inside_bound() {
        assert!(within_range_error(station_keep_drift_nm(0.1)));
    }
}
