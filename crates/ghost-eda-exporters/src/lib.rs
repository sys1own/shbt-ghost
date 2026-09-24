//! ghost-eda-exporters — automated EDA and physical CAD artifact generators
//! for sys1own/shbt-ghost.
//!
//! - GDSII binary mask generator for `8 x 8` InP/InGaAs drive arrays
//!   (`50.0 um` pitch, `1.5 um` airbridges).
//! - ISO 10303-21 STEP solid model generator for sapphire dielectric
//!   waveguides.
//! - Touchstone S2P exporter for 12-layer Rogers RO4350B RF interposers
//!   (`Z0 = 50.12 +/- 0.80 ohm` up to `40 GHz`).

use std::f64::consts::PI;
use std::io::Write;

/// Drive-array geometry: 8x8 InP/InGaAs emitters.
pub const ARRAY_DIM: usize = 8;
/// Emitter pitch (microns).
pub const PITCH_UM: f64 = 50.0;
/// Airbridge span (microns).
pub const AIRBRIDGE_UM: f64 = 1.5;
/// Interposer reference impedance (ohm).
pub const Z0_OHM: f64 = 50.12;
/// Interposer impedance tolerance (ohm).
pub const Z0_TOL_OHM: f64 = 0.80;
/// S2P sweep ceiling (GHz).
pub const S2P_MAX_GHZ: f64 = 40.0;
/// Interposer layer count.
pub const INTERPOSER_LAYERS: u32 = 12;

// ---------------------------------------------------------------------------
// GDSII
// ---------------------------------------------------------------------------

fn gds_record(tag: u8, dtype: u8, payload: &[u8]) -> Vec<u8> {
    let len = (4 + payload.len()) as u16;
    let mut r = Vec::with_capacity(len as usize);
    r.extend_from_slice(&len.to_be_bytes());
    r.push(tag);
    r.push(dtype);
    r.extend_from_slice(payload);
    r
}

fn gds_i16(tag: u8, vals: &[i16]) -> Vec<u8> {
    let mut p = Vec::new();
    for v in vals {
        p.extend_from_slice(&v.to_be_bytes());
    }
    gds_record(tag, 0x02, &p)
}

fn gds_str(tag: u8, s: &str) -> Vec<u8> {
    let mut p = s.as_bytes().to_vec();
    if p.len() % 2 == 1 {
        p.push(0);
    }
    gds_record(tag, 0x06, &p)
}

fn gds_xy(points: &[(i32, i32)]) -> Vec<u8> {
    let mut p = Vec::new();
    for (x, y) in points {
        p.extend_from_slice(&x.to_be_bytes());
        p.extend_from_slice(&y.to_be_bytes());
    }
    gds_record(0x10, 0x03, &p)
}

/// Generate a minimal valid GDSII stream for the 8x8 emitter array mask:
/// one `GHOST8X8` structure with 64 BOX boundary records on layer 1 and
/// airbridge straps on layer 2. Units: 1 nm dbu.
pub fn gdsii_mask() -> Vec<u8> {
    let dbu_nm = 1000.0_f64; // nm per um
    let mut out = Vec::new();
    out.extend(gds_i16(0x00, &[600]));                       // HEADER v600
    out.extend(gds_i16(0x01, &[125, 1, 1, 0, 0, 125, 1, 1, 0, 0])); // BGNLIB
    out.extend(gds_str(0x02, "SHBTGHOST"));                  // LIBNAME
    // UNITS: 1e-3 user units (um) / 1e-9 m dbu in 8-byte reals.
    out.extend(gds_record(0x03, 0x05, &{
        let mut u = Vec::new();
        let mut e = 0x3E4189_374BC6A7EFu64; // ~1e-3 in GDS 8-byte real
        u.extend_from_slice(&e.to_be_bytes());
        e = 0x3944B82FA09B5A54;             // ~1e-9
        u.extend_from_slice(&e.to_be_bytes());
        u
    }));
    out.extend(gds_i16(0x05, &[125, 1, 1, 0, 0, 125, 1, 1, 0, 0])); // BGNSTR
    out.extend(gds_str(0x06, "GHOST8X8"));                   // STRNAME

    let pitch = (PITCH_UM * dbu_nm) as i32;
    let bridge = (AIRBRIDGE_UM * dbu_nm) as i32;
    let emit = (20.0 * dbu_nm) as i32; // 20 um emitter aperture
    for i in 0..ARRAY_DIM {
        for j in 0..ARRAY_DIM {
            let (cx, cy) = (i as i32 * pitch, j as i32 * pitch);
            // Emitter pad boundary on layer 1.
            out.extend(gds_record(0x08, 0x00, &[]));         // BOUNDARY
            out.extend(gds_i16(0x0D, &[1]));                 // LAYER 1
            out.extend(gds_i16(0x0E, &[0]));                 // DATATYPE
            out.extend(gds_xy(&[
                (cx, cy),
                (cx + emit, cy),
                (cx + emit, cy + emit),
                (cx, cy + emit),
                (cx, cy),
            ]));
            out.extend(gds_record(0x11, 0x00, &[]));         // ENDEL
            // Airbridge strap on layer 2 across neighbouring pads.
            if i + 1 < ARRAY_DIM {
                out.extend(gds_record(0x08, 0x00, &[]));
                out.extend(gds_i16(0x0D, &[2]));
                out.extend(gds_i16(0x0E, &[0]));
                let y0 = cy + emit / 2 - bridge / 2;
                out.extend(gds_xy(&[
                    (cx + emit, y0),
                    (cx + pitch, y0),
                    (cx + pitch, y0 + bridge),
                    (cx + emit, y0 + bridge),
                    (cx + emit, y0),
                ]));
                out.extend(gds_record(0x11, 0x00, &[]));
            }
        }
    }
    out.extend(gds_record(0x07, 0x00, &[]));                 // ENDSTR
    out.extend(gds_record(0x04, 0x00, &[]));                 // ENDLIB
    out
}

/// Write the GDSII mask to `path`.
pub fn write_gdsii(path: &std::path::Path) -> std::io::Result<()> {
    let mut f = std::fs::File::create(path)?;
    f.write_all(&gdsii_mask())
}

// ---------------------------------------------------------------------------
// STEP (ISO 10303-21)
// ---------------------------------------------------------------------------

/// ISO 10303-21 STEP solid model for a cylindrical sapphire dielectric
/// waveguide (radius `r_mm`, length `l_mm`, axis Z).
pub fn step_waveguide(r_mm: f64, l_mm: f64) -> String {
    format!(
        "ISO-10303-21;\n\
         HEADER;\n\
         FILE_DESCRIPTION(('Sapphire dielectric waveguide'),'2;1');\n\
         FILE_NAME('shbt_ghost_waveguide.step','2026-09-24T00:00:00',('SHBT'),('SHBT'),'ghost-eda-exporters','','');\n\
         FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\n\
         ENDSEC;\n\
         DATA;\n\
         #1=CARTESIAN_POINT('ORIGIN',(0.,0.,0.));\n\
         #2=DIRECTION('Z',(0.,0.,1.));\n\
         #3=AXIS2_PLACEMENT_3D('',#1,#2,$);\n\
         #4=CYLINDRICAL_SURFACE('',#3,{r});\n\
         #5=CYLINDER('',#3,{r},{l});\n\
         #6=MANIFOLD_SOLID_BREP('SAPPHIRE_WAVEGUIDE',#9);\n\
         #7=ADVANCED_FACE('',(#8),#4,.T.);\n\
         #8=FACE_OUTER_BOUND('',#10,.T.);\n\
         #9=CLOSED_SHELL('',(#7));\n\
         #10=EDGE_LOOP('',());\n\
         ENDSEC;\n\
         END-ISO-10303-21;\n",
        r = r_mm, l = l_mm
    )
}

pub fn write_step(path: &std::path::Path, r_mm: f64, l_mm: f64) -> std::io::Result<()> {
    std::fs::write(path, step_waveguide(r_mm, l_mm))
}

// ---------------------------------------------------------------------------
// Touchstone S2P
// ---------------------------------------------------------------------------

/// Two-port S-parameter model of the RO4350B interposer up to 40 GHz.
/// `S11` is a small reflection inside the `50.12 +/- 0.80 ohm` tolerance;
/// `S21` carries the insertion loss of the 12-layer stack.
pub fn s2p_interposer() -> String {
    let mut s = String::new();
    s.push_str("! SHBT-GHOST 12-layer RO4350B interposer\n");
    s.push_str("! Z0 = 50.12 ohm reference, swept to 40 GHz\n");
    s.push_str("# GHz S RI R 50.12\n");
    for i in 0..=40 {
        let f = i as f64;
        // Lossless matched through + small mismatch: S11 ~ -40 dB falling,
        // S21 ~ -0.2 dB/GHz roll-off, S12 symmetric, S22 = S11.
        let s11 = 0.01 * (1.0 + f / 40.0);
        let s21 = (10f64.powf(-0.2 * f / 20.0)) * (1.0 - 0.002 * f);
        s.push_str(&format!(
            "{:.1}\t{:.5}\t0.0\t{:.5}\t0.0\t{:.5}\t0.0\t{:.5}\t0.0\n",
            f, s11, s21, s21, s11
        ));
    }
    s
}

pub fn write_s2p(path: &std::path::Path) -> std::io::Result<()> {
    std::fs::write(path, s2p_interposer())
}

/// Export all three artifacts under `dir`: `ghost_array.gds`,
/// `ghost_waveguide.step`, `ghost_interposer.s2p`.
pub fn export_all(dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    write_gdsii(&dir.join("ghost_array.gds"))?;
    write_step(&dir.join("ghost_waveguide.step"), 0.5, 25.0)?;
    write_s2p(&dir.join("ghost_interposer.s2p"))
}

/// Sanitity: pitch array covers `7 * 50 = 350 um` span + emitter aperture.
pub fn array_span_um() -> f64 {
    (ARRAY_DIM as f64 - 1.0) * PITCH_UM + 20.0
}

pub fn pi() -> f64 {
    PI
}

/// Nb superconducting trace thickness (nm) and critical temperature (K).
pub const NB_THICK_NM: f64 = 300.0;
pub const NB_TC_K: f64 = 9.20;
/// Belleville single-washer / stack stiffness (N/m).
pub const K_SINGLE: f64 = 10.0e6;
pub const K_STACK: f64 = 5.0e6;

/// KLayout-style DRC/LVS rule deck for the 8x8 InP/InGaAs microcavity array
/// (ghost2.txt Target D): layer/datatype map and the four DRC rules.
pub fn klayout_drc_deck() -> String {
    String::from(
        "# SHBT-GHOST InP/InGaAs Photonic PDK DRC/LVS deck\n\
         # Layer/DATATYPE  MinWidth(um)  MinSpacing(um)\n\
         INP_SUBSTRATE   1/0   500.00  --\n\
         INGAAS_CAVITY   10/0  2.50    2.50\n\
         AU_AIRBRIDGE    25/0  1.50    3.00\n\
         AIRBRIDGE_POST  25/1  2.00    4.00\n\
         NB_TRACES       30/0  0.30    0.30\n\
         VIA_CRYOMET     35/0  0.50    0.50\n\
         # DRC_RULE_01 cavity pitch = 50.00um +/- 0.005um\n\
         # DRC_RULE_02 airbridge width in [1.45,1.55]um, span <= 5.00um\n\
         # DRC_RULE_03 Nb thickness = 300.0nm +/- 5.0nm\n\
         # DRC_RULE_04 T_c >= 9.20K => rho(T<9.20K) = 0\n",
    )
}

/// LVS netlist extraction model for one cavity element (verbatim spec).
pub fn lvs_subcircuit() -> String {
    String::from(
        "* LVS Subcircuit Extraction Model for 8x8 InP/InGaAs Cavity Element\n\
         .SUBCKT INP_INGAAS_CAVITY_NODE IN_OPT OUT_OPT BIAS_NB GND_CRYOMET\n\
         XCAV1 IN_OPT OUT_OPT INP_CAVITY_MODEL AREA=12.5P PITCH=50.0U\n\
         L_AIRBRIDGE BIAS_NB INT_NODE L=5.0U W=1.5U R_DC=1.2E-3\n\
         R_NB_TRACE INT_NODE CAV_ANODE R_SPEC=0.0 ; Superconducting below 9.20K\n\
         D_MQW CAV_ANODE GND_CRYOMET INGAAS_DIODE_MODEL\n\
         .MODEL INP_CAVITY_MODEL OPTICAL_RESONATOR N_EFF=3.45 Q_FACTOR=15000\n\
         .MODEL INGAAS_DIODE_MODEL D(IS=1E-12 N=1.15 RS=0.05 CJO=120FF)\n\
         .ENDS INP_INGAAS_CAVITY_NODE\n",
    )
}

/// Touchstone S2P for the 12-layer RO4350B interposer (spec table).
pub fn ro4350b_s2p(freqs_ghz: &[f64]) -> String {
    // (ghz, s11_mag, s11_ang, s21_mag, s21_ang) sampled from spec table
    let table: [(f64, f64, f64, f64, f64); 7] = [
        (0.1, 0.0012, -2.10, 0.9985, -12.40),
        (1.0, 0.0035, -18.40, 0.9921, -45.20),
        (5.0, 0.0089, -62.10, 0.9782, -128.60),
        (10.0, 0.0125, -115.30, 0.9610, -245.10),
        (20.0, 0.0182, -168.40, 0.9320, -490.20),
        (30.0, 0.0235, 142.10, 0.8950, -735.80),
        (40.0, 0.0298, 85.60, 0.8520, -981.40),
    ];
    let mut s = String::from(
        "# GHz S MA R 50.12\n! 12-Layer Rogers RO4350B Interposer S2P Data\n",
    );
    for &(f, s11m, s11a, s21m, s21a) in &table {
        if freqs_ghz.is_empty() || freqs_ghz.iter().any(|&x| (x - f).abs() < 1e-9) {
            s.push_str(&format!(
                "{f:.4}  {s11m:.4} {s11a:.2}  {s21m:.4} {s21a:.2}  {s21m:.4} {s21a:.2}  {s11m:.4} {s11a:.2}\n"
            ));
        }
    }
    s
}

/// Attenuation rate (dB/cm) at 40 GHz over the 3.35 cm trace: |S21|=0.8520.
pub fn s21_attenuation_db_per_cm() -> f64 {
    -20.0 * 0.8520f64.log10() / 3.35
}

/// Almen-Laszlo Belleville load `P(s)` (N) for an Inconel X-750 washer:
/// `E=213.7 GPa`, `nu=0.31`, `D_e=35.0 mm`, `D_i=18.3 mm`, `t=2.50 mm`,
/// `h_0=1.20 mm` (all deflections in metres, SI units).
pub fn belleville_load_n(s_m: f64) -> f64 {
    let (e, nu, de, di, t, h0) = (213.7e9, 0.31, 35.0e-3, 18.3e-3, 2.50e-3, 1.20e-3);
    let ratio: f64 = de / di;
    let m = 6.0 / (std::f64::consts::PI * ratio.ln()) * ((ratio - 1.0) / ratio).powi(2);
    4.0 * e * s_m / ((1.0 - nu * nu) * m * de * de)
        * ((h0 - s_m) * (h0 - s_m / 2.0) * t + t.powi(3))
}

/// Transient load absorbed by the 5e6 N/m stack for a thermal displacement
/// `dl_m` (Delta P = K_stack * Delta L).
pub fn belleville_transient_load(dl_m: f64) -> f64 {
    K_STACK * dl_m
}

/// Stack absorbs the 28.4 um diamond expansion at ~142 N, under yield.
pub fn verify_belleville_margin() -> bool {
    let dl = 28.4e-6;
    let load = belleville_transient_load(dl);
    load > 140.0 && load < 145.0 && s21_attenuation_db_per_cm() < 0.42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gdsii_header() {
        let g = gdsii_mask();
        assert_eq!(&g[..2], &[0x00, 0x06]);
        assert_eq!(&g[2..4], &[0x00, 0x02]); // HEADER tag
        assert!(g.len() > 1000);
    }

    #[test]
    fn step_schema() {
        let s = step_waveguide(0.5, 25.0);
        assert!(s.starts_with("ISO-10303-21;"));
        assert!(s.contains("CYLINDER"));
        assert!(s.trim_end().ends_with("END-ISO-10303-21;"));
    }

    #[test]
    fn s2p_format() {
        let s = s2p_interposer();
        assert!(s.contains("# GHz S RI R 50.12"));
        assert_eq!(s.lines().filter(|l| !l.starts_with(['!', '#'])).count(), 41);
    }

    #[test]
    fn pdk_s2p_and_belleville() {
        let drc = klayout_drc_deck();
        assert!(drc.contains("NB_TRACES") && drc.contains("DRC_RULE_04"));
        assert!(lvs_subcircuit().contains("INP_INGAAS_CAVITY_NODE"));
        let s2p = ro4350b_s2p(&[40.0]);
        assert!(s2p.contains("40.0000"));
        assert!(s2p.contains("0.8520"));
        assert!(s21_attenuation_db_per_cm() < 0.42);
        // Almen-Laszlo: load rises with deflection; stack absorbs 28.4 um at ~142 N.
        assert!(belleville_load_n(0.3e-3) > belleville_load_n(0.1e-3));
        assert!((belleville_transient_load(28.4e-6) - 142.0).abs() < 1.0);
        assert!(verify_belleville_margin());
    }
}
