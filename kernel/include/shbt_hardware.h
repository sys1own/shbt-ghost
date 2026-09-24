/*
 * shbt_hardware.h — SHBT-MMIO-1 register map for sys1own/shbt-ghost.
 *
 * 56-byte memory-mapped register block at physical base 0x70000000,
 * plus the 2,112-byte UnifiedStinespringFrame SRAM arena partitioned
 * into a 640-byte active residual segment (eta_A = 10/33) and a
 * 1,472-byte dark ledger QEC segment (eta_D = 23/33).
 */
#ifndef SHBT_HARDWARE_H
#define SHBT_HARDWARE_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SHBT_MMIO_BASE_ADDR   0x70000000UL
#define SHBT_MMIO_BLOCK_BYTES 56U

/* Word offsets 0x00 - 0x34 */
#define SHBT_REG_SEED_MASS_HI   0x00U  /* uint64 R/W: upper 64b of 512-bit seed mass */
#define SHBT_REG_SEED_MASS_LO   0x08U  /* uint64 R/W: lower 64b of seed mass mantissa */
#define SHBT_REG_DELTA_N_BITS   0x10U  /* uint64 R/W: active bit congestion Delta N  */
#define SHBT_REG_POWER_DEBT_KW  0x18U  /* uint64 R  : entropy debt, mW units         */
#define SHBT_REG_LANR_OUTPUT_KW 0x20U  /* uint64 R  : LANR output, mW units          */
#define SHBT_REG_WAKE_COMP_MU   0x28U  /* uint64 R/W: wake compensation mu_comp      */
#define SHBT_REG_SYS_CONTROL    0x30U  /* uint32 R/W: control bitfield               */
#define SHBT_REG_SYS_STATUS     0x34U  /* uint32 R  : status bitfield                */

#define SHBT_CTRL_ENABLE_BIT     (1U << 0)
#define SHBT_CTRL_QUENCH_BIT     (1U << 1)
#define SHBT_CTRL_SUPERPOS_BIT   (1U << 2)
#define SHBT_CTRL_OPTICS_BIT     (1U << 3)

#define SHBT_STATUS_READY_BIT        (1U << 0)
#define SHBT_STATUS_LOCK_ERROR_BIT   (1U << 1)
#define SHBT_STATUS_ECC_ERROR_BIT    (1U << 2)
#define SHBT_STATUS_QUENCH_ACTIVE    (1U << 3)

/* Power guardrails (encoded in milliwatts to stay in integer registers). */
#define SHBT_MAX_POWER_DEBT_MW    906000UL   /* 906.00 kW */
#define SHBT_MIN_LANR_OUTPUT_MW   906360UL   /* 906.36 kW floor (N_min = 1633) */
#define SHBT_QUENCH_LATENCY_NS    2.18       /* GaN current-shunt crowbar */

/* SRAM arena layout */
#define SHBT_SRAM_FRAME_BYTES   2112U
#define SHBT_ACTIVE_ARENA_BYTES 640U    /* eta_A = 10/33 */
#define SHBT_DARK_LEDGER_BYTES  1472U   /* eta_D = 23/33 */
#define SHBT_FIBONACCI_BRAIDS   124U

typedef struct {
    uint8_t active_arena[SHBT_ACTIVE_ARENA_BYTES];
    uint8_t dark_ledger[SHBT_DARK_LEDGER_BYTES];
} UnifiedStinespringFrame;

#ifdef __cplusplus
}
#endif

#endif /* SHBT_HARDWARE_H */
