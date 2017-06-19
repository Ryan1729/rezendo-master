This is a version of [Rezendo](https://codewiz.org/~scubed/rezendo/rezendo.html#r=le4PAASH8KwFgrs4Qc8) (which is a version of [Zendo](http://www.koryheath.com/zendo/) using regular expressions) where *you* play as the master. It is currently "playable" but it often produces puzzles that the computer player will never figure out. (for example `1+|[012]*`)For those puzzles where it will work, getting the computer player to figure out the puzzle more or less comes down to understanding how it was programmed, which I (as the one who programmed it,) don't find particularly entertaining. So I'm shelving this for now. I currently consider this a failed experiment, but it was still worth trying.

## Possible Future Work
* reset button to clear computer player's memory of the current puzzle.
* instead of comparing simplified regex strings, [take a more theoretically grounded approach](https://cs.stackexchange.com/questions/12876/equivalence-of-regular-expressions)


## Installing required lib on Linux

This program relies on `libBearLibTerminal.so` so that should be copied into `usr/local/lib` or another folder indicated by this command: `ldconfig -v 2>/dev/null | grep -v ^$'\t'`

then you should run `sudo ldconfig` to complete the installation.

Then the executable should run correctly.

Alternately if your OS has a package for BearLibTerminal, that may work as well.

Once that's done compiling in debug mode with `cargo build` and release mode with `cargo build --release` should work.

## Compiling release mode for Windows

You will need a copy of the precompiled `BearLibTerminal.dll` and `BearLibTerminal.lib`.

Perform the folloing steps:

copy BearLibTerminal.lib to the project root

Comment out the line containing `crate-type = ["dylib"]` in the `Cargo.toml` in the `state_manipulation` folder. (this is more or less a workaround for [this issue](https://github.com/rust-lang/rust/issues/18807), hopefully we will eventually be able to make this switch using the `cfg` attribute, but currently using the attribute doesn't appear to work correctly.)

Run `cargo build --release` then copy the exe in `./target/release` to the desired location as well as `BearLibTerminal.dll` and any necessary assets (graphics, sound, etc.).
