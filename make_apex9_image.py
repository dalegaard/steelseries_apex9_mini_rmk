#!/usr/bin/env python3
"""
Build a combined STM32L412 flash image:

    0x08000000 .. 0x08008FFF  bootstrap + 0xFF padding
    0x08009000 ..             SteelSeries application image

Requires:
    arm-none-eabi-as
    arm-none-eabi-ld
    arm-none-eabi-objcopy

Example:
    python3 make_apex9_image.py apex9_firmware.bin apex9_combined.bin
"""

from __future__ import annotations

import argparse
import shutil
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

FLASH_BASE = 0x08000000
APP_BASE = 0x08009000
BOOTSTRAP_SIZE = APP_BASE - FLASH_BASE
FLASH_SIZE = 128 * 1024
FLASH_END = FLASH_BASE + FLASH_SIZE

SRAM_BASE = 0x20000000
SRAM_END = 0x2000A000

ASSEMBLY = r"""
.syntax unified
.cpu cortex-m4
.thumb

.equ APP_BASE, 0x08009000
.equ VTOR,     0xE000ED08

.section .isr_vector, "a", %progbits
.align 9
.global _vectors
_vectors:
    .word _stack_start
    .word bootstrap + 1
    .word loop + 1
    .word loop + 1
    .word loop + 1
    .word loop + 1
    .word loop + 1
    .word 0
    .word 0
    .word loop + 1
    .word loop + 1
    .word loop + 1
    .word loop + 1
    .word loop + 1
    .word loop + 1
    .word loop + 1

.section .text.bootstrap, "ax", %progbits
.align 2

.thumb_func
.global bootstrap
bootstrap:
    cpsid   i

    ldr     r1, =APP_BASE
    ldr     r0, =VTOR
    str     r1, [r0]
    dsb
    isb

    ldr     r0, [r1, #0]
    ldr     r2, [r1, #4]

    msr     msp, r0
    msr     psp, r0
    movs    r0, #0
    msr     control, r0
    isb
    bx      r2

.thumb_func
.global loop
loop:
    b       loop
"""

LINKER_SCRIPT = r"""
ENTRY(bootstrap)

MEMORY
{
    FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 0x9000
}

_stack_start = 0x2000A000;

SECTIONS
{
    .isr_vector ORIGIN(FLASH) :
    {
        KEEP(*(.isr_vector))
    } > FLASH

    .text :
    {
        *(.text.bootstrap)
        *(.text*)
        *(.rodata*)
    } > FLASH

    /DISCARD/ :
    {
        *(.comment)
        *(.ARM.attributes)
    }
}
"""


def require_tool(name: str) -> str:
    path = shutil.which(name)
    if path is None:
        raise RuntimeError(
            f"Required tool not found: {name}. Install the GNU Arm Embedded "
            "toolchain and ensure it is in PATH."
        )
    return path


def run(command: list[str]) -> None:
    subprocess.run(command, check=True)


def validate_application(image: bytes) -> tuple[int, int]:
    if len(image) < 8:
        raise ValueError("Application image is too small to contain a vector table.")

    maximum = FLASH_END - APP_BASE
    if len(image) > maximum:
        raise ValueError(
            f"Application is 0x{len(image):X} bytes, but only 0x{maximum:X} "
            f"bytes fit at 0x{APP_BASE:08X}."
        )

    initial_sp, reset_handler = struct.unpack_from("<II", image, 0)

    if not (SRAM_BASE < initial_sp <= SRAM_END):
        raise ValueError(
            f"Initial SP 0x{initial_sp:08X} is outside expected SRAM "
            f"0x{SRAM_BASE:08X}..0x{SRAM_END:08X}."
        )

    reset_address = reset_handler & ~1
    if not (APP_BASE <= reset_address < FLASH_END):
        raise ValueError(
            f"Reset handler 0x{reset_handler:08X} is outside application flash."
        )

    if not (reset_handler & 1):
        raise ValueError("Reset handler does not have the Thumb bit set.")

    return initial_sp, reset_handler


def build_bootstrap(workdir: Path) -> bytes:
    assembler = require_tool("arm-none-eabi-as")
    linker = require_tool("arm-none-eabi-ld")
    objcopy = require_tool("arm-none-eabi-objcopy")

    asm_path = workdir / "bootstrap.S"
    ld_path = workdir / "bootstrap.ld"
    obj_path = workdir / "bootstrap.o"
    elf_path = workdir / "bootstrap.elf"
    bin_path = workdir / "bootstrap.bin"

    asm_path.write_text(ASSEMBLY)
    ld_path.write_text(LINKER_SCRIPT)

    run([assembler, "-mcpu=cortex-m4", "-mthumb", "-o", str(obj_path), str(asm_path)])
    run([linker, "-T", str(ld_path), "--gc-sections", "-o", str(elf_path), str(obj_path)])
    run([objcopy, "-O", "binary", str(elf_path), str(bin_path)])

    bootstrap = bin_path.read_bytes()
    if len(bootstrap) > BOOTSTRAP_SIZE:
        raise RuntimeError("Bootstrap exceeds the reserved 0x9000-byte region.")

    return bootstrap


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Create an STM32L412 bootstrap + Apex 9 application image."
    )
    parser.add_argument("firmware", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument(
        "--full-flash",
        action="store_true",
        help="Pad output to the full 128 KiB internal flash size.",
    )
    args = parser.parse_args()

    try:
        application = args.firmware.read_bytes()
        initial_sp, reset_handler = validate_application(application)

        with tempfile.TemporaryDirectory(prefix="apex9-bootstrap-") as temp:
            bootstrap = build_bootstrap(Path(temp))

        combined = bytearray(b"\xFF" * BOOTSTRAP_SIZE)
        combined[:len(bootstrap)] = bootstrap
        combined.extend(application)

        if args.full_flash:
            combined.extend(b"\xFF" * (FLASH_SIZE - len(combined)))

        args.output.write_bytes(combined)

        print(f"Application SP: 0x{initial_sp:08X}")
        print(f"Reset handler:  0x{reset_handler:08X}")
        print(f"Bootstrap size: 0x{len(bootstrap):X}")
        print(f"Application at: 0x{APP_BASE:08X}")
        print(f"Output size:    0x{len(combined):X}")
        print(f"Wrote: {args.output}")
        return 0

    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
