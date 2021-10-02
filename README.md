# grc-rs

## Description

Generic colouriser for the output for many programs (A port of grc + grcat to
rust). `grc` must be installed as its configuration files are used.

## Status

Colouring rules work as good as 'grc'. Replacement/skip/count not yet
implemented.

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
