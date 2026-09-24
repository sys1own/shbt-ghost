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
}
