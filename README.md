# gg

gg will clone or fetch a repo into a standardised location (e.g.,
~/working/github.com/org/repo).
It will also cd into it and open it in an IDE for you.

## Installation

### macOS/Linux x86

1. Download and unpack the relevant asset from the
[Releases](https://github.com/marcelocantos/gg/releases) page.
2. Run gg.
3. On macOS, if macOS blocked gg from running
   1. Go to ***System Preferences > Security*** and unblock it.
   2. Run gg again and deal with the prompt.
4. Follow the instructions.

### macOS M1/M2

As above. If you really want an M1-optimised build, with 0% chance of noticing
the difference:

1. Install rust.
2. Clone this repo, cd into it and `cargo install --path .`.
3. Run gg.
4. Follow the instructions.
