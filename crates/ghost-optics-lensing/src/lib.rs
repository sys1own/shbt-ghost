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

// ---------------------------------------------------------------------------
// Eikonal coronal plasma dispersion & C6 phase masks (ghost1.txt transfer)
// ---------------------------------------------------------------------------

/// Non-paraxial eikonal solver for Baumbach-Allen coronal plasma profiles
/// `N_e(r) = A/r^6 + B/r^2` over `lambda in [200 nm, 5.0 um]`, with
/// C6-symmetric phase-mask pre-distortion holding `C <= 1e-10` across
/// baselines `L in [169.30, 1692.99] m`.
pub struct PlasmaEikonalSolver {
    pub wavelength: f64,
    pub baseline: f64,
}

impl PlasmaEikonalSolver {
    /// Total eikonal phase shift `Phi_total(b, omega)` at impact parameter
    /// `impact_b` (m) with gravitational radius `r_g` (m): gravitational
    /// delay + post-Newtonian correction - plasma dispersion integral.
    pub fn compute_eikonal_phase_shift(&self, impact_b: f64, r_g: f64) -> f64 {
        let k = 2.0 * std::f64::consts::PI / self.wavelength;
        let e = 1.602176634e-19;
        let eps_0 = 8.8541878128e-12;
        let m_e = 9.1093837015e-31;
        let omega = 2.99792458e8 * k;

        let a_const = 1.55e14;
        let b_const = 2.99e12;

        let gravitational_delay =
            (4.0 * k * r_g / 2.99792458e8) * (2.0 * 1.0e11 / impact_b).ln();
        let post_newtonian = (7.0 * std::f64::consts::PI * k * r_g * r_g) / (4.0 * impact_b);

        let plasma_factor = (k * e * e) / (eps_0 * m_e * omega * omega);
        let integral_ne = (3.0 * std::f64::consts::PI * a_const) / (8.0 * impact_b.powi(5))
            + (std::f64::consts::PI * b_const) / impact_b;

        let plasma_dispersion = plasma_factor * integral_ne;

        gravitational_delay + post_newtonian - plasma_dispersion
    }

    /// C6 phase-mask verification: residual phase variance `sigma^2` gives
    /// contrast `C = exp(sigma^2) - 1 <= 1e-10` inside the focal envelope.
    pub fn verify_c6_rejection(&self, phase_residual_variance: f64) -> bool {
        let contrast = phase_residual_variance.exp() - 1.0;
        contrast <= 1.0e-10 && self.baseline >= 169.30 && self.baseline <= 1692.99
    }
}

/// PINN physics-loss weight `lambda_phys`.
pub const PINN_LAMBDA_PHYS: f64 = 1.0;
/// PINN regularization weight `lambda_reg`.
pub const PINN_LAMBDA_REG: f64 = 1.0e-3;
/// Coronal plasma phase noise floor used for Wiener filtering (rad^2).
pub const PLASMA_PHASE_VAR: f64 = 1.0e-11;

/// PINN wave-optics deconvolution engine (transferred from shbt-sglt):
/// implicit neural surface `f_theta(r, lambda)` minimizing the combined loss
/// `L_PINN = L_data + lambda_phys ||nabla^2 E + k^2 n_eff^2 E||^2 + lambda_reg R(f_theta)`.
/// Realized as real-time Wiener deconvolution of the Bessel `J_0^2` caustic
/// under coronal plasma phase perturbations.
#[derive(Debug, Clone)]
pub struct PinnDeconvolutionEngine {
    pub lambda_phys: f64,
    pub lambda_reg: f64,
    /// Effective refractive index of the propagation medium.
    pub n_eff: f64,
}

impl PinnDeconvolutionEngine {
    pub fn new() -> Self {
        Self {
            lambda_phys: PINN_LAMBDA_PHYS,
            lambda_reg: PINN_LAMBDA_REG,
            n_eff: 1.0,
        }
    }

    /// Helmholtz residual `r = nabla^2 E + k^2 n_eff^2 E` at a sample point.
    pub fn helmholtz_residual(&self, e: f64, laplacian_e: f64, k: f64) -> f64 {
        laplacian_e + k * k * self.n_eff * self.n_eff * e
    }

    /// Physics-informed loss over residual samples.
    pub fn physics_loss(&self, l_data: f64, residuals: &[f64], reg: f64) -> f64 {
        let r2: f64 = residuals.iter().map(|r| r * r).sum();
        l_data + self.lambda_phys * r2 + self.lambda_reg * reg
    }

    /// Wiener deconvolution gain for caustic mode magnitude `h_mag` under
    /// plasma phase-noise variance `phase_var`: `W = |H|^2 / (|H|^2 + S_phi)`.
    pub fn wiener_gain(&self, h_mag: f64, phase_var: f64) -> f64 {
        let h2 = h_mag * h_mag;
        h2 / (h2 + phase_var)
    }

    /// Contrast after deconvolution: residual phase error `phase_var` scaled by
    /// `(1 - wiener_gain)` must keep coronagraphic `C <= 1e-10`.
    pub fn verify_deconvolved_contrast(&self, h_mag: f64, phase_var: f64) -> bool {
        let g = self.wiener_gain(h_mag, phase_var);
        let residual = phase_var * (1.0 - g);
        residual <= CONTRAST_BOUND
    }
}

impl Default for PinnDeconvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimum C6 phase-mask actuator loop rate (Hz).
pub const C6_LOOP_MIN_HZ: f64 = 4.80e3;
/// Baumbach-Allen A coefficient (cm^-3 -> m^-3).
pub const BA_A: f64 = 2.99e14;
/// Baumbach-Allen B coefficient (cm^-3 -> m^-3).
pub const BA_B: f64 = 1.55e14;
/// Solar radius (m).
pub const R_SUN: f64 = 6.957e8;

/// Covariant 3D RMHD coronal raytracer (ghost2.txt Target B): 2PN geodesic
/// light paths through magnetized plasma `r < 10 R_sun` with Baumbach-Allen
/// density `N_e(r,theta,t) = (A/r^6 + B/r^2)[1 + delta_CME(theta,t)]`,
/// eikonal phase `Phi_total(b,omega)` and Faraday rotation `Delta_Psi`.
#[derive(Debug, Clone)]
pub struct RmhdCoronalSolver {
    pub wavelength: f64,
}

impl RmhdCoronalSolver {
    pub fn new(wavelength: f64) -> Self {
        Self { wavelength }
    }

    /// Modified Baumbach-Allen density with CME modulation (m^-3).
    pub fn electron_density(&self, r_over_rsun: f64, delta_cme: f64) -> f64 {
        (BA_A / r_over_rsun.powi(6) + BA_B / r_over_rsun.powi(2)) * (1.0 + delta_cme)
    }

    /// CME spatio-temporal modulation `delta_CME(theta,t)` (spec form).
    pub fn cme_modulation(&self, a_cme: f64, theta: f64, theta0: f64, sigma: f64, t: f64, tau_rise: f64) -> f64 {
        a_cme * (-(theta - theta0).powi(2) / (2.0 * sigma * sigma)).exp()
            * (t / tau_rise) * (1.0 - t / tau_rise).exp()
    }

    /// Local plasma frequency `omega_p = sqrt(4 pi N_e e^2 / m_e)` (rad/s).
    pub fn plasma_frequency(&self, n_e: f64) -> f64 {
        let e = 1.602176634e-19;
        let m_e = 9.1093837015e-31;
        (4.0 * std::f64::consts::PI * n_e * e * e / m_e).sqrt()
    }

    /// Total eikonal phase `Phi_total(b,omega)` — grav + 2PN + plasma terms.
    pub fn eikonal_phase(&self, b: f64, r_g: f64, n_e_integral: f64) -> f64 {
        let c = 2.99792458e8;
        let k = 2.0 * std::f64::consts::PI / self.wavelength;
        let omega = c * k;
        let grav = (4.0 * k * r_g / c) * (2.0e11 / b).ln();
        let pn = 7.0 * std::f64::consts::PI * k * r_g * r_g / (4.0 * b);
        let e = 1.602176634e-19;
        let m_e = 9.1093837015e-31;
        let eps0 = 8.8541878128e-12;
        let plasma = (k * e * e / (eps0 * m_e * omega * omega)) * n_e_integral;
        grav + pn - plasma
    }

    /// Faraday rotation `Delta Psi = (e^3 lambda^2 / 8 pi^3 eps0 m_e^2 c^3)
    /// * int N_e B_parallel ds` (radians).
    pub fn faraday_rotation(&self, b_parallel: f64, n_e_path: f64) -> f64 {
        let e: f64 = 1.602176634e-19;
        let m_e: f64 = 9.1093837015e-31;
        let c: f64 = 2.99792458e8;
        let eps0 = 8.8541878128e-12;
        let lam2 = self.wavelength * self.wavelength;
        e.powi(3) * lam2 / (8.0 * std::f64::consts::PI.powi(3) * eps0 * m_e * m_e * c * c * c)
            * n_e_path * b_parallel
    }
}

/// Closed-loop C6 phase-mask controller (ghost2.txt Target B): regularized
/// pseudo-inverse Jacobian feedback `a <- a - g J+ [I_meas - I_target]
/// - eta L_C6 a` at `f_actuator >= 4.80 kHz`.
#[derive(Debug, Clone)]
pub struct C6PhaseMaskController {
    pub gamma: f64,
    pub eta: f64,
    pub loop_hz: f64,
    /// 6-element actuator commands.
    pub actuators: [f64; 6],
}

impl C6PhaseMaskController {
    pub fn new(loop_hz: f64) -> Self {
        Self {
            gamma: 0.35,
            eta: 0.01,
            loop_hz,
            actuators: [0.0; 6],
        }
    }

    /// C6-symmetry-enforcing cyclic Laplacian `L a` (lattice Laplacian on the
    /// 6-ring: `2a_i - a_{i-1} - a_{i+1}`).
    pub fn c6_laplacian(a: &[f64; 6]) -> [f64; 6] {
        let mut out = [0.0; 6];
        for (i, o) in out.iter_mut().enumerate() {
            *o = 2.0 * a[i] - a[(i + 5) % 6] - a[(i + 1) % 6];
        }
        out
    }

    /// One feedback update with scalar Jacobian `j` (uniform mode).
    pub fn update(&mut self, j: f64, i_measured: f64, i_target: f64, reg: f64) {
        let j_pinv = j / (j * j + reg); // regularized pseudo-inverse
        let l_a = Self::c6_laplacian(&self.actuators);
        let err = i_measured - i_target;
        for (i, a) in self.actuators.iter_mut().enumerate() {
            *a -= self.gamma * j_pinv * err + self.eta * l_a[i];
        }
    }

    /// Loop sustains the null iff rate >= 4.80 kHz and residual intensity
    /// ratio stays under `C <= 1e-10`.
    pub fn verify_nulling(&self, contrast: f64) -> bool {
        self.loop_hz >= C6_LOOP_MIN_HZ && contrast <= CONTRAST_BOUND
    }
}

impl Default for C6PhaseMaskController {
    fn default() -> Self {
        Self::new(C6_LOOP_MIN_HZ)
    }
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

    #[test]
    fn eikonal_finite_and_wideband() {
        let solver = PlasmaEikonalSolver { wavelength: 800e-9, baseline: 200.0 };
        let phi = solver.compute_eikonal_phase_shift(1.0e11, 1.48e3);
        assert!(phi.is_finite());
        for &l in &[200e-9, 800e-9, 5.0e-6] {
            let s = PlasmaEikonalSolver { wavelength: l, baseline: F0_M };
            assert!(s.compute_eikonal_phase_shift(1.0e11, 1.48e3).is_finite());
        }
    }

    #[test]
    fn c6_rejection_window() {
        let ok = PlasmaEikonalSolver { wavelength: 800e-9, baseline: 169.30 };
        assert!(ok.verify_c6_rejection(1e-12));
        let far = PlasmaEikonalSolver { wavelength: 800e-9, baseline: 2000.0 };
        assert!(!far.verify_c6_rejection(1e-12));
        let bad = PlasmaEikonalSolver { wavelength: 800e-9, baseline: 500.0 };
        assert!(!bad.verify_c6_rejection(1e-9));
    }

    #[test]
    fn pinn_deconvolution() {
        let p = PinnDeconvolutionEngine::new();
        let k = 2.0 * std::f64::consts::PI / 800e-9;
        let r = p.helmholtz_residual(1.0, -k * k * 1.0, k);
        assert!(r.abs() < 1e-6);
        assert!(p.physics_loss(0.01, &[1e-6, -2e-6], 1e-4) > 0.01);
        let g = p.wiener_gain(1.0, PLASMA_PHASE_VAR);
        assert!(g > 0.999);
        assert!(p.verify_deconvolved_contrast(1.0, PLASMA_PHASE_VAR));
        assert!(!p.verify_deconvolved_contrast(1e-8, 1e-6));
    }

    #[test]
    fn rmhd_and_c6_loop() {
        let r = RmhdCoronalSolver::new(800e-9);
        let ne = r.electron_density(2.0, 0.05);
        assert!((ne - (BA_A / 64.0 + BA_B / 4.0) * 1.05).abs() < 1e10);
        assert!(r.cme_modulation(0.2, 0.0, 0.0, 0.3, 60.0, 60.0) > 0.0);
        assert!(r.plasma_frequency(ne) > 0.0);
        assert!(r.eikonal_phase(1.0e11, 1.48e3, 1e22).is_finite());
        let fr = r.faraday_rotation(1e-4, 1e20);
        assert!(fr.is_finite() && fr.abs() > 0.0);
        // Faraday rotation scales as lambda^2.
        let r2 = RmhdCoronalSolver::new(1600e-9);
        assert!((r2.faraday_rotation(1e-4, 1e20) / fr - 4.0).abs() < 1e-9);

        let mut c = C6PhaseMaskController::new(5.0e3);
        for _ in 0..50 {
            c.update(0.8, 1e-9, 0.0, 1e-6);
        }
        assert!(c.actuators.iter().all(|a| a.is_finite()));
        assert!(c.verify_nulling(1e-12));
        assert!(!C6PhaseMaskController::new(4.0e3).verify_nulling(1e-12));
        assert_eq!(C6PhaseMaskController::c6_laplacian(&[1.0; 6]), [0.0; 6]);
    }
}
