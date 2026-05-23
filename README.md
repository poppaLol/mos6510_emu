# mos6510emu

I found this code again on an old laptop after a long stretch away from it. It was one of those small time-capsule projects: enough structure to remember the ambition, enough rough edges to remember exactly where the wheels started to wobble.

Rather than restart from scratch, I am using it as an agentic engineering experiment. The goal is to see how far I can take an abandoned emulator project by pairing with a coding agent: reading the old code carefully, reconstructing past intent, fixing issues in small commits, and leaving behind a project that is easier for future-me to understand.

## What This Is

This is a Rust experiment in emulating the MOS 6510 CPU used in the Commodore 64.

The current code is mostly a CPU core plus the beginnings of a C64 memory model:

- CPU state lives in `src/proc.rs`.
- Status flags and addressing modes live in `src/flags.rs`.
- RAM, ROM loading, stack helpers, and early C64 memory banking live in `src/memory.rs`.
- Instruction implementations and the current fetch/decode/execute loop live in `src/main.rs`.
- Opcode behavior is covered by focused unit tests in `src/test_*.rs`.

There are also ROM files under `rom/`, used by the current boot experiment.

## Current State

This is not a complete C64 emulator yet. It is closer to a test-driven 6510 interpreter that can start walking through the C64 KERNAL boot sequence.

Recently revived work includes:

- fixing indirect-indexed addressing for `($nn),Y`, which was causing the boot sequence to misread `STA ($D1),Y` and fall into a bogus `BRK`;
- correcting the processor status flag bit layout to match the real 6502/6510 byte layout;
- adding regression tests around both of those fixes.

The test suite currently covers a useful subset of arithmetic, logical, load/store, branch, stack, and shift/rotate behavior.

## Running It

Run the tests:

```sh
cargo test
```

Run the current emulator loop:

```sh
cargo run
```

The runtime output is still very noisy because memory reads and writes print trace information directly. That noise is useful while rediscovering the boot path, but it should eventually become a proper trace/debug facility.

## Things Still To Do

The next layer of work is mostly about making the CPU and memory foundations trustworthy:

- Fix RAM and ROM array sizes/off-by-one boundaries.
- Correct ROM offsets and C64 memory map details.
- Add zero-page wrapping for indexed zero-page and indirect addressing modes.
- Fix page-boundary cycle detection to use 256-byte pages.
- Separate the CPU core from the binary runner.
- Replace ad hoc debug printing with a structured trace mode.
- Add enough CIA/VIC/SID-facing behavior for the KERNAL boot path to keep progressing.

## Project Spirit

The point is not just to finish an emulator. It is to treat an old, half-remembered codebase as a collaboration surface: archaeology first, then small verified changes, then gradually more ambitious engineering.

If this becomes a working C64 emulator, wonderful. If it becomes a clear record of how to revive a neglected systems project with an agent in the loop, that is also a success.
