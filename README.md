# Ruschip - A multi-variant CHIP-8 Interpreter

<p align="center">
    <img src="assets/icon.png" width="256px" heigth="256px" />
</p>

## Features

- Supports multiple variants
- Supports most&mdash;if not all&mdash;of the quirks, and they can be toggled
- Runs on native platforms and the web
- Supports customization of display colors
- Supports the loading of custom CHIP-8 fonts

## Emulator Specifications

### CHIP-8

- Runs @ ~700 instructions per second
- Stack allows at most 12 elements

### SUPER-CHIP

- Runs @ ~700 instructions per second
- Stack allows at most 12 elements
- Starts in 64×32 low-resolution mode
- Clears display on mode change
- Supports `FX75` and `FX85` persistent register storage via file-backed storage on native platforms and browser local storage on the web
- Supports program exit via `00FD`

## References

[Cowgod's Chip-8 Technical Reference v1.0](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)  
[Octo - Mastering SuperChip](http://johnearnest.github.io/Octo/docs/SuperChip.html)  
[CHIP-8 extensions and compatibility](https://chip-8.github.io/extensions/)  
