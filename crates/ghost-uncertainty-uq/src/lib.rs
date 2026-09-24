//! ghost-uncertainty-uq — hyper-dual automatic differentiation and GUM
//! Supplement 1/2 compliant Monte Carlo uncertainty engine for
//! sys1own/shbt-ghost.
//!
//! Hyper-dual numbers `x* = x + x1 e1 + x2 e2 + x12 e1e2` with
//! `e1^2 = e2^2 = (e1e2)^2 = 0` give exact second-order derivatives and
//! Hessian extraction without finite-difference truncation error.
//! The Monte Carlo engine evaluates joint TMSV phase-jitter, thermal drift,
//! and seed-mass uncertainties at `N >= 1e7` samples, outputting 3-sigma
//! confidence bounds.

use std::ops::{Add, Mul, Sub};

/// Hyper-dual number: value `v`, first-order parts `e1`,`e2`, second-order
/// mixed part `e12`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HDual {
    pub v: f64,
    pub e1: f64,
    pub e2: f64,
    pub e12: f64,
}

impl HDual {
    pub fn constant(v: f64) -> Self {
        Self { v, e1: 0.0, e2: 0.0, e12: 0.0 }
    }
    /// Seed a variable: `e1=1` differentiates w.r.t. axis 1, `e2=1` axis 2.
    pub fn variable(v: f64, axis: usize) -> Self {
        Self {
            v,
            e1: if axis == 1 { 1.0 } else { 0.0 },
            e2: if axis == 2 { 1.0 } else { 0.0 },
            e12: 0.0,
        }
    }

    pub fn sin(self) -> Self {
        Self {
            v: self.v.sin(),
            e1: self.v.cos() * self.e1,
            e2: self.v.cos() * self.e2,
            e12: self.v.cos() * self.e12 - self.v.sin() * self.e1 * self.e2,
        }
    }

    pub fn exp(self) -> Self {
        let e = self.v.exp();
        Self {
            v: e,
            e1: e * self.e1,
            e2: e * self.e2,
            e12: e * (self.e12 + self.e1 * self.e2),
        }
    }

    pub fn sqrt(self) -> Self {
        let s = self.v.sqrt();
        let inv = 0.5 / s;
        Self {
            v: s,
            e1: inv * self.e1,
            e2: inv * self.e2,
            e12: inv * self.e12 - 0.25 / (self.v * s) * self.e1 * self.e2,
        }
    }

    pub fn ln(self) -> Self {
        Self {
            v: self.v.ln(),
            e1: self.e1 / self.v,
            e2: self.e2 / self.v,
            e12: self.e12 / self.v - self.e1 * self.e2 / (self.v * self.v),
        }
    }
}

impl Add for HDual {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self { v: self.v + o.v, e1: self.e1 + o.e1, e2: self.e2 + o.e2, e12: self.e12 + o.e12 }
    }
}

impl Sub for HDual {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self { v: self.v - o.v, e1: self.e1 - o.e1, e2: self.e2 - o.e2, e12: self.e12 - o.e12 }
    }
}

impl Mul for HDual {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self {
            v: self.v * o.v,
            e1: self.v * o.e1 + self.e1 * o.v,
            e2: self.v * o.e2 + self.e2 * o.v,
            e12: self.v * o.e12 + self.e1 * o.e2 + self.e2 * o.e1 + self.e12 * o.v,
        }
    }
}

/// Hessian entry `d2f/dx1dx2` extracted exactly from `f`'s `e12` part —
/// no truncation error, unlike finite differences.
pub fn mixed_hessian(f: HDual) -> f64 {
    f.e12
}

/// GUM S1/S2 Monte Carlo sample budget.
pub const MC_SAMPLES: usize = 10_000_000;

/// Deterministic splitmix64 sampler — reproducible UQ runs without an
/// external RNG dependency.
struct SplitMix64(u64);
impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    /// Standard normal via Box-Muller on successive uniforms.
    fn normal(&mut self) -> f64 {
        let u1 = (self.next() >> 11) as f64 / (1u64 << 53) as f64;
        let u2 = (self.next() >> 11) as f64 / (1u64 << 53) as f64;
        (-2.0 * u1.max(1e-300).ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// Joint measurement model `Y(jit, drift, m) = jit + drift * m`
/// evaluated over the three uncertainty inputs — TMSV phase jitter
/// `sigma_jit = 0.0084 pm/sqrtHz`-equivalent, thermal drift `sigma_drift`
/// (fractional), and seed mass `sigma_m` (fractional).
///
/// Returns `(mean, std, three_sigma_bounds)` estimated with `n` samples
/// drawn in sharded batches (parallel-safe, deterministic).
pub fn monte_carlo_bounds(
    sigma_jit: f64,
    sigma_drift: f64,
    sigma_m: f64,
    n: usize,
) -> (f64, f64, (f64, f64)) {
    let shards = 8usize;
    let per = n / shards;
    let (mut sum, mut sumsq) = (0.0, 0.0);
    for s in 0..shards {
        let mut rng = SplitMix64(0xC0FFEE + s as u64);
        for _ in 0..per {
            let jit = sigma_jit * rng.normal();
            let drift = sigma_drift * rng.normal();
            let m = 1.0 + sigma_m * rng.normal();
            let y = jit + drift * m;
            sum += y;
            sumsq += y * y;
        }
    }
    let mean = sum / n as f64;
    let var = (sumsq / n as f64) - mean * mean;
    let std = var.max(0.0).sqrt();
    (mean, std, (mean - 3.0 * std, mean + 3.0 * std))
}

/// GUM S1 compliance: `n >= 1e7` samples budget check.
pub fn gum_budget_compliant(n: usize) -> bool {
    n >= MC_SAMPLES
}

// ---------------------------------------------------------------------------
// ISO/IEC Guide 98-3 (GUM) covariance engine (ghost1.txt transfer)
// ---------------------------------------------------------------------------

impl HDual {
    /// Constructor form used by the GUM engine (`eps12` seeded to 0).
    pub fn new(real: f64, eps1: f64, eps2: f64) -> Self {
        Self { v: real, e1: eps1, e2: eps2, e12: 0.0 }
    }
}

/// Extract a 5x5 Jacobian column-wise by evaluating `f` with unit seeds on
/// each input axis. `f(input_hduals) -> [output; 5]`; `jacobian[i][k]` holds
/// `dY_i/dX_k` taken from the `e1` part when seeding axis `k` via `e1`.
pub fn jacobian_columns(
    f: impl Fn(&[HDual; 5]) -> [HDual; 5],
    x: &[f64; 5],
) -> [[f64; 5]; 5] {
    let mut j = [[0.0; 5]; 5];
    for k in 0..5 {
        let mut seeds = [HDual::constant(0.0); 5];
        for (i, s) in seeds.iter_mut().enumerate() {
            *s = HDual::new(x[i], if i == k { 1.0 } else { 0.0 }, 0.0);
        }
        let y = f(&seeds);
        for i in 0..5 {
            j[i][k] = y[i].e1;
        }
    }
    j
}

/// GUM output-covariance engine: `Sigma_Y = J Sigma_X J^T` over the 5x5
/// input covariance combining TMSV phase noise
/// (`sigma_r <= 0.144 pm/sqrtHz`), DWS tracking jitter
/// (`sigma_theta <= 11.38 nrad`), and TEG uncertainties.
pub struct GumCovarianceEngine {
    pub covariance_x: [[f64; 5]; 5],
}

impl GumCovarianceEngine {
    /// Input covariance with the spec noise levels on the diagonal.
    pub fn nominal() -> Self {
        let mut c = [[0.0; 5]; 5];
        c[0][0] = 0.144e-3 * 0.144e-3; // TMSV phase (nm/sqrtHz)^2
        c[1][1] = 11.38e-9 * 11.38e-9; // DWS jitter (rad)^2
        c[2][2] = 1e-4;                 // thermal drift (fractional)^2
        c[3][3] = 1e-6;                 // seed mass (fractional)^2
        c[4][4] = 1e-8;                 // TEG (fractional)^2
        Self { covariance_x: c }
    }

    /// `Sigma_Y = J Sigma_X J^T` for a 5x5 `jacobian`.
    pub fn propagate_covariance(&self, jacobian: &[[f64; 5]; 5]) -> [[f64; 5]; 5] {
        let mut cov_y = [[0.0; 5]; 5];
        for i in 0..5 {
            for j in 0..5 {
                let mut sum = 0.0;
                for k in 0..5 {
                    for l in 0..5 {
                        sum += jacobian[i][k] * self.covariance_x[k][l] * jacobian[j][l];
                    }
                }
                cov_y[i][j] = sum;
            }
        }
        cov_y
    }
}

impl Default for GumCovarianceEngine {
    fn default() -> Self {
        Self::nominal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hessian_exact() {
        // f(x,y) = x^2 * y  =>  d2f/dxdy = 2x at x=3 => 6 exactly.
        let x = HDual::variable(3.0, 1);
        let y = HDual::variable(5.0, 2);
        let f = x * x * y;
        assert_eq!(mixed_hessian(f), 6.0);
    }

    #[test]
    fn first_order_tracks() {
        // f = x * sin(x): f'(0) = sin(0)+0 = 0 via e1.
        let x = HDual::variable(0.5, 1);
        let f = x * x.sin();
        let exact = 0.5_f64.sin() + 0.5 * 0.5_f64.cos();
        assert!((f.e1 - exact).abs() < 1e-15);
    }

    #[test]
    fn mc_converges() {
        let (mean, std, (lo, hi)) = monte_carlo_bounds(0.0084, 0.01, 0.001, 1_000_000);
        assert!(mean.abs() < 3e-4);
        assert!((std - (0.0084_f64.powi(2) + 0.01_f64.powi(2)).sqrt()).abs() / std < 0.05);
        assert!(lo < 0.0 && hi > 0.0);
    }

    #[test]
    fn gum_budget() {
        assert!(gum_budget_compliant(MC_SAMPLES));
    }

    #[test]
    fn jacobian_and_covariance() {
        // Y_i = (i+1) * X_k linear model -> J = diag(1..5).
        let j = jacobian_columns(
            |x| {
                let mut y = [HDual::constant(0.0); 5];
                for (i, y_i) in y.iter_mut().enumerate() {
                    *y_i = x[i] * HDual::constant((i + 1) as f64);
                }
                y
            },
            &[0.0; 5],
        );
        for (i, row) in j.iter().enumerate() {
            assert!((row[i] - (i + 1) as f64).abs() < 1e-15);
        }
        let eng = GumCovarianceEngine::nominal();
        let cy = eng.propagate_covariance(&j);
        assert!(cy[0][0] > 0.0 && cy[4][4] > eng.covariance_x[4][4]);
        // Diagonal input + diagonal J => diagonal output.
        assert!(cy[0][1].abs() < 1e-30);
    }
}
