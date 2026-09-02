# Backend

`backend` is Ruschip's CHIP-8 interpreter. It owns emulated CPU state and
memory, while receiving display and keypad state through the interfaces in
this directory. The frontend is responsible for scheduling `tick`, updating
the keypad, presenting the display, and playing audio when `Backend::sound()`
is non-zero.

## Public boundary

The crate-level [`crate::backend::Backend`] enum is the integration point. It
currently contains only `Backend::Chip8`, but keeps the caller independent of
the concrete interpreter.

Typical lifecycle:

1. Create `Backend::default()` and use `default_display_buffer()` to obtain a
   compatible 64 × 32 display buffer.
2. Call `load(font, program)`. `font` may be omitted to use
   `defaults::BACKEND_FONT`.
3. Update `KeypadState` from the UI, then call `tick(n, display, keypad)` to
   execute at most `n` instructions.
4. Read `sound()` to determine whether the sound timer is active. Reset with
   `reset()` before reusing the loaded program.

`tick` returns a `BackendError` for invalid program state, memory accesses,
stack operations, instructions, keys, or font sprites. Errors carry the
instruction address and decoded instruction whenever one is available.

## CHIP-8 model

The CHIP-8 implementation uses 4 KiB of memory, 16 general-purpose `V`
registers (with `VF` used as the flag register), an `I` address register, a
program counter, and a call stack. Programs load at `0x200`; the built-in or
provided 80-byte low-resolution font occupies the beginning of memory.

The interpreter implements the standard instruction families used in
`chip8.rs`: control flow, conditional skips, register arithmetic and logic,
memory operations, random masking, keypad skips, timers, BCD conversion,
and font addressing. `0NNN` is ignored because it requires the original
RCA 1802/M6800 environment. Drawing (`DXYN`) ends the current `tick` early;
so does `FX0A`, which waits for a key-release event.

Calls are bounded to 16 entries even though the backing stack is a `Vec`.
`2NNN` reports `StackOverflow` at that limit and `00EE` reports
`StackUnderflow` when there is no return address.

## Compatibility options

`BackendOptions`, obtained through `options_mut()`, controls the supported
CHIP-8 dialect differences:

| Field | When enabled |
| --- | --- |
| `copy_and_shift` | `8XY6` and `8XYE` copy `VY` into `VX` before shifting. |
| `increment_address` | `FX55` and `FX65` advance `I` past the transferred registers. |
| `quirky_jump` | `BNNN` uses `VX + NNN`; otherwise it uses `V0 + NNN`. |
| `reset_flag` | `8XY1`, `8XY2`, and `8XY3` clear `VF`. |

The default interpreter enables `copy_and_shift`, `increment_address`, and
`reset_flag`; `quirky_jump` is disabled.

## Display interface

`interfaces::display_buffer::DisplayBuffer` stores one-bit pixels in rows and
tracks whether its contents changed. `flattened()` returns pixels in row-major
order and clears the dirty flag, allowing the frontend to upload only changed
frames.

Sprites are XOR-drawn. The method returns the number of rows containing a
pixel collision; the interpreter writes `VF = 1` when that count is non-zero.
Eight-bit sprites are normal CHIP-8 sprites; a 32-byte sprite is interpreted
as sixteen big-endian 16-bit rows. `DisplayOptions::clip_sprites` selects
clipping at the right/bottom edge instead of wrapping. The buffer also exposes
scroll operations and resolution-halving support for higher-level variants,
although the current CHIP-8 backend creates a 64 × 32 buffer with clipping
enabled and half-pixel scrolling disabled.

`KeypadState` represents CHIP-8 keys `0x0..=0xF`. Its `update()` method maps
egui input through `defaults::KEY_MAP`. `pressed_key()` deliberately detects a
held-to-released transition; this is the event consumed by `FX0A`.

## Timers and execution rate

Delay and sound timers are wall-clock timers. Their remaining value is
computed from `Instant::elapsed()` at a nominal 60 Hz (`1000 / 60`
milliseconds per decrement), rather than being decremented by `tick`.
Consequently, CPU scheduling and timer progression are independent, but timer
accuracy is limited by millisecond rounding and when callers poll them.

`Frontend::tick()` uses an instruction budget of `28`; at 60 calls per second,
that is 1,680 instructions per second. The backend itself does not impose an
instruction rate or schedule calls to `tick`.

## Implementation notes

- `Instruction` is a two-byte, big-endian decoder with helpers for CHIP-8
  opcode fields.
- `load()` validates program length only. A custom font must contain at least
  `FONT_SIZE` bytes because that many bytes are copied into memory.
- `reset()` clears registers, stack, and timers and restores the program
  counter, but retains memory and the `loaded` flag. Load a new program to
  replace program memory.
- Display scrolling assumes the requested distance does not exceed the display
  dimension; callers should maintain that invariant.
