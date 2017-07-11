# Stackable windows for bspwm

![Demo](https://gfycat.com/ScentedBackCrustacean)

A lot of the time, you have a couple of terminals or other programs
that you don't want to close, but you also don't want to take up too much space.

i3 has a stacking feature to solve that problem, this project attempts to implement
the same idea in bspwm.

The project consists of two binaries, `rspwm` which keeps track of the current stacks and
`rspc` which sends messages to `rpswm` to control it.

## Installation

- Install the rust compiler and cargo.
- Clone the repo
- Run `cargo install` to install the binaries in `~/.cargo/bin/`
- Add `~/.cargo/bin/rspwm` to the startup script
- Add a keybinding for `rspc stack create` and `rspc stack remove`
- For all keybindings that change focus, add `rspc stack focus current`

