# Revival Notes

These notes record the technical state of the project as it was rediscovered, plus the direction for bringing it back to life. They are intentionally practical: what the code is doing, what went wrong, what has already been fixed, and what should come next.

## Current Shape

This project is a Rust MOS 6510 / early C64 emulator experiment.

The code is currently organized as:

- `src/main.rs`: instruction implementations, address calculation, fetch/decode/execute dispatch, and the current executable loop.
- `src/proc.rs`: CPU state (`Mos6510`) and a small `ProcDelta` helper for state updates.
- `src/flags.rs`: processor status flags, addressing mode enum, opcode-to-addressing-mode lookup, and cycle/byte increment helpers.
- `src/memory.rs`: C64 RAM, BASIC/character/KERNAL ROM loading, simple memory banking, stack helpers, and direct debug printing.
- `src/test_*.rs`: focused tests for instruction behavior.

The executable loads ROMs, reads the reset vector from `0xFFFC`, and begins stepping through KERNAL startup. This is not yet a full C64 emulator. It is best understood as a partially working 6510 interpreter with the beginning of a C64 memory model.

## Recent Archaeology

The boot sequence used to fail around this trace:

```text
0xea0c 0x91 0x88 0xd1 0x0 0x0  ------  write (0x20) at: 0x01 ----- AddressMode YIndirect
0xea0e 0x0  ------  write (0xea) at: 0x1f9 ------  write (0x10) at: 0x1f8 ------  write (0x2c) at: 0x1f7 ----- AddressMode Implied
```

The ROM bytes at `0xEA0C` are:

```text
EA0C: 91 D1     STA ($D1),Y
EA0E: 88        DEY
EA0F: 10 F6     BPL ...
EA11: 60        RTS
```

The bug was in indirect-indexed addressing. The old `YIndirect` implementation treated the operand and following opcode byte as a 16-bit pointer:

```rust
memory.read_word(memory.read_word(proc.program_counter + 1)) + proc.y_index as u16
```

For `STA ($D1),Y`, that incorrectly read `D1 88` as pointer `0x88D1`. Since RAM there was zeroed, the effective address became `0x0000 + Y`, and in the observed boot trace it wrote `0x20` to address `0x0001`.

On the C64, address `0x0001` controls memory banking. Writing the wrong value there changed ROM visibility, so the next fetch at `0xEA0E` saw RAM byte `0x00` instead of the KERNAL ROM byte `0x88`. `0x00` is `BRK`, which explained the stack writes that followed.

The fix was to use the operand byte as a zero-page pointer:

```rust
memory.read_word(memory.read_byte(proc.program_counter + 1) as u16) + proc.y_index as u16
```

A regression test now covers the exact shape of the failure: operand `0xD1`, following byte `0x88`, and a zero-page pointer at `$00D1/$00D2`.

## Recent Fixes

The following fixes have been committed:

- `Fix indirect indexed addressing`
  - Corrects `AddressingMode::YIndirect` to use the operand as a zero-page pointer.
  - Adds a regression test around the `STA/LDA ($D1),Y` boot-sequence failure shape.

- `Use 6502 status flag bit layout`
  - Corrects status flag bits to real 6502/6510 layout:

    ```text
    N V - B D I Z C
    80 40 20 10 08 04 02 01
    ```

  - Keeps the existing `Flags::ALWAYS` name for bit `0x20`, the unused status bit conventionally read/pushed as set.
  - Adds a test that asserts the exact flag bit values.

In the current working tree, memory region sizing has also been corrected:

- RAM is now sized to the full 16-bit address space (`0x10000` bytes).
- BASIC and KERNAL ROM buffers are now `0x2000` bytes each.
- Character ROM is now `0x1000` bytes.
- ROM mapping offsets now use the mapped starts: `0xA000`, `0xD000`, and `0xE000`.
- Boundary tests cover first and last mapped ROM bytes plus writes to RAM address `0xFFFF`.

Zero-page wrapping has also been corrected:

- `ZeroPage,X` and `ZeroPage,Y` now wrap inside `$00..$FF`.
- `(zp,X)` wraps after adding X to the operand.
- Zero-page pointer word reads wrap the high byte from `$FF` back to `$00`.
- `(zp),Y` now uses the wrapped zero-page pointer read before adding Y.
- Addressing-level regression tests cover these wraparound cases directly.

The suite was green after these changes:

```text
cargo test
245 passed; 0 failed
```

## Known Technical Risks

These are the main correctness risks identified while reading the code.

### Indirect Addressing Details

`XIndirect` and `YIndirect` now use wrapped zero-page pointer reads, but the broader indirect-addressing behavior still deserves a careful audit against a 6502 reference.

`JMP (addr)` on the original 6502 has the famous page-boundary indirect bug. A C64-compatible emulator may need to reproduce it.

### Page-Crossing Detection

Page crossing currently compares `address >> 12`, which detects 4KB boundary crossings. 6502 cycle penalties use 256-byte pages, so this should compare high bytes instead:

```rust
(base_address & 0xFF00) != (final_address & 0xFF00)
```

Existing tests currently encode the old behavior in places, so this needs a deliberate test pass.

### Instruction Timing

Several instructions either use generic cycle helpers or have comments noting that timing still needs validation. Load/store, flag, stack, jump, and interrupt behavior should be checked against a reliable opcode table.

### Status Flag Semantics

The flag bit positions are now corrected, but some flag calculations still need accuracy review:

- overflow behavior for `ADC` and `SBC`;
- carry behavior for compare instructions;
- decimal mode, which is not meaningfully implemented yet;
- `PHP`, `PLP`, `BRK`, `RTI` status-byte behavior.

### C64 Devices

The memory map currently treats much of the I/O space as RAM or ROM and simply records writes. The KERNAL boot path will eventually need at least minimal behavior for:

- CIA registers;
- VIC-II registers;
- interrupt sources;
- keyboard-related reads;
- possibly SID registers as writable sinks.

## Recommended Direction

The next phase should make the CPU and memory foundations trustworthy before chasing full C64 behavior.

1. Fix 256-byte page-crossing detection and update affected cycle tests.
2. Audit `XIndirect`, `YIndirect`, and `Indirect` addressing against 6502 behavior.
3. Split `main.rs` into smaller modules:
   - `cpu.rs`
   - `addressing.rs`
   - `instructions.rs`
   - `machine.rs`
   - a small `main.rs` runner
4. Replace direct `print!` calls in memory and execution with a structured trace mode.
5. Add a trace format that prints registers, status, PC, opcode, effective address, and cycle count.
6. Compare traces against known-good 6502/6510 references or test ROMs.
7. Add minimal C64 device behavior only after CPU/memory behavior is less ambiguous.

## Working Style

This revival is also an agentic engineering experiment. The useful pattern so far is:

1. Read the code and infer past intent.
2. Reproduce or explain the observed failure.
3. Make one small fix.
4. Add a regression test.
5. Run the full suite.
6. Commit with a narrow message.
7. Record the reasoning while it is still fresh.

Keep doing that. The emulator is complicated enough that broad rewrites will hide bugs, but small verified changes make the boot path more meaningful each time.
