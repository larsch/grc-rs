# grc-rs

## Description

Generic colouriser for the output for many programs (A port of grc + grcat to
rust). `grc` must be installed as its configuration files are used.

Currently relies on the 'regex' crate which doesn't support
look-ahead/look-behind, limiting the number of supported grc/grcat rules. grc-rs
will fall back gracefully and ignore unsupported regexes.

## Status

Works well: ip, mount, free, dig, du, env, lspci, last, ss, lsof, uptime,
whois, vmstat, systemctl, lsattr, ntpdate, lsmod, tcpdump, nmap, iptables

Partially works: lsblk, uptime, ps, df

Untested: docker*, semanage*, ifconfig, ant, cvs, lolcat, log

## Usage

Either create shell aliases for the command that you want colourised:

```sh
alias mount='grc-rs mount'
```

or use the `--aliases` option to generate a list. The brave can put this in `~/.bashrc` or `~/.zshrc`, but things may break.

```sh
eval $(grc-rs --aliases)
```
