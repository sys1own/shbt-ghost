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

// ---------------------------------------------------------------------------
// Chalcogenide GST metamaterial self-healing (transferred from shbt-sglt /
// shbt-cf): electro-thermal nanosecond pulse annealing model.
// ---------------------------------------------------------------------------

/// Ionizing dose tolerance for the routing stack (krad(Si)).
pub const GST_RAD_TOLERANCE_KRAD: f64 = 100.0;
/// Required conductivity recovery fraction after annealing.
pub const GST_RECOVERY_FRACTION: f64 = 0.999;

/// GST thermal model state: sheet conductivity (S/sq) relative to pristine.
#[derive(Debug, Clone, Copy)]
pub struct GstCell {
    /// Current conductivity ratio `sigma/sigma_0` in [0, 1].
    pub conductivity: f64,
    /// Accumulated ionizing dose (krad(Si)).
    pub dose_krad: f64,
}

impl GstCell {
    pub fn pristine() -> Self {
        Self { conductivity: 1.0, dose_krad: 0.0 }
    }

    /// Apply ionizing radiation: conductivity degrades exponentially with
    /// dose, `sigma/sigma_0 = exp(-D / D_char)` with `D_char = 60 krad`.
    pub fn irradiate(&mut self, dose_krad: f64) {
        self.dose_krad += dose_krad;
        self.conductivity = (-self.dose_krad / 60.0).exp();
    }

    /// Electro-thermal nanosecond anneal: with pulse energy density `e`
    /// (mJ/cm^2) at/above the 27.9 crystallization threshold, amorphous GST
    /// recrystallizes, recovering `> 99.9%` conductivity. Below threshold
    /// only partial recovery `e/27.9`-scaled occurs.
    pub fn anneal(&mut self, pulse_mj_cm2: f64, pulses: u32) {
        let per = (pulse_mj_cm2 / GST_ANNEAL_MJ_CM2).min(1.0);
        let mut cond = self.conductivity;
        for _ in 0..pulses {
            // Recrystallized fraction per pulse saturates toward 1.
            cond += (1.0 - cond) * (1.0 - (-3.0 * per).exp());
        }
        self.conductivity = cond.min(1.0);
        self.dose_krad = 0.0;
    }

    /// Recovery fraction achieved: `sigma / sigma_0`.
    pub fn recovery(&self) -> f64 {
        self.conductivity
    }
}

/// Verify a cell meets `> 99.9%` recovery after a qualifying anneal.
pub fn gst_recovered(cell: &GstCell) -> bool {
    cell.conductivity >= GST_RECOVERY_FRACTION
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

    #[test]
    fn gst_heals_over_100krad() {
        let mut cell = GstCell::pristine();
        cell.irradiate(100.0); // >= D_DDD bound
        assert!(cell.conductivity < 0.2);
        cell.anneal(GST_ANNEAL_MJ_CM2, 3);
        assert!(gst_recovered(&cell));
        assert!(cell.recovery() > GST_RECOVERY_FRACTION);
    }

    #[test]
    fn gst_subthreshold_partial() {
        let mut cell = GstCell::pristine();
        cell.irradiate(100.0);
        cell.anneal(10.0, 1);
        assert!(!gst_recovered(&cell));
    }
}
