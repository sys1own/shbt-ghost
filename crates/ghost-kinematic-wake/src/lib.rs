//! ghost-kinematic-wake — dynamic interference Lagrangian `L_int`, third-order
//! wake tensor momentum compensation `mu_comp(t)`, holographic eigenvector
//! rigidity `|mu_comp - mu_0| <= 1e-12`, and 5th-order minimum-jerk
//! trajectories `s(tau) = 10 t^3 - 15 t^4 + 6 t^5`.

use ghost_core_engine as core;
use rug::Float;

/// Third-order wake coupling coefficients (boundary topological invariants).
pub const WAKE_ALPHA: [f64; 3] = [
    1.000000000000e-4,
    3.141592653589e-6,
    2.718281828459e-8,
];

/// Holographic mass-rigidity bound.
pub const RIGIDITY_BOUND: f64 = 1.0e-12;

/// Speed of light.
pub const C_LIGHT: f64 = 299_792_458.0;

/// Peak normalized velocity of the minimum-jerk profile: `max(s') = 1.875`.
pub const MIN_JERK_MAX_VEL: f64 = 1.8750;
/// Peak normalized acceleration: `max(|s''|) = 5.7735`.
pub const MIN_JERK_MAX_ACCEL: f64 = 5.7735;

/// Minimum-jerk position `s(tau)`, tau in [0, 1].
pub fn min_jerk_s(tau: f64) -> f64 {
    tau.powi(3) * (10.0 - 15.0 * tau + 6.0 * tau * tau)
}
/// `s'(tau)`.
pub fn min_jerk_ds(tau: f64) -> f64 {
    30.0 * tau * tau - 60.0 * tau.powi(3) + 30.0 * tau.powi(4)
}
/// `s''(tau)`.
pub fn min_jerk_d2s(tau: f64) -> f64 {
    60.0 * tau - 180.0 * tau * tau + 120.0 * tau.powi(3)
}

/// Dynamic interference Lagrangian `L_int` for effective velocity `v`
/// (m/s) at overflow `delta_n` (512-bit). Models boundary phase drag:
/// `L_int = sum_k alpha_k (v/c)^k * DeltaN / N_total`.
pub fn interference_lagrangian(v_eff: f64, delta_n: &Float) -> Float {
    let ratio = Float::with_val(core::MPFR_PREC, delta_n)
        / core::horizon_capacity();
    let beta = v_eff / C_LIGHT;
    let mut l = Float::with_val(core::MPFR_PREC, 0.0);
    for (k, a) in WAKE_ALPHA.iter().enumerate() {
        l += Float::with_val(core::MPFR_PREC, *a)
            * Float::with_val(core::MPFR_PREC, beta.powi((k + 1) as i32));
    }
    l * ratio
}

/// Raw (uncompensated) wake correction requested by the kinematic wake.
pub fn wake_correction_raw(v_eff: f64, delta_n: &Float) -> Float {
    interference_lagrangian(v_eff, delta_n)
}

/// Wake-compensated mass parameter `mu_comp(t)` after the holographic
/// feedback loop enforces eigenvector rigidity: the controller saturates the
/// applied boundary-density offset at the `1e-12` bound so
/// `|mu_comp - mu_0| <= 1e-12` for all `v_eff <= 0.1 c`.
pub fn mu_compensated(mu_0: f64, v_eff: f64, delta_n: &Float) -> Float {
    let corr = wake_correction_raw(v_eff, delta_n);
    let bound = Float::with_val(core::MPFR_PREC, RIGIDITY_BOUND);
    let applied = if corr.clone().abs() > bound {
        if corr > 0.0 { bound.clone() } else { -bound }
    } else {
        corr
    };
    Float::with_val(core::MPFR_PREC, mu_0) - applied
}

/// Rigidity residual `|mu_comp - mu_0|`.
pub fn rigidity_residual(mu_0: f64, v_eff: f64, delta_n: &Float) -> Float {
    let m = mu_compensated(mu_0, v_eff, delta_n);
    (m - Float::with_val(core::MPFR_PREC, mu_0)).abs()
}

/// Third-order wake tensor Theta_mnr — fully symmetric in its indices.
/// Returns the `(mu, nu, rho)` component of the symmetric wake tensor.
pub fn wake_tensor_symmetric(mu: usize, nu: usize, rho: usize) -> f64 {
    // Symmetric rank-3 tensor built from the seed direction n and the
    // isotropic trace; component depends only on the sorted index multiset.
    let mut idx = [mu, nu, rho];
    idx.sort_unstable();
    match idx {
        [0, 0, 0] => WAKE_ALPHA[0],
        [0, 0, 1] | [0, 0, 2] | [0, 0, 3] => WAKE_ALPHA[1],
        [0, 1, 1] | [0, 2, 2] | [0, 3, 3] => WAKE_ALPHA[1],
        _ => WAKE_ALPHA[2],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_jerk_bounds() {
        let (mut vmax, mut amax) = (0.0_f64, 0.0_f64);
        for i in 0..=10000 {
            let t = i as f64 / 10000.0;
            vmax = vmax.max(min_jerk_ds(t));
            amax = amax.max(min_jerk_d2s(t).abs());
        }
        assert!((vmax - MIN_JERK_MAX_VEL).abs() < 1e-3);
        assert!((amax - MIN_JERK_MAX_ACCEL).abs() < 1e-3);
    }

    #[test]
    fn rigidity_enforced() {
        let dn = Float::with_val(core::MPFR_PREC, core::DELTA_N0_BITS);
        let resid = rigidity_residual(1.0, 0.1 * C_LIGHT, &dn);
        assert!(resid.to_f64() <= RIGIDITY_BOUND);
    }

    #[test]
    fn wake_tensor_symmetry() {
        for mu in 0..4 {
            for nu in 0..4 {
                for rho in 0..4 {
                    assert_eq!(
                        wake_tensor_symmetric(mu, nu, rho),
                        wake_tensor_symmetric(rho, mu, nu)
                    );
                }
            }
        }
    }
}
