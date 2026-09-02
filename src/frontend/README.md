# Frontend

`frontend` is the presentation adapter between the CHIP-8 backend and the
native eframe/egui application. It owns the backend's display buffer and
keypad state, converts the one-bit display into an egui texture, and gates a
continuous audio source from the CHIP-8 sound timer.

The module does not own application state such as menus, program selection, or
the emulation run/pause state; those are handled by `ui`.

## Constructing a frontend

`Frontend::new(ctx, backend, display_buffer)` requires:

- an egui context for creating the display texture;
- a configured `backend::Backend`; and
- the corresponding `DisplayBuffer`, normally produced by
  `backend.default_display_buffer()`.

Construction opens the default audio sink, creates a `RodioAudio` instance,
and uploads the initial display pixels using `defaults::COLORS`. It can fail
with `FrontendError::Audio` when the default output sink cannot be opened.

`Frontend` exposes its `backend`, `display_buffer`, `keypad_state`, and
`colors` for the application layer. `display_texture()` returns the texture ID
used to render the emulator display.

## Per-frame integration

The caller is responsible for updating the keypad before emulating a frame:

```rust
frontend.keypad_state.update(&input);
frontend.tick()?;
```

`tick()` follows this sequence:

1. It enables or disables audio based on the backend's current sound timer.
2. It executes up to 28 CHIP-8 instructions through `Backend::tick`.
3. If the display buffer is dirty, it rebuilds the pixel image and replaces the
   egui texture with nearest-neighbor sampling.

The CPU budget is fixed at 28 instructions per frontend tick. A caller running
the frontend at 60 FPS therefore targets 1,680 instructions per second, but
the frontend itself does not schedule or request frames.

`reset()` resets the backend only; it does not recreate the texture, clear the
display buffer, or reload a program. `suspend()` immediately disables audio.

## Display conversion

The backend display is a row-major sequence of booleans. `update_texture()`
maps each bit through `Colors`:

| Pixel value | Color |
| --- | --- |
| `true` | `Colors::active` |
| `false` | `Colors::inactive` |

The display buffer's `flattened()` iterator clears its dirty flag as it is
consumed. Consequently, callers that manually change `colors` should call
`update_texture()` to repaint an otherwise unchanged frame.

## Audio

`audio::RodioAudio` opens rodio's default output sink and appends one infinite
mono source. The source produces a 440 Hz square wave with an amplitude of
0.1 while enabled and silence while disabled. An atomic boolean is shared with
the audio callback, so `set_enabled()` changes the gate without replacing or
reconnecting the source; repeated requests for the existing state are skipped.

`Frontend::tick()` observes the sound timer before executing the instruction
batch, so an instruction that changes the timer takes effect on the next
frontend tick. `suspend()` only disables the audio gate and does not pause
emulation.

## Errors

`FrontendError` wraps audio setup failures and `BackendError`s. Its
`is_fatal()` classification treats audio errors and these backend errors as
fatal:

- `MemoryOverflow`
- `ProgramInvalid`
- `ProgramNotLoaded`

Other backend errors, such as an unrecognized instruction or a stack error,
remain recoverable according to this classification; the application decides
how to display or handle them.
