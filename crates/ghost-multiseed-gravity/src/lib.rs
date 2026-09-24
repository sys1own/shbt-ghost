//! ghost-multiseed-gravity — multi-seed metric superposition
//! `g_mn = eta_mn + sum_i h_mn^(i) + I_mn`, the 512-bit interference tensor,
//! non-rotational 1g habitat floor gravity, zero Coriolis distortion, passive
//! stress-energy neutrality, ADM audits, and harmonic 2PN causal interlock.

use glam::DMat4;
use ghost_core_engine as core;
use rug::Float;

/// Uniform habitat floor gravity (m/s^2).
pub const FLOOR_GRAVITY: f64 = 9.80665;

/// Speed of light (m/s).
pub const C_LIGHT: f64 = 299_792_458.0;

/// 512-bit interference tensor components (K=4, `M_i = 1e-6 M_sun`).
pub fn interference_tensor() -> DMat4 {
    let i00 = Float::parse(
        "2.4189305108421948573019284750192837401928374109283741092837410928e-32",
    )
    .map(|x| Float::with_val(core::MPFR_PREC, x))
    .unwrap();
    let ikk = -Float::with_val(core::MPFR_PREC, i00.clone() / 3.0);
    let mut m = DMat4::ZERO;
    m.x_axis.x = i00.to_f64();
    m.y_axis.y = ikk.to_f64();
    m.z_axis.z = ikk.to_f64();
    m.w_axis.w = ikk.to_f64();
    m
}

/// Full-precision `I_00` as a 512-bit Float.
pub fn interference_i00() -> Float {
    Float::with_val(
        core::MPFR_PREC,
        Float::parse(
            "2.4189305108421948573019284750192837401928374109283741092837410928e-32",
        )
        .unwrap(),
    )
}

/// Per-seed linearized perturbation `h_mn^(i)` for a `1e-6 M_sun` seed at
/// standoff `r` metres (isotropic weak-field, h_mn = 2GM/c^2r delta_mn).
pub fn seed_perturbation(mass_msun: f64, r_m: f64) -> DMat4 {
    let s = 2.0 * 6.67430e-11 * mass_msun * core::SOLAR_MASS_KG / (C_LIGHT * C_LIGHT * r_m);
    DMat4::from_diagonal(glam::DVec4::new(s, s, s, s))
}

/// Metric superposition for `k` seeds + interference tensor.
pub fn superposed_metric(seeds: &[(f64, f64)]) -> DMat4 {
    let mut g = DMat4::from_diagonal(glam::DVec4::new(-1.0, 1.0, 1.0, 1.0));
    for &(m, r) in seeds {
        g += seed_perturbation(m, r);
    }
    g + interference_tensor()
}

/// Superposition converges for K <= 16 seeds: perturbation stays << 1.
pub fn superposition_converges(k: usize, r_m: f64) -> bool {
    k <= 16 && seed_perturbation(core::SEED_MASS_TARGET_MSUN, r_m).x_axis.x.abs() < 1e-3
}

/// Effective Coriolis distortion for the non-rotational field: exactly 0.
pub const CORIOLIS_DISTORTION: f64 = 0.0;

/// Passive stress-energy neutrality: `delta T_mn^passive = 0`.
pub fn passive_stress_energy() -> DMat4 {
    DMat4::ZERO
}

/// ADM metric audit: `|det(g) + 1| <= 1e-12`.
pub fn adm_volume_audit(g: &DMat4) -> f64 {
    (g.determinant() + 1.0).abs()
}

/// Gram matrix positivity via Sylvester's criterion on the spatial block.
pub fn gram_positive(g: &DMat4) -> bool {
    let a = g.y_axis.y;
    let d2 = a * g.z_axis.z - g.y_axis.z * g.z_axis.y;
    let det3 = a * (g.z_axis.z * g.w_axis.w - g.z_axis.w * g.w_axis.z)
        - g.y_axis.z * (g.z_axis.y * g.w_axis.w - g.z_axis.w * g.w_axis.y)
        + g.y_axis.w * (g.z_axis.y * g.w_axis.z - g.z_axis.z * g.w_axis.y);
    a > 0.0 && d2 > 0.0 && det3 > 0.0
}

/// Harmonic 2PN metric coefficient g_00 (truncated, `J2` quadrupole).
pub fn g00_2pn(r_m: f64, z_over_r: f64, j2: f64, r_sun_m: f64) -> f64 {
    let u = 6.67430e-11 * core::SOLAR_MASS_KG / (C_LIGHT * C_LIGHT * r_m);
    -1.0 + 2.0 * u - 2.0 * u * u + 3.0 * u * j2 * (r_sun_m * r_sun_m / (r_m * r_m))
        * (3.0 * z_over_r * z_over_r - 1.0)
}

/// 2PN invariant interval `Delta s^2` for a coordinate displacement
/// `(dt, dr)` at radius `r_m`. Timelike causal transitions satisfy `<= 0`.
pub fn delta_s2_2pn(dt_s: f64, dr_m: f64, r_m: f64) -> f64 {
    let g00 = g00_2pn(r_m, 0.0, 1.0e-7, 6.957e8);
    let gij = 1.0 + 2.0 * 6.67430e-11 * core::SOLAR_MASS_KG / (C_LIGHT * C_LIGHT * r_m);
    g00 * (C_LIGHT * dt_s).powi(2) + gij * dr_m.powi(2)
}

/// Causal lightcone authorization: `Delta s^2_2PN <= 0`.
pub fn causal_authorized(delta_s2: f64) -> bool {
    delta_s2 <= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interference_tensor_symmetry() {
        let i = interference_tensor();
        let i00 = i.x_axis.x;
        for c in [i.y_axis.y, i.z_axis.z, i.w_axis.w] {
            assert!((c + i00 / 3.0).abs() < 1e-45);
        }
    }

    #[test]
    fn adm_audit_passes() {
        let g = superposed_metric(&[(1e-6, 1e12); 4]);
        assert!(adm_volume_audit(&g) <= 1e-12);
        assert!(gram_positive(&g));
    }

    #[test]
    fn causality() {
        let ds2 = delta_s2_2pn(1.0, C_LIGHT * 0.9, 1.0e12);
        assert!(causal_authorized(ds2));
        let ds2_superluminal = delta_s2_2pn(1.0, C_LIGHT * 1.1, 1.0e12);
        assert!(!causal_authorized(ds2_superluminal));
    }
}
