[![Build Status](https://travis-ci.org/Byron/dua-cli.svg?branch=master)](https://travis-ci.org/Byron/dua-cli)

**dua** (-> _Disk Usage Analyzer_) is a tool to conveniently learn about the usage of disk space of a given directory. It's parallel by default and will max out your SSD, providing relevant information as fast as possible. Optionally delete superfluous data, and do so more quickly than `rm`.

[![asciicast](https://asciinema.org/a/AaFU0fPE2E612XCjpNg9JeAgX.svg)](https://asciinema.org/a/AaFU0fPE2E612XCjpNg9JeAgX)

### Installation

### Binary Release

```sh
curl -LSfs https://japaric.github.io/trust/install.sh | \
    sh -s -- --git byron/dua-cli --crate dua
```

#### Cargo
Via `cargo`, which can be obtained using [rustup][rustup]

```
cargo install dua-cli
```

#### VoidLinux
Via `xbps` on your VoidLinux system.

```
xbps-install dua-cli
```

#### Fedora
Via `dnf` on your Fedora system.

```
sudo dnf install dua-cli
```

#### Arch Linux
Via `pacman` on your ArchLinux system.

```
sudo pacman -S dua-cli
```

### Usage

```bash
# count the space used in the current working directory
dua
# count the space used in all directories that are not hidden
dua *
# learn about additional functionality
dua aggregate --help
```

### Interactive Mode

Launch into interactive mode with the `i` or `interactive` subcommand. Get help on keyboard
shortcuts with `?`.
Use this mode to explore, and/or to delete files and directories to release disk space.

Please note that great care has been taken to prevent accidential deletions due to a multi-stage
process, which makes this mode viable for exploration.

```bash
dua i
dua interactive
```

### Roadmap

#### 🚧v2.3  - Various features and fixes as they come up while people are using it

##### Other Features

 * [ ] Evaluate unit coloring - can we highlight different units better, make them stick out?
 
 See `CHANGELOG.md` for past releases.
 
### Development

#### Run tests

```bash
make tests
```

#### Learn about other targets

```
make
```

### Acknowledgements

Thanks to [jwalk][jwalk], all there was left to do is to write a command-line interface. As `jwalk` matures, **dua** should benefit instantly.

### Limitations

* Interactive mode only looks good in dark terminals (see [this issue](https://github.com/Byron/dua-cli/issues/13))
* _easy fix_: file names in main window are not truncated if too large. They are cut off on the right.
* There are plenty of examples in `tests/fixtures` which don't render correctly in interactive mode.
  This can be due to graphemes not interpreted correctly. With Chinese characters for instance,
  column sizes are not correctly computed, leading to certain columns not being shown.
  In other cases, the terminal gets things wrong - I use alacritty, and with certain characaters it
  performs worse than, say iTerm3.
  See https://github.com/minimaxir/big-list-of-naughty-strings/blob/master/blns.txt for the source.
* One cannot abort the filesystem traversal
 * as we are in raw terminal mode, signals will not be sent to us. As as we are single-threaded in
   the GUI, we can not listen to input events while traversing the filesystem. This can be solved,
   of course, and I would love the solution to use async :).
* In interactive mode, you will need about 60MB of memory for 1 million entries in the graph.
* In interactive mode, the maximum amount of files is limited to 2^32 - 1 (`u32::max_value() - 1`) entries.
  * One node is used as to 'virtual' root
  * The actual amount of nodes stored might be lower, as there might be more edges than nodes, which are also limited by a `u32` (I guess)
  * The limitation is imposed by the underlying [`petgraph`][petgraph] crate, which declares it as `unsafe` to use u64 for instance.
  * It's possibly *UB* when that limit is reached, however, it was never observed either.
* Dedication to `termion`
  * we use [`termion`][termion] exlusively, and even though [`tui`][tui] supports multiple backends, we only support its termion backend. _Reason_: `tui` is only used for parts of the program, and in all other parts `termion` is used for coloring the output. Thus we wouldn't support changing to a different backend anyway unless everything is done with TUI, which is really not what it is made for.


[petgraph]: https://crates.io/crates/petgraph
[rustup]: https://rustup.rs/
[jwalk]: https://crates.io/crates/jwalk
[termion]: https://crates.io/crates/termion
[tui]: https://github.com/fdehau/tui-rs
