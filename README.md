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
