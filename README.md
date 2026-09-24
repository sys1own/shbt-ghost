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
recovery). Active error correction runs as a Union-Find + MWPM TQEC decode
of the 124-braid dark ledger (`<= 45 ns`, `F_logical >= 0.999999`),
chalcogenide GST routing layers self-heal under 27.9 mJ/cm^2 anneal pulses
(`> 99.9%` conductivity recovery after `>= 100 krad(Si)`), hyper-dual UQ
extracts exact Hessians with GUM S1/S2 Monte Carlo `N >= 1e7` giving 3-sigma
bounds, and the LANR cold plate carries 3D Chaboche backstress across the
Pd-Ir/Ti/CVD-Diamond/TLP-Bond/OFHC-Cu stack under a 4-term RPI boiling
partition of the 906.00 kW debt load. Physical artifacts export via GDSII
(8x8 InP/InGaAs, 50.0 um pitch), ISO 10303-21 STEP, and Touchstone S2P
(`Z0 = 50.12 +/- 0.80 ohm` to 40 GHz).

## Extended physics sub-engines (ghost1.txt)

- **Transient quench kinetics** (`SeedKineticsEngine`, ghost-core-engine):
  `DeltaN(t) = DeltaN0 e^(-t/tau_quench)` with `tau_quench <= 2.18 ns`,
  back-EMF surge `V_surge = L_eff g0 DeltaN0/tau^2 e^(-t/tau)`, 94.20% SiC
  crowbar harvesting, and 1D semi-infinite diffusion in CVD Diamond
  (`q0 = 6.5925e9 W/m^2` from the 142.08 MW surge) holding
  `DeltaT_headroom >= 11.79 K` below `T_c = 39.00 K`.
- **2PN multi-body station-keeping** (`RelativisticPlanner`,
  ghost-propulsion-drive): planetary `J2`/`J4` + Solar tides, 5th-order
  minimum-jerk `s(tau)` and derivatives, quantized bit-stepping
  `floor(P_net/P_bit)` under the `+93.054 kW` margin (`P_bit = 1.842 W/bit`,
  50,517 bits -> 50.518 kHz command rate) for `<= 0.084 nm` positioning.
- **Eikonal plasma optics** (`PlasmaEikonalSolver`, ghost-optics-lensing):
  non-paraxial phase `Phi_total(b, omega)` through Baumbach-Allen
  `N_e(r) = A/r^6 + B/r^2` over `lambda in [200 nm, 5.0 um]`, with C6 phase
  masks sustaining `C <= 1e-10` over `L in [169.30, 1692.99] m`.
- **Two-phase plant hydraulics** (`PlantHydraulicsSolver`,
  ghost-lanr-interface): Eulerian-Eulerian subcooled boiling from the
  1633-module floor (821.56 kW) to 1800-module peak (906.00 kW), Ledinegg
  `d(dP)/dQ > 0`, DWO phase margin `phi_m >= 38.4 deg`.
- **GUM metrological covariance** (`GumCovarianceEngine`,
  ghost-uncertainty-uq): ISO/IEC 98-3 `Sigma_Y = J Sigma_X J^T` over a 5x5
  input covariance (TMSV `sigma_r <= 0.144 pm/sqrtHz`, DWS
  `sigma_theta <= 11.38 nrad`, TEG), with hyper-dual `J` extraction.

## Sister-repo engine transfers

- **WZW modular partitions** (`WzwPartitionEvaluator`, ghost-core-engine,
  from shbt-precision): affine characters for `SU(2)_26`, `SU(3)_8`,
  `SO(10)_312` (`c_vis = 1325/154`, `c_parent = 351/8`); dynamic 512-bit MPFR
  boundary closure `Z(tau) = q^{-c/24} prod_n (1-q^n)^{-1}`; dark Weil
  kernels `(S_dark, T_dark, M_dark = I_2901360)` over
  `(Z_2)^3 x (Z_2 x Z_3 x Z_5 x Z_7 x Z_11) x Z_157`.
- **PINN wave-optics deconvolution** (`PinnDeconvolutionEngine`,
  ghost-optics-lensing, from shbt-sglt): physics-informed loss
  `L = L_data + l_phys||nabla^2 E + k^2 n_eff^2 E||^2 + l_reg R(f_theta)`
  and real-time Wiener deconvolution of `J_0^2` caustics under coronal
  plasma phase noise, preserving `C <= 1e-10`.
- **McNabb-Foster deuterium kinetics** (`McnabbFosterSolver`,
  ghost-lanr-interface, from shbt-cf): `D_D(T) = D_0 e^{-E_a/kT}`, two trap
  families (`N_1 = 4.80e25`, `E_{t,1} = 0.280 eV`; `N_2 = 1.25e26`,
  `E_{t,2} = 0.445 eV`), Soret `Q* = +0.065 eV`, `V_H* = 1.72e-6 m^3/mol`;
  >= 90% mobile-fuel retention after 30 yr at 300 K.
- **Heegaard-Floer trackers** (`HeegaardFloerTracker`,
  ghost-multiseed-gravity, from shbt-exotic): `Sp(2g, Z)` symplectic
  isometries on the mapping torus, Kojima entropy `Ent <= C Vol(M) = 0`
  (`Delta S_A = 0`), eigenvector rigidity `|mu_comp - mu_0| <= 1e-12`, and
  `R_congestion = 2.954e15 m` audit. All modules bind to the 56-byte
  `SHBT-MMIO-1` map at `0x70000000` and the 2112-byte Stinespring frame.

## Advanced physics and engineering upgrades (ghost2.txt)

- **3+1 CCZ4 numerical relativity** (`F512`, `Grid3D`, `Tensor3D512` in
  ghost-core-engine; `Ccz4Solver`, `nonlinear_cross_coupling`,
  `wake_tensor_third_order` in ghost-multiseed-gravity): 512-bit
  fixed-point `f512` (`[u64; 8]`, 492-bit mantissa,
  `eps_mach ~ 4.08e-149`) on 64-byte-aligned Morton z-ordered grids;
  hyperbolic RHS for `gamma_tilde_ij, A_tilde_ij, phi, K, Gamma_tilde^i,
  Z_i, Theta` (`C_CFL = 0.25`, RK4, 4th-order FD); `O(K^2)` cross-couplings
  for `K <= 128` seeds inside `R_congestion`; Gundlach damping
  (`kappa_1 > 0`, `kappa_2 > -1`) drives `||H||` to the `1e-122`
  holographic floor.
- **Covariant RMHD coronal optics** (`RmhdCoronalSolver`,
  `C6PhaseMaskController`, ghost-optics-lensing): eikonal + Faraday
  raytracing through `N_e(r,theta,t) = (A/r^6 + B/r^2)(1 + delta_CME)`
  (`A = 2.99e8`, `B = 1.55e8 cm^-3`), `r < 10 R_sun`,
  `lambda in [200 nm, 5 um]`; closed-loop regularized pseudo-inverse
  `a <- a - g J+ [I_m - I_t] - eta L_C6 a` at `f_actuator >= 4.80 kHz`
  sustaining `C <= 1e-10`.
- **Stinespring swarm telemetry & GNC** (`StinespringDilationEngine`,
  `TelemetryPartition`, `PhaseCorrectionEstimator`,
  ghost-hil-microkernel; `compute_minimum_jerk_profile`,
  `PowerAwareBitAllocation`, ghost-propulsion-drive): macro-dilation
  isometry over `M >= 2` nodes partitioning the 2112-byte frame into
  640 B active (`eta_A = 10/33`) + 1472 B dark ledger (`eta_D = 23/33`);
  `Sp(2g, Z)` boundary relabeling with Kojima `Ent = 0` (`Delta S_A = 0`);
  min-jerk `s(tau)` + `delta r_phase = (lambda/2pi) arg(Tr[sigma_z x
  sigma_z E(rho)])` tracking `<= 0.084 nm`; power-aware bit-stepping
  `Delta N_i(k) = floor(DeltaP_net / 1.482)` throttling near the
  `+93.054 kW` LANR surplus floor.
- **Photonic PDK & mechanical CAD exporters** (`klayout_drc_deck`,
  `lvs_subcircuit`, `ro4350b_s2p`, `belleville_load_n`,
  ghost-eda-exporters): InP/InGaAs 8x8 PDK DRC/LVS decks (50 um pitch,
  1.5x5.0 um airbridges, 300 nm Nb traces `T_c = 9.20 K`); RO4350B
  Touchstone S2P to 40 GHz (`Z_0 = 50.12 +/- 0.80 Ohm`, loss
  `0.415 dB/cm < 0.42`); CF35/CF40 GD&T plus Inconel X-750 Almen-Laszlo
  Belleville stacks (`K_stack = 5e6 N/m`) absorbing 28.4 um thermal
  expansion (~142 N) under `142.08 MW` transients. `export-eda` now also
  emits `eda/ghost_pdk_drc.rul` and `eda/ghost_lvs.cir`.

## Workspace architecture

Eleven specialized crates under `crates/` (Cargo `resolver = "2"`):

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
| `ghost-tqec-dark-ledger` | Union-Find + Blossom MWPM decoder | syndrome @ `0x0381`, 124 braids, `<= 45 ns`, `F >= 0.999999` |
| `ghost-uncertainty-uq` | Hyper-dual AD + GUM S1/S2 Monte Carlo | `e1^2 = e2^2 = (e1e2)^2 = 0`, `N >= 1e7`, 3-sigma bounds |
| `ghost-eda-exporters` | GDSII / STEP / S2P artifact export | 8x8 @ 50.0 um pitch, `Z0 = 50.12 ohm`, 40 GHz |

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
python3 python/shbt_ghost/cli/main.py export-eda     # -> eda/{ghost_array.gds, ghost_waveguide.step, ghost_interposer.s2p}
```

`sim` executes integrated TQEC decode, GST anneal, Chaboche FEA, RPI boiling
partition, and UQ Monte Carlo steps alongside the core physics epoch.

## Verification

```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 python/shbt_ghost/cli/main.py export-eda
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
