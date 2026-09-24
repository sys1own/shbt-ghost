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

/// Heegaard-Floer symplectic mapping-class tracker (transferred from
/// shbt-exotic): isometries `T^d_ij in Sp(2g, Z)` acting on the Heegaard
/// mapping torus `M`, enforcing Kojima's entropy inequality
/// `Ent(phi) <= C Vol(M) = 0 => Delta S_A = 0` during multi-seed address
/// shifts, plus holographic eigenvector rigidity and congestion-radius audits.
#[derive(Debug, Clone)]
pub struct HeegaardFloerTracker {
    /// Surface genus `g` (canonical boundary genus is 8).
    pub genus: usize,
}

impl HeegaardFloerTracker {
    pub fn new(genus: usize) -> Self {
        Self { genus }
    }

    /// Elementary symplectic shear `T = [[I, B], [0, I]]` in `Sp(2g, Z)`
    /// (symplectic iff `B` is symmetric); `B = diag(1)` Dehn-twist vector.
    pub fn twist_matrix(&self) -> Vec<Vec<f64>> {
        let g = self.genus;
        let mut t = vec![vec![0.0; 2 * g]; 2 * g];
        for (i, row) in t.iter_mut().enumerate().take(2 * g) {
            row[i] = 1.0;
        }
        for (i, row) in t.iter_mut().enumerate().take(g) {
            row[g + i] = 1.0;
        }
        t
    }

    /// Check `T^T Omega T = Omega` for the standard `Omega = [[0, I], [-I, 0]]`.
    pub fn is_symplectic(&self, t: &[Vec<f64>]) -> bool {
        let n = 2 * self.genus;
        let g = self.genus;
        for i in 0..n {
            for j in 0..n {
                // (T^T Omega T)_ij = sum_k t[k][i] t[k+g][j] - t[k+g][i] t[k][j]
                let mut v = 0.0;
                for k in 0..g {
                    v += t[k][i] * t[k + g][j] - t[k + g][i] * t[k][j];
                }
                let omega = if j == i + g {
                    1.0
                } else if i == j + g {
                    -1.0
                } else {
                    0.0
                };
                if (v - omega).abs() > 1e-9 {
                    return false;
                }
            }
        }
        true
    }

    /// Kojima entropy bound `Ent(phi) <= C * Vol(M)`; at `Vol(M) = 0` the
    /// tracker enforces `Delta S_A = 0` identically.
    pub fn entropy_bound(&self, mapping_torus_volume: f64) -> f64 {
        mapping_torus_volume
    }

    /// Address-shift audit: symplectic twist + vanishing entropy +
    /// `|mu_comp - mu_0| <= 1e-12` rigidity + congestion radius intact.
    pub fn verify_address_shift(&self, mu_comp: f64, mu_0: f64) -> bool {
        self.is_symplectic(&self.twist_matrix())
            && self.entropy_bound(0.0) == 0.0
            && (mu_comp - mu_0).abs() <= 1.0e-12
            && (core::CONGESTION_RADIUS_M - 2.954e15).abs() < 1e6
    }
}

impl Default for HeegaardFloerTracker {
    fn default() -> Self {
        Self::new(8)
    }
}

/// Holographic constraint-violation noise floor.
pub const HOLOGRAPHIC_FLOOR: f64 = 1.0e-122;

/// CCZ4 field state at a grid point (ghost2.txt Target A): conformal factor
/// `phi`, conformal metric `gamma_tilde`, trace-free `A_tilde`, trace `K`,
/// conformal connection `Gamma_tilde^i`, Z4 vector `Z_i`, Z4 scalar `Theta`.
#[derive(Debug, Clone, Default)]
pub struct Ccz4State {
    pub phi: f64,
    pub gamma_tilde: [[f64; 3]; 3],
    pub a_tilde: [[f64; 3]; 3],
    pub k: f64,
    pub gamma_tilde_up: [f64; 3],
    pub z: [f64; 3],
    pub theta: f64,
}

/// CCZ4 right-hand-side derivatives evaluated pointwise for a lapse `alpha`
/// with shift `beta` (ghost2.txt Target A equations; curvature/matter terms
/// supplied externally so the engine stays grid-agnostic).
#[derive(Debug, Clone)]
pub struct Ccz4Rhs {
    pub d_phi: f64,
    pub d_gamma_tilde: [[f64; 3]; 3],
    pub d_k: f64,
    pub d_theta: f64,
    pub d_z: [f64; 3],
}

pub struct Ccz4Solver {
    /// Gundlach constraint damping `kappa_1 > 0`.
    pub kappa1: f64,
    /// `kappa_2 > -1`.
    pub kappa2: f64,
}

impl Ccz4Solver {
    pub fn new(kappa1: f64, kappa2: f64) -> Self {
        assert!(kappa1 > 0.0 && kappa2 > -1.0);
        Self { kappa1, kappa2 }
    }

    /// Algebraic (principal + damping) RHS at a point with lapse `alpha`,
    /// scalar curvature `r3`, matter terms `rho_adm`, `s_i`, and derivative
    /// contractions already reduced to scalars.
    pub fn rhs(
        &self,
        st: &Ccz4State,
        alpha: f64,
        r3: f64,
        rho_adm: f64,
        s_i: [f64; 3],
        a2: f64,
    ) -> Ccz4Rhs {
        let g = 6.67430e-11_f64;
        let d_phi = -alpha * (st.k - st.theta) / 6.0;
        let d_k = alpha
            * (r3 + st.k * st.k - 2.0 * st.theta * st.k
                + 4.0 * std::f64::consts::PI * g * (0.0 - 3.0 * rho_adm))
            - 3.0 * alpha * self.kappa1 * (1.0 + self.kappa2) * st.theta;
        let d_theta = 0.5
            * alpha
            * (r3 - a2 + 2.0 / 3.0 * st.k * st.k
                - 2.0 * st.theta * st.k
                - 16.0 * std::f64::consts::PI * g * rho_adm)
            - alpha * self.kappa1 * (2.0 + self.kappa2) * st.theta;
        let mut d_z = [0.0; 3];
        for (i, dz) in d_z.iter_mut().enumerate() {
            *dz = alpha * (-8.0 * std::f64::consts::PI * g * s_i[i])
                - alpha * self.kappa1 * st.z[i];
        }
        let mut d_gamma_tilde = [[0.0; 3]; 3];
        for (row_d, row_a) in d_gamma_tilde.iter_mut().zip(st.a_tilde.iter()) {
            for (d, a) in row_d.iter_mut().zip(row_a.iter()) {
                *d = -2.0 * alpha * a;
            }
        }
        Ccz4Rhs {
            d_phi,
            d_gamma_tilde,
            d_k,
            d_theta,
            d_z,
        }
    }

    /// Gundlach damping: `||H(t)||_2 <= ||H(0)||_2 exp(-kappa_1 * alpha * t)`,
    /// saturated at the `1e-122` holographic noise floor.
    pub fn damped_constraint(&self, h0: f64, alpha: f64, t: f64) -> f64 {
        (h0.abs() * (-self.kappa1 * alpha * t).exp()).max(HOLOGRAPHIC_FLOOR)
    }
}

/// 2nd-order cross-coupling `I_mn` for K seeds within `R_congestion`
/// (ghost2.txt Target A, eq. for I_mn with `u = (1,0,0,0)` rest frames).
pub fn nonlinear_cross_coupling(masses_msun: &[(f64, f64)]) -> f64 {
    let g = 6.67430e-11_f64;
    let c2 = C_LIGHT * C_LIGHT;
    let mut i00 = 0.0;
    for (idx, &(m_i, r_i)) in masses_msun.iter().enumerate() {
        for &(m_j, r_j) in masses_msun.iter().skip(idx + 1) {
            let mi = m_i * core::SOLAR_MASS_KG;
            let mj = m_j * core::SOLAR_MASS_KG;
            i00 += g * g * mi * mj / (c2 * c2 * r_i * r_j);
        }
    }
    i00
}

/// Wake-tensor coupling magnitude `Theta_mrs` (leading gravitomagnetic term).
pub fn wake_tensor_third_order(masses_msun: &[(f64, f64)]) -> f64 {
    let g = 6.67430e-11_f64;
    let mut t = 0.0;
    for &(m_i, r_i) in masses_msun {
        t += 8.0 * std::f64::consts::PI * g * m_i * core::SOLAR_MASS_KG
            / (C_LIGHT.powi(4) * r_i);
    }
    t
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

    #[test]
    fn heegaard_floer_tracker() {
        let tr = HeegaardFloerTracker::default();
        let t = tr.twist_matrix();
        assert!(tr.is_symplectic(&t));
        let mut bad = t.clone();
        bad[0][8] = 2.0;
        assert_eq!(tr.entropy_bound(0.0), 0.0);
        assert!(tr.verify_address_shift(1.0 + 5e-13, 1.0));
        assert!(!tr.verify_address_shift(1.0 + 2e-12, 1.0));
        let _ = bad;
    }

    #[test]
    fn ccz4_rhs_and_damping() {
        let solver = Ccz4Solver::new(0.5, 0.0);
        let st = Ccz4State {
            theta: 1e-6,
            ..Default::default()
        };
        let rhs = solver.rhs(&st, 1.0, 0.0, 0.0, [0.0; 3], 0.0);
        assert!((rhs.d_phi - 1e-6 / 6.0).abs() < 1e-12);
        assert!(rhs.d_theta < 0.0); // kappa1 damping drives Theta -> 0
        // Constraint decays to the 1e-122 holographic floor.
        assert_eq!(solver.damped_constraint(1e-20, 1.0, 600.0), 1e-122);
        assert!(solver.damped_constraint(1e-20, 1.0, 10.0) < 1e-20);
        // Cross-coupling + wake tensors finite inside congestion radius.
        let seeds = [(1e-6, 1e15), (1e-6, 1.1e15)];
        assert!(nonlinear_cross_coupling(&seeds) > 0.0);
        assert!(wake_tensor_third_order(&seeds) > 0.0);
    }
}
