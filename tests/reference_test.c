/*
 * tests/reference_test.c — C ABI regression for the shbt-ghost kernel:
 * SECDED Hamming(72,64) ECC cycle, legacy register block, and the
 * 128-byte dual-cacheline shbt_ghost_mmio_t battery/metric contract.
 */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "../kernel/include/shbt_hardware.h"
#include "../kernel/include/shbt_ghost_mmio.h"

extern uint8_t shbt_compute_secded_ecc(uint64_t data);
extern int shbt_verify_and_correct_secded(uint64_t *data, uint8_t stored_ecc);
extern void shbt_ghost_kernel_init(void);
extern void shbt_ghost_kernel_step(void);
extern void shbt_trigger_quench_interlock(void);
extern uint32_t shbt_sys_status(void);
extern void shbt_avx512_givens_remapping(double c, double s);
extern void shbt_ghost_mmio_init(void);
extern void shbt_ghost_mmio_set_state(uint32_t state);
extern void shbt_ghost_mmio_burst_telemetry(uint64_t target_bits,
    uint64_t actual_bits, double dec_v, double gross_w,
    float core_k, float plate_k, uint16_t soc);
extern uint32_t shbt_ghost_mmio_state(void);
extern uint32_t shbt_ghost_mmio_quench_latency_ps(void);
extern const volatile shbt_ghost_mmio_t *shbt_ghost_mmio_block(void);

int main(void)
{
    /* SECDED single-bit correction over all 64 positions. */
    uint64_t data = 0x0123456789ABCDEFULL;
    uint8_t ecc = shbt_compute_secded_ecc(data);
    for (int bit = 0; bit < 64; ++bit) {
        uint64_t w = data ^ (1ULL << bit);
        assert(shbt_verify_and_correct_secded(&w, ecc) == 1);
        assert(w == data);
    }
    uint64_t clean = data;
    assert(shbt_verify_and_correct_secded(&clean, ecc) == 0);

    /* Legacy register block lifecycle + quench interlock. */
    shbt_ghost_kernel_init();
    assert(shbt_sys_status() & SHBT_STATUS_READY_BIT);
    shbt_ghost_kernel_step();
    shbt_trigger_quench_interlock();
    assert(shbt_sys_status() & SHBT_STATUS_QUENCH_ACTIVE);
    shbt_avx512_givens_remapping(0.6, 0.8);

    /* 128-byte dual-cacheline contract: size, alignment, state machine. */
    _Static_assert(sizeof(shbt_ghost_mmio_t) == 128, "MMIO contract must be 128 B");
    _Static_assert(offsetof(shbt_ghost_mmio_t, adm_lapse_alpha) == 64,
                   "cacheline 1 must start at offset 64");

    shbt_ghost_mmio_init();
    assert(shbt_ghost_mmio_state() == SHBT_STATE_STANDBY_STASIS);
    const volatile shbt_ghost_mmio_t *m = shbt_ghost_mmio_block();
    assert(m->adm_lapse_alpha == 1.0);
    assert(m->tmsv_squeezing_r == 2.50);
    assert(m->dark_ledger_braids == 124);

    shbt_ghost_mmio_set_state(SHBT_STATE_TRIGGER_ARMED);
    assert(shbt_ghost_mmio_state() == SHBT_STATE_TRIGGER_ARMED);
    shbt_ghost_mmio_set_state(SHBT_STATE_BURST_TRACTION);
    assert(shbt_ghost_mmio_state() == SHBT_STATE_BURST_TRACTION);

    /* Peak-burst telemetry: 2.7114e13 bits/step, 400 kV bus, 109.05 TW. */
    shbt_ghost_mmio_burst_telemetry(27114495114006ULL, 27114495114006ULL,
                                    400000.0, 109.05e12, 21.5f, 21.2f, 880);
    assert(m->current_bits_step == 27114495114006ULL);
    assert(m->dec_bus_voltage_v == 400000.0);
    assert(m->battery_soc_permille == 880);

    shbt_ghost_mmio_set_state(SHBT_STATE_EMERGENCY_QUENCH);
    assert(shbt_ghost_mmio_state() == SHBT_STATE_EMERGENCY_QUENCH);
    assert(shbt_ghost_mmio_quench_latency_ps() == 2180);
    assert(m->dec_bus_voltage_v == 0.0);

    printf("reference_test: all C ABI checks passed\n");
    return 0;
}
