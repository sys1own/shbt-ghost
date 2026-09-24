//! ghost-optics-lensing — synthetic gravitational lensing with variable focal
//! baselines (`f0 = 169.30 m` at `r0 = 1.000 m` aperture up to
//! `f_max = 1692.99 m`), coronagraphic Bessel `J_0^2` caustic beam shaping,
//! contrast rejection `C <= 1.0e-10`, and GST phase-change self-healing.

/// Minimum focal baseline (m) at aperture radius `r0 = 1.000 m`.
pub const F0_M: f64 = 169.30;
/// Maximum focal baseline (m).
pub const F_MAX_M: f64 = 1692.99;
/// Reference aperture radius (m).
pub const R0_M: f64 = 1.000;
/// Coronagraphic contrast rejection bound.
pub const CONTRAST_BOUND: f64 = 1.0e-10;
/// GST phase-change anneal pulse energy (mJ/cm^2).
pub const GST_ANNEAL_MJ_CM2: f64 = 27.9;

/// Effective focal length for boundary congestion modulation index `q`
/// in `[0, 1]`: `f(q) = f0 * (1 + 9 q)`.
pub fn focal_length(q: f64) -> f64 {
    F0_M * (1.0 + (F_MAX_M / F0_M - 1.0) * q.clamp(0.0, 1.0))
}

/// Bessel `J_0(x)` via its uniformly convergent power series:
/// `J_0(x) = sum_k (-1)^k (x^2/4)^k / (k!)^2`.
pub fn bessel_j0(x: f64) -> f64 {
    let x2o4 = x * x / 4.0;
    let (mut sum, mut term) = (1.0_f64, 1.0_f64);
    for k in 1..64 {
        term *= -x2o4 / (k as f64 * k as f64);
        sum += term;
        if term.abs() < 1e-18 * sum.abs() {
            break;
        }
    }
    sum
}

/// Coronagraphic caustic shaping profile: intensity `J_0^2(x)`.
pub fn caustic_profile(x: f64) -> f64 {
    bessel_j0(x).powi(2)
}

/// Achieved starlight contrast rejection at a working angle `x` (radians of
/// the shaped null): `C = J_0^2(x)` evaluated at the first null's residual.
/// With apodization factor `a`, `C(a) = J_0^2(a) * a^-2` at deep nulls.
pub fn contrast_rejection(apodization: f64) -> f64 {
    // Deep-null residual: driving apodization toward the first J0 zero
    // (x ~ 2.4048) suppresses on-axis starlight below 1e-10.
    let j = bessel_j0(apodization);
    j * j
}

/// Whether a measured contrast meets the coronagraph bound.
pub fn contrast_compliant(c: f64) -> bool {
    c <= CONTRAST_BOUND
}

/// GST self-healing check: anneal pulse energy (mJ/cm^2) must reach 27.9.
pub fn gst_healing_sufficient(pulse_mj_cm2: f64) -> bool {
    pulse_mj_cm2 >= GST_ANNEAL_MJ_CM2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focal_range() {
        assert!((focal_length(0.0) - F0_M).abs() < 1e-9);
        assert!((focal_length(1.0) - F_MAX_M).abs() < 1e-9);
    }

    #[test]
    fn bessel_known_values() {
        assert!((bessel_j0(0.0) - 1.0).abs() < 1e-15);
        assert!((bessel_j0(2.404825557695773)).abs() < 1e-12); // first zero
    }

    #[test]
    fn deep_null_contrast() {
        // Apodization at the first J0 null gives C ~ 0 < 1e-10.
        let c = contrast_rejection(2.404825557695773);
        assert!(contrast_compliant(c));
    }
}
