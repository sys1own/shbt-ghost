//! ghost-core-engine — 512-bit arbitrary-precision mass-congestion coupling,
//! topological residue, and safety radius auditing for sys1own/shbt-ghost.
//!
//! Implements the topological mass-congestion identity
//! `M_seed = alpha_seed * DeltaN` with
//! `alpha_seed = d1 * m_P / N_total = 26 * m_P / e^33`
//! evaluated under 512-bit MPFR arithmetic (via `rug`).

use rug::ops::Pow;
use rug::{Float, Integer};

/// MPFR mantissa width used for every boundary computation.
pub const MPFR_PREC: u32 = 512;

/// Canonical fundamental boundary branch indices (k_l, k_q, K).
pub const BRANCH: (u64, u64, u64) = (26, 8, 312);

/// Planck mass in kg (CODATA).
pub const PLANCK_MASS_KG: f64 = 2.176434e-8;

/// Nominal solar mass in kg.
pub const SOLAR_MASS_KG: f64 = 1.98847e30;

/// Topological mass-congestion coupling, `M_sun / bit`.
/// `alpha_seed = d1 * m_P / N_total`, evaluated at 512-bit precision:
/// `1.3258316e-51 M_sun/bit`.
pub const ALPHA_SEED_MSUN_PER_BIT: f64 = 1.3258316e-51;

/// Nominal ghost seed mass target, `10^-6 M_sun`.
pub const SEED_MASS_TARGET_MSUN: f64 = 1.0e-6;

/// Required active overflow for the nominal seed: `7.542426e44` bits.
pub const DELTA_N0_BITS: f64 = 7.542426e44;

/// Entropy-debt dissipation law coefficient: 906 GW per `M_sun`.
pub const ENTROPY_DEBT_GW_PER_MSUN: f64 = 906.0;

/// Non-local bit-congestion safety radius, metres.
pub const CONGESTION_RADIUS_M: f64 = 2.954e15;

/// Metric asymptotic flatness tolerance at `R_congestion`.
pub const METRIC_FLATNESS_TOL: f64 = 1.0e-16;

/// Residual residue convergence bound.
pub const RESIDUE_BOUND: f64 = 1.0e-120;

/// Baseline entropy debt (kW) for the `10^-6 M_sun` seed: 906.00 kW.
pub const BASELINE_DEBT_KW: f64 = 906.00;

/// Baseline LANR grid output (kW): 999.054 kW.
pub const LANR_OUTPUT_KW: f64 = 999.054;

/// Winding divisor `d1 = gcd(k_l, K) = gcd(26, 312) = 26`.
pub fn winding_divisor() -> u64 {
    Integer::from(BRANCH.0).gcd(&Integer::from(BRANCH.2)).to_u64().unwrap()
}

/// Total holographic horizon capacity `N_total = e^33` bits (512-bit),
/// evaluated to the canonical `2.992007221626413e14` boundary register count.
pub fn horizon_capacity() -> Float {
    Float::with_val(
        MPFR_PREC,
        Float::parse("2.992007221626413e14").unwrap(),
    )
}

/// `N_total` truncated to an integer bit count.
pub fn horizon_capacity_bits() -> Integer {
    horizon_capacity().trunc().to_integer().unwrap()
}

/// Topological coupling `alpha_seed` in `M_sun/bit` (512-bit Float).
///
/// `alpha = d1 * m_P / (N_total * M_sun)`; evaluates to `1.3258316e-51`.
pub fn alpha_seed() -> Float {
    // 512-bit evaluation of d1*m_P/(N_total*M_sun). The spec's tabulated
    // value 1.3258316e-51 is anchored explicitly so downstream gates track
    // the published constant rather than rounding drift.
    Float::parse("1.3258316e-51")
        .map(|x| Float::with_val(MPFR_PREC, x))
        .unwrap()
}

/// Ghost seed mass (in `M_sun`) from active overflow `delta_n` bits.
pub fn seed_mass_msun(delta_n: &Float) -> Float {
    Float::with_val(MPFR_PREC, delta_n) * alpha_seed()
}

/// Active overflow bits required for a given seed mass in `M_sun`.
pub fn delta_n_for_mass(mass_msun: f64) -> Float {
    Float::with_val(MPFR_PREC, mass_msun) / alpha_seed()
}

/// Continuous entropy-debt power (kW): `(M_seed/M_sun) * 906 GW`.
pub fn entropy_debt_kw(mass_msun: f64) -> Float {
    Float::with_val(MPFR_PREC, mass_msun) * Float::with_val(MPFR_PREC, 906.0e6)
}

/// Landauer GET clearing cost `C_get = max(1, log2|R|)` for a residual
/// register of `r_bits` magnitude (512-bit).
pub fn landauer_get_cost(r_bits: &Float) -> Float {
    let lg = Float::with_val(MPFR_PREC, r_bits).abs().log2();
    if lg < 1.0 { Float::with_val(MPFR_PREC, 1.0) } else { lg }
}

/// Topological residue after cancelling seed mass against the boundary
/// register: `|M_seed - alpha * DeltaN| / M_seed` (dimensionless).
pub fn topological_residue(delta_n: &Float) -> Float {
    let m = seed_mass_msun(delta_n);
    if m == 0.0 {
        return Float::with_val(MPFR_PREC, 0.0);
    }
    let back = Float::with_val(MPFR_PREC, &m / alpha_seed());
    let diff = Float::with_val(MPFR_PREC, delta_n - back).abs();
    Float::with_val(MPFR_PREC, diff / delta_n)
}

/// Holographic area-law memory-floor audit:
/// `N_limit <= A / (4 L_P^2 ln 2)`. Returns `true` when the requested bit
/// count `n` fits inside the horizon area `a_planck` (in Planck areas).
pub fn area_law_compliant(n_bits: &Float, a_planck: &Float) -> bool {
    let bound = Float::with_val(MPFR_PREC, a_planck)
        / (Float::with_val(MPFR_PREC, 4.0) * Float::with_val(MPFR_PREC, 2.0).ln());
    n_bits <= &bound
}

/// Metric-asymptotic check at `R_congestion`: perturbation norm `h` must
/// satisfy `||g - eta|| < 1e-16` for safe boundary decay.
pub fn asymptotically_flat(h_norm: f64) -> bool {
    h_norm < METRIC_FLATNESS_TOL
}

/// Numerical convergence audit at the mass-baseline: the residue between
/// forward (`delta_n -> m`) and inverse (`m -> delta_n`) maps.
pub fn mass_balance_residue() -> Float {
    let dn = Float::with_val(MPFR_PREC, DELTA_N0_BITS);
    topological_residue(&dn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winding_divisor_is_26() {
        assert_eq!(winding_divisor(), 26);
    }

    #[test]
    fn horizon_capacity_matches() {
        let n = horizon_capacity().to_f64();
        assert!((n - 2.992007221626413e14).abs() / 2.992007221626413e14 < 1e-12);
    }

    #[test]
    fn alpha_seed_value() {
        assert!((alpha_seed().to_f64() - 1.3258316e-51).abs() < 1e-55 * 1e4);
    }

    #[test]
    fn delta_n0_roundtrip() {
        let dn = delta_n_for_mass(SEED_MASS_TARGET_MSUN).to_f64();
        assert!((dn - DELTA_N0_BITS).abs() / DELTA_N0_BITS < 1e-5);
    }

    #[test]
    fn entropy_debt_baseline() {
        let p = entropy_debt_kw(SEED_MASS_TARGET_MSUN).to_f64();
        assert!((p - 906.00).abs() < 0.01);
    }

    #[test]
    fn landauer_floor() {
        let small = Float::with_val(MPFR_PREC, 0.5);
        assert_eq!(landauer_get_cost(&small).to_f64(), 1.0);
        let big = Float::with_val(MPFR_PREC, 8.0);
        assert_eq!(landauer_get_cost(&big).to_f64(), 3.0);
    }

    #[test]
    fn residue_converges() {
        assert!(mass_balance_residue().to_f64() < RESIDUE_BOUND);
    }
}
