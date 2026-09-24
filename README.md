# shbt-ghost

Multi-physics digital twin simulator for synthetic **ghost seed** propulsion,
multi-seed artificial gravity, and gravitational optics under Static
Holographic Boundary Theory (SHBT).

## System physics

When local boundary information processing exceeds the holographic ceiling
`N_limit`, a topological overflow `DeltaN = N_local - N_limit` produces an
effective ghost-seed mass

```
M_seed = alpha_seed * DeltaN ,  alpha_seed = d1 * m_P / N_total = 26 m_P / e^33
       = 1.3258316e-51 M_sun/bit      (canonical branch (26, 8, 312), 512-bit MPFR)
```

Sustaining the baseline `M_seed = 1e-6 M_sun` requires `DeltaN_0 =
7.542426e44` bits and dissipates a continuous Landauer entropy debt

```
P_debt = (M_seed / M_sun) * 906 GW  =  906.00 kW
```

supplied by a 1,800-module LANR plant delivering `999.054 kW_net`
(`555.03 W/module`, floor `N_min = 1633`, `N+167` reserve, 33.804% TEG).
Kinematic congestion wakes are cancelled by third-order momentum
compensation enforcing `|mu_comp - mu_0| <= 1e-12`; propulsion executes via
forward offset vectoring and discrete bit-stepping
`Delta N(k) = floor(DeltaP_net / P_bit)`; habitat gravity is synthesized by
K-seed metric superposition `g_mn = eta_mn + sum h_mn^(i) + I_mn`
(`I_00 = +2.4189e-32`, `I_kk = -I_00/3`), non-rotational `9.80665 m/s^2`
floor gravity, zero Coriolis distortion, and `det`/`Gram` ADM audits.
Synthetic optics span `f0 = 169.30 m` to `f_max = 1692.99 m` with Bessel
`J0^2` caustics and `C <= 1e-10` coronagraph rejection. Causality is
enforced by harmonic 2PN lightcone authorization `Delta s^2 <= 0` and a
sub-2.50 ns GaN current-shunt quench (`tau <= 2.18 ns`, 94.20% SiC
recovery).

## Workspace architecture

Eight specialized crates under `crates/` (Cargo `resolver = "2"`):

| Crate | Domain | Key invariant |
|---|---|---|
| `ghost-core-engine` | 512-bit mass-congestion topology | `alpha_seed`, `N_total = e^33`, `R_congestion = 2.954e15 m` |
| `ghost-propulsion-drive` | Reactionless traction & TMSV metrology | `Delta N = floor(dP/P_bit)`, `r = 2.50` (21.715 dB) |
| `ghost-multiseed-gravity` | Metric superposition & 1g floor | `I_00 = 2.4189e-32`, `g_z = 9.80665`, `ds^2 <= 0` |
| `ghost-kinematic-wake` | Wake Lagrangian & minimum-jerk | `|mu_comp - mu_0| <= 1e-12`, `max s'' = 5.7735` |
| `ghost-optics-lensing` | Gravitational optics | `f0 = 169.30 m`, `C <= 1e-10`, `J0^2` |
| `ghost-lanr-interface` | LANR power ledger | `999.054 kW`, floor `1633`, `94.20%` SiC |
| `ghost-hil-microkernel` | C11 bare-metal FFI bridge | MMIO `0x70000000`, SECDED Hamming(72,64) |
| `ghost-gpu-acceleration` | CUDA/ROCm + WebGPU | `>= 100 Hz @ 4K`, `>100 GB/s`, `504 Gbps`, `60 FPS` |

## Microkernel

- `kernel/include/shbt_hardware.h` — 56-byte `SHBT-MMIO-1` register map at
  `0x70000000` (word offsets `0x00`–`0x34`).
- `kernel/linker.ld` — 2,112-byte `.stinespring_frame` SRAM arena on a
  64-byte boundary (640 B active residual `eta_A = 10/33`, 1,472 B dark
  ledger `eta_D = 23/33`, 124 Fibonacci braid descriptors).
- `kernel/shbt_ghost_kernel.c` — `shbt_ghost_kernel_init`,
  `shbt_compute_secded_ecc`, `shbt_verify_and_correct_secded`,
  `shbt_trigger_quench_interlock`, `shbt_avx512_givens_remapping`.

## CLI

```bash
python3 python/shbt_ghost/cli/main.py build-kernel   # -> build/shbt_reference.so
python3 python/shbt_ghost/cli/main.py sim            # co-simulation epoch
python3 python/shbt_ghost/cli/main.py verify         # -> 70-gate JSON on stdout
```

## Verification

```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 tests/run_all_tests.py
python3 python/shbt_ghost/cli/main.py verify > verification_matrix.json
```

`verification_matrix.json` records all 70 gates (`GATE-01`–`GATE-70`) with
`"status": "PASS"`.

## Technology transfer

Modules are adapted from `shbt-exotic` (mass-congestion, superposition,
wake compensation), `shbt-recon` (2PN metrics, stackups, substrates,
Stinespring dilation), `shbt-sglt` (TMSV metrology, minimum-jerk, lensing),
`shbt-cf` (LANR grid, SiC crowbars), `shbt-precision` (canonical branch
arithmetic, MPFR state evaluation, Landauer accounting), and `shbt-qc`
(C11 microkernel, SHBT-MMIO-1, SECDED).
