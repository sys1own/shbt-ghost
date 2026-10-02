/*
 * shbt_ghost_mmio.h — 128-byte dual-cacheline hardware contract for
 * sys1own/shbt-ghost (ghost3.txt upgrade of the legacy 56-byte
 * SHBT-MMIO-1 layout).
 *
 * Anchored at physical base 0x70000000, 64-byte aligned, protected by
 * SECDED Hamming(72,64) ECC and mapped for AVX-512 register operations.
 *
 *   Cacheline 0 (0x00-0x3F): command, control, bit stepping, DEC bus
 *     voltage (15 kV-400 kV), gross graser burst power, core/cold-plate
 *     temperatures, battery SoC, PCSS crowbar status, and syndrome.
 *   Cacheline 1 (0x40-0x7F): ADM lapse alpha (1.0), |det(g) + 1| error,
 *     shift norm ||beta^i||, TMSV squeezing r (2.50), mu_comp rigidity
 *     error, dark ledger braid count (124), quench latency, and syndrome.
 */
#ifndef SHBT_GHOST_MMIO_H
#define SHBT_GHOST_MMIO_H

#include <stdint.h>
#include <stddef.h>

#define SHBT_MMIO_BASE_ADDR 0x70000000UL

typedef enum {
    SHBT_STATE_STANDBY_STASIS   = 0x01,
    SHBT_STATE_TRIGGER_ARMED    = 0x02,
    SHBT_STATE_BURST_TRACTION   = 0x04,
    SHBT_STATE_DEC_COOLDOWN     = 0x08,
    SHBT_STATE_EMERGENCY_QUENCH = 0x10
} shbt_power_state_t;

/* 128-byte dual-cacheline hardware contract */
typedef struct __attribute__((aligned(64))) {
    /* Cacheline 0: Command, Control, and Telemetry (Offsets 0x00 - 0x3F) */
    volatile uint32_t ctrl_status;         /* 0x00: System status & state machine */
    volatile uint32_t trigger_delay_ps;    /* 0x04: Laser trigger sync delay (ps) */
    volatile uint64_t target_bits_step;    /* 0x08: Commanded Delta N(k) bits     */
    volatile uint64_t current_bits_step;   /* 0x10: Actual Delta N(k) stepped     */
    volatile double   dec_bus_voltage_v;   /* 0x18: DEC bus voltage (15kV-400kV)  */
    volatile double   gross_burst_power_w; /* 0x20: Instantaneous graser power (W)*/
    volatile float    core_temp_kelvin;    /* 0x28: Active core temperature (K)   */
    volatile float    cold_plate_temp_k;   /* 0x2C: Cold plate interface temp (K) */
    volatile uint16_t battery_soc_permille;/* 0x30: Isomer State of Charge (0-1000)*/
    volatile uint16_t pcss_crowbar_arm;    /* 0x32: PCSS quench interlock arm bits*/
    volatile uint32_t ecc_syndrome_c0;     /* 0x34: SECDED ECC syndrome line 0    */
    volatile uint64_t reserved_c0;         /* 0x38: Reserved pad to 64 bytes      */

    /* Cacheline 1: Metric Invariants and Safety Ledger (Offsets 0x40 - 0x7F) */
    volatile double   adm_lapse_alpha;     /* 0x40: Lapse function alpha (1.0)    */
    volatile double   det_g_error;         /* 0x48: |det(g) + 1| deviation        */
    volatile double   shift_norm_beta;     /* 0x50: Spatial shift norm ||beta^i|| */
    volatile double   tmsv_squeezing_r;    /* 0x58: Squeezing parameter r (2.50)  */
    volatile double   mu_comp_rigidity;    /* 0x60: |mu_comp - mu0| deviation     */
    volatile uint32_t dark_ledger_braids;  /* 0x68: Active braid invariant count  */
    volatile uint32_t quench_latency_ps;   /* 0x6C: Crowbar quench latency (ps)   */
    volatile uint32_t ecc_syndrome_c1;     /* 0x70: SECDED ECC syndrome line 1    */
    volatile uint32_t reserved_c1[3];      /* 0x74: Reserved pad to 128 bytes     */
} shbt_ghost_mmio_t;

_Static_assert(sizeof(shbt_ghost_mmio_t) == 128, "shbt_ghost_mmio_t must be exactly 128 bytes");
_Static_assert(offsetof(shbt_ghost_mmio_t, adm_lapse_alpha) == 64, "Cacheline 1 must align at offset 64");

#endif /* SHBT_GHOST_MMIO_H */
