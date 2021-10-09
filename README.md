[![Build Status](https://github.com/larsch/grc-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/larsch/grc-rs/actions/workflows/rust.yml)

# grc-rs

## Description

Generic colouriser for the output for many programs (A port of grc + grcat to
rust). `grc` must be installed as its configuration files are used.

## Status

Colouring rules work as good as 'grc'. Replacement/skip/count not yet
implemented.

## Installation

Installation via cargo will give you the binary, but not the man page and `zsh`
shell completion script.

```sh
cargo install grc-rs
```

From AUR:

```sh
yay -S grc-rs
```

Or manually, which will also install man page and zsh completions:

```sh
cargo build --release
sudo make install
```

## Usage

Either create shell aliases for the command that you want colourised:

```sh
alias mount='grc-rs mount'
```

or use the `--aliases` option to generate a list. The brave can put this in
`~/.bashrc` or `~/.zshrc`, but things may break.

```sh
eval $(grc-rs --aliases)
```

## Configuration

Configuration files are in same format as `grc`/`grcat`. **grc-rs** supports
reading from additional configuration, `/etc/grc-rs.conf`, `~/.grc-rs`, and
`~/.config/grc-rs/grc-rs`. Colouring rules will be searched for in additional
paths `/usr/share/grc-rs`, `~/.config/grc-rs` and `~/.local/share/grc-rs`.

To extend the existing configuration for a command that is already configured,
simply add a new rule in `~/.config/grc-rs/grc-rs` and have a unique
`conf.command`. To replace existing rules for a known command, create
`~/.config/grc-rs/conf.command` and it will be used instead of the one from
`/usr/share/grc`.
