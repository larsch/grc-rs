## grc-rs

A port of grc + grcat to rust. Currently relies on the 'regex' crate which
doesn't support look-ahead/look-behind, limiting the number of supported
grc/grcat rules. grc-rs will fall back gracefully and ignore unsupported
regexes.

Works well: ip, mount, free, dig, du, env, lspci, last, ss, lsof, uptime,
whois, vmstat, systemctl, lsattr, ntpdate, lsmod, tcpdump, nmap, iptables

Partially works: lsblk, uptime, ps, df

Untested: docker*, semanage*, ifconfig, ant, cvs, lolcat, log

