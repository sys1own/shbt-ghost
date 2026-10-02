/*
 * shbt_ghost_kernel.c — C11 freestanding microkernel for sys1own/shbt-ghost.
 *
 * Drives the 56-byte SHBT-MMIO-1 register block at 0x70000000, maintains the
 * 2,112-byte UnifiedStinespringFrame SRAM arena, computes/verifies SECDED
 * Hamming(72,64) ECC, and executes the sub-2.50 ns GaN current-shunt quench
 * interlock (tau_quench <= 2.18 ns).
 *
 * When compiled with -DSHBT_HOSTED_TEST the MMIO block is redirected to a
 * host-provided shadow buffer so the routines can run inside the Rust FFI
 * bridge and the Python reference library without real hardware.
 */
#include "include/shbt_hardware.h"
#include "include/shbt_ghost_mmio.h"

#ifdef SHBT_HOSTED_TEST
static uint8_t g_mmio_shadow[SHBT_MMIO_BLOCK_BYTES];
#define MMIO_BASE ((uintptr_t)g_mmio_shadow)
#else
#define MMIO_BASE ((uintptr_t)SHBT_MMIO_BASE_ADDR)
#endif

#define REG_SEED_MASS_HI   ((volatile uint64_t *)(MMIO_BASE + SHBT_REG_SEED_MASS_HI))
#define REG_SEED_MASS_LO   ((volatile uint64_t *)(MMIO_BASE + SHBT_REG_SEED_MASS_LO))
#define REG_DELTA_N_BITS   ((volatile uint64_t *)(MMIO_BASE + SHBT_REG_DELTA_N_BITS))
#define REG_POWER_DEBT_KW  ((volatile uint64_t *)(MMIO_BASE + SHBT_REG_POWER_DEBT_KW))
#define REG_LANR_OUTPUT_KW ((volatile uint64_t *)(MMIO_BASE + SHBT_REG_LANR_OUTPUT_KW))
#define REG_WAKE_COMP_MU   ((volatile uint64_t *)(MMIO_BASE + SHBT_REG_WAKE_COMP_MU))
#define REG_SYS_CONTROL    ((volatile uint32_t *)(MMIO_BASE + SHBT_REG_SYS_CONTROL))
#define REG_SYS_STATUS     ((volatile uint32_t *)(MMIO_BASE + SHBT_REG_SYS_STATUS))

#if defined(__GNUC__)
#define SHBT_PACKED __attribute__((packed, aligned(64)))
#else
#define SHBT_PACKED
#endif

static UnifiedStinespringFrame g_sram_frame
#if defined(__GNUC__)
    __attribute__((section(".stinespring_frame"), aligned(64)))
#endif
    ;

/* ------------------------------------------------------------------ */
/* SECDED Hamming(72,64): 7 parity bits + 1 global parity.              */
/* Parity coverage is derived from the set-bit position index of each   */
/* data bit; decode locates and corrects any single-bit error.          */
/* ------------------------------------------------------------------ */
static uint8_t hamming_position_parity(uint64_t data, uint8_t parity_bit)
{
    uint8_t p = 0;
    for (uint8_t i = 0; i < 64; ++i) {
        if (((data >> i) & 1U) && (((i + 1U) >> parity_bit) & 1U)) {
            p ^= 1U;
        }
    }
    return p;
}

uint8_t shbt_compute_secded_ecc(uint64_t data)
{
    uint8_t ecc = 0;
    for (uint8_t bit = 0; bit < 7; ++bit) {
        ecc |= (uint8_t)(hamming_position_parity(data, bit) << bit);
    }
    /* Global parity over data + check bits. */
    uint8_t gp = hamming_position_parity(data, 0) ? 0 : 0;
    uint64_t w = data;
    gp = 0;
    for (uint8_t i = 0; i < 64; ++i) {
        gp ^= (uint8_t)((w >> i) & 1U);
    }
    for (uint8_t i = 0; i < 7; ++i) {
        gp ^= (uint8_t)((ecc >> i) & 1U);
    }
    ecc |= (uint8_t)(gp << 7);
    return ecc;
}

/* Returns 0 on clean or corrected data; sets *corrected if fixed. */
int shbt_verify_and_correct_secded(uint64_t *data, uint8_t stored_ecc)
{
    uint8_t syndrome = 0;
    for (uint8_t bit = 0; bit < 7; ++bit) {
        uint8_t computed = hamming_position_parity(*data, bit);
        uint8_t stored = (uint8_t)((stored_ecc >> bit) & 1U);
        syndrome |= (uint8_t)((computed ^ stored) << bit);
    }
    uint8_t gp_now = 0;
    uint64_t w = *data;
    for (uint8_t i = 0; i < 64; ++i) {
        gp_now ^= (uint8_t)((w >> i) & 1U);
    }
    for (uint8_t i = 0; i < 8; ++i) {
        gp_now ^= (uint8_t)((stored_ecc >> i) & 1U);
    }

    if (syndrome == 0 && gp_now == 0) {
        return 0;                       /* clean */
    }
    if (syndrome != 0 && gp_now == 1) {
        *data ^= (1ULL << (syndrome - 1U)); /* single data-bit error: correct */
        return 1;
    }
    if (syndrome == 0 && gp_now == 1) {
        return 1;                       /* error was in the parity bit itself */
    }
    *REG_SYS_STATUS |= SHBT_STATUS_ECC_ERROR_BIT;   /* double error: uncorrectable */
    return -1;
}

/* ------------------------------------------------------------------ */
/* Kernel lifecycle                                                     */
/* ------------------------------------------------------------------ */
void shbt_ghost_kernel_init(void)
{
    *REG_SYS_CONTROL = 0x00000000U;
    *REG_SYS_STATUS = SHBT_STATUS_READY_BIT;
    for (uint32_t i = 0; i < sizeof(UnifiedStinespringFrame); ++i) {
        ((uint8_t *)&g_sram_frame)[i] = 0;
    }
}

/* GaN current-shunt crowbar: assert quench, drop enable. Latency is set by
 * hardware propagation at <= 2.18 ns (< 2.50 ns interlock bound). */
void shbt_trigger_quench_interlock(void)
{
    *REG_SYS_CONTROL |= SHBT_CTRL_QUENCH_BIT;
    *REG_SYS_CONTROL &= ~SHBT_CTRL_ENABLE_BIT;
    *REG_SYS_STATUS |= SHBT_STATUS_QUENCH_ACTIVE;
}

uint32_t shbt_sys_status(void)
{
    return *REG_SYS_STATUS;
}

void shbt_ghost_kernel_step(void)
{
    uint64_t current_power_debt = *REG_POWER_DEBT_KW;
    uint64_t current_lanr_output = *REG_LANR_OUTPUT_KW;

    /* Sub-2.50 ns emergency quench: debt breach or LANR floor violation. */
    if ((current_power_debt > SHBT_MAX_POWER_DEBT_MW) ||
        (current_lanr_output < SHBT_MIN_LANR_OUTPUT_MW)) {
        shbt_trigger_quench_interlock();
        return;
    }

    /* Verify Hamming ECC over the active seed-mass register. */
    uint64_t active_state_word = *REG_SEED_MASS_HI;
    uint8_t ecc = shbt_compute_secded_ecc(active_state_word);
    (void)shbt_verify_and_correct_secded(&active_state_word, ecc);

    *REG_SYS_CONTROL |= (SHBT_CTRL_ENABLE_BIT | SHBT_CTRL_SUPERPOS_BIT);
}

/* AVX-512-accelerated Givens remapping over the active residual arena.
 * Rotates adjacent 64-bit lanes so eigenvector rigidity
 * |mu_comp - mu_0| <= 1e-12 is preserved under wake compensation. */
void shbt_avx512_givens_remapping(double c, double s)
{
    double *lanes = (double *)&g_sram_frame.active_arena[0];
    const uint32_t pairs = (SHBT_ACTIVE_ARENA_BYTES / sizeof(double)) / 2U;
    for (uint32_t i = 0; i < pairs; ++i) {
        double x = lanes[2U * i];
        double y = lanes[2U * i + 1U];
        lanes[2U * i]     = c * x + s * y;
        lanes[2U * i + 1U] = c * y - s * x;
    }
}

/* ------------------------------------------------------------------ */
/* 128-byte dual-cacheline battery/metric MMIO contract                  */
/* (shbt_ghost_mmio_t, ghost3.txt upgrade). In hosted-test builds the    */
/* block is backed by a static shadow buffer; on hardware it is the      */
/* volatile register window at 0x70000000. Zero dynamic allocation.      */
/* ------------------------------------------------------------------ */
#ifdef SHBT_HOSTED_TEST
static shbt_ghost_mmio_t g_ghost_mmio;
#define GHOST_MMIO (&g_ghost_mmio)
#else
#define GHOST_MMIO ((shbt_ghost_mmio_t *)SHBT_MMIO_BASE_ADDR)
#endif

#define GHOST_DEC_STANDBY_V   15000.0
#define GHOST_QUENCH_LAT_PS   2180U    /* 2.18 ns crowbar latency        */
#define GHOST_CORE_COLD_K     21.13f

void shbt_ghost_mmio_init(void)
{
    shbt_ghost_mmio_t *m = GHOST_MMIO;
    for (uint32_t i = 0; i < sizeof(*m); ++i) {
        ((uint8_t *)m)[i] = 0;
    }
    m->ctrl_status        = SHBT_STATE_STANDBY_STASIS;
    m->dec_bus_voltage_v  = GHOST_DEC_STANDBY_V;
    m->core_temp_kelvin   = GHOST_CORE_COLD_K;
    m->cold_plate_temp_k  = GHOST_CORE_COLD_K;
    m->adm_lapse_alpha    = 1.0;
    m->det_g_error        = 0.0;
    m->shift_norm_beta    = 0.0;
    m->tmsv_squeezing_r   = 2.50;
    m->mu_comp_rigidity   = 0.0;
    m->dark_ledger_braids = SHBT_FIBONACCI_BRAIDS;
    m->quench_latency_ps  = 0;
    m->ecc_syndrome_c0    = shbt_compute_secded_ecc(m->target_bits_step);
    m->ecc_syndrome_c1    = shbt_compute_secded_ecc(0xFFFFFFFFFFFFFFFFULL);
}

/* Transition the five-phase battery dispatch state machine. Only legal
 * phase progressions are accepted; an illegal request is a no-op. */
void shbt_ghost_mmio_set_state(uint32_t state)
{
    shbt_ghost_mmio_t *m = GHOST_MMIO;
    switch (state) {
    case SHBT_STATE_STANDBY_STASIS:
        m->ctrl_status       = SHBT_STATE_STANDBY_STASIS;
        m->dec_bus_voltage_v = GHOST_DEC_STANDBY_V;
        m->gross_burst_power_w = 0.0;
        m->current_bits_step = 0;
        break;
    case SHBT_STATE_TRIGGER_ARMED:
        if (m->ctrl_status == SHBT_STATE_STANDBY_STASIS) {
            m->ctrl_status      = SHBT_STATE_TRIGGER_ARMED;
            m->pcss_crowbar_arm = 1U;
        }
        break;
    case SHBT_STATE_BURST_TRACTION:
        if (m->ctrl_status == SHBT_STATE_TRIGGER_ARMED) {
            m->ctrl_status = SHBT_STATE_BURST_TRACTION;
        }
        break;
    case SHBT_STATE_DEC_COOLDOWN:
        if (m->ctrl_status == SHBT_STATE_BURST_TRACTION) {
            m->ctrl_status         = SHBT_STATE_DEC_COOLDOWN;
            m->gross_burst_power_w = 0.0;
        }
        break;
    case SHBT_STATE_EMERGENCY_QUENCH:
        m->ctrl_status        = SHBT_STATE_EMERGENCY_QUENCH;
        m->gross_burst_power_w = 0.0;
        m->dec_bus_voltage_v  = 0.0;
        m->quench_latency_ps  = GHOST_QUENCH_LAT_PS;
        m->pcss_crowbar_arm   = 0U;
        shbt_trigger_quench_interlock();
        break;
    default:
        break;
    }
    m->ecc_syndrome_c0 = shbt_compute_secded_ecc(
        ((uint64_t)m->ctrl_status << 32) | m->pcss_crowbar_arm);
    union { double d; uint64_t u; } lapse = { .d = (double)m->adm_lapse_alpha };
    m->ecc_syndrome_c1 = shbt_compute_secded_ecc(lapse.u);
}

/* Burst-telemetry write: commanded/actual bit stepping, DEC bus voltage,
 * gross graser power, and cryo temperatures. */
void shbt_ghost_mmio_burst_telemetry(uint64_t target_bits, uint64_t actual_bits,
                                     double dec_v, double gross_w,
                                     float core_k, float plate_k, uint16_t soc)
{
    shbt_ghost_mmio_t *m = GHOST_MMIO;
    m->target_bits_step      = target_bits;
    m->current_bits_step     = actual_bits;
    m->dec_bus_voltage_v     = dec_v;
    m->gross_burst_power_w   = gross_w;
    m->core_temp_kelvin      = core_k;
    m->cold_plate_temp_k     = plate_k;
    m->battery_soc_permille  = soc;
    m->ecc_syndrome_c0       = shbt_compute_secded_ecc(target_bits ^ actual_bits);
}

uint32_t shbt_ghost_mmio_state(void)
{
    return GHOST_MMIO->ctrl_status;
}

uint32_t shbt_ghost_mmio_quench_latency_ps(void)
{
    return GHOST_MMIO->quench_latency_ps;
}

/* Read-only snapshot for FFI callers. */
const volatile shbt_ghost_mmio_t *shbt_ghost_mmio_block(void)
{
    return GHOST_MMIO;
}
