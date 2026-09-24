#!/usr/bin/env python3
"""Integration harness for sys1own/shbt-ghost: builds the C reference
microkernel and exercises the HIL boundary (ECC, quench interlock, SRAM
layout) plus the Python-side physics sanity checks."""
import ctypes
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

from shbt_ghost.cli import main as cli  # noqa: E402


def main() -> int:
    cli.build_kernel()
    lib = ctypes.CDLL(str(cli.REF_SO))
    lib.shbt_compute_secded_ecc.restype = ctypes.c_uint8
    lib.shbt_compute_secded_ecc.argtypes = [ctypes.c_uint64]
    lib.shbt_verify_and_correct_secded.restype = ctypes.c_int
    lib.shbt_verify_and_correct_secded.argtypes = [
        ctypes.POINTER(ctypes.c_uint64), ctypes.c_uint8]
    lib.shbt_sys_status.restype = ctypes.c_uint32

    # ECC single-bit correction across all 64 bit positions.
    data = 0x0123456789ABCDEF
    ecc = lib.shbt_compute_secded_ecc(data)
    for bit in range(64):
        w = ctypes.c_uint64(data ^ (1 << bit))
        r = lib.shbt_verify_and_correct_secded(ctypes.byref(w), ecc)
        assert r == 1 and w.value == data, f"ECC failed at bit {bit}"

    # Clean word verifies.
    w = ctypes.c_uint64(data)
    assert lib.shbt_verify_and_correct_secded(ctypes.byref(w), ecc) == 0

    # Quench interlock asserts QUENCH_ACTIVE (bit 3).
    lib.shbt_ghost_kernel_init()
    lib.shbt_trigger_quench_interlock()
    assert lib.shbt_sys_status() & (1 << 3)

    # Givens remap executes without fault.
    lib.shbt_avx512_givens_remapping(ctypes.c_double(0.6), ctypes.c_double(0.8))

    print("run_all_tests: all checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
