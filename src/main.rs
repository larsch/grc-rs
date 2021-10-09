mod colourise;
mod grc;

use std::fs::File;
use std::io::BufRead;
use std::process::{Command, Stdio};
use std::str::FromStr;

use colourise::colourise;
use grc::{GrcConfigReader, GrcatConfigEntry, GrcatConfigReader};

enum ColourMode {
    On,
    Off,
    Auto,
}

impl FromStr for ColourMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(ColourMode::On),
            "off" => Ok(ColourMode::Off),
            "auto" => Ok(ColourMode::Auto),
            _ => Err(()),
        }
    }
}

// Main
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut command: Vec<String> = Vec::new();
    let mut colour = ColourMode::Auto;
    let mut show_all_aliases = false;
    let mut except_aliases: Vec<String> = Vec::new();
    let mut show_aliases = false;
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Generic colouriser");
        ap.stop_on_first_argument(true);
        ap.refer(&mut colour).add_option(
            &["--colour"],
            argparse::Store,
            "Override color output (on, off, auto)",
        );
        ap.refer(&mut command).required().add_argument(
            "command",
            argparse::Collect,
            "Command to run",
        );
        ap.refer(&mut show_aliases).add_option(
            &["--aliases"],
            argparse::StoreTrue,
            "Output shell aliases for available binaries",
        );
        ap.refer(&mut show_all_aliases).add_option(
            &["--all-aliases"],
            argparse::StoreTrue,
            "Output all shell aliases",
        );
        ap.refer(&mut except_aliases).add_option(
            &["--except"],
            argparse::Collect,
            "Exclude alias from generated list (multiple or comma-separated allowed)",
        );
        ap.parse_args_or_exit();
    }

    if show_aliases || show_all_aliases {
        let grc = std::env::current_exe().unwrap();
        let grc = grc.display();

        // Curated list of command that work well
        for cmd in &[
            "ant",
            "blkid",
            "common",
            "curl",
            "cvs",
            "df",
            "diff",
            "dig",
            "dnf",
            "docker",
            "du",
            "dummy",
            "env",
            "esperanto",
            "fdisk",
            "findmnt",
            "free",
            "gcc",
            "getfacl",
            "getsebool",
            "id",
            "ifconfig",
            "ip",
            "iptables",
            "irclog",
            "iwconfig",
            "jobs",
            "kubectl",
            "last",
            "ldap",
            "log",
            "lolcat",
            "lsattr",
            "lsblk",
            "lsmod",
            "lsof",
            "lspci",
            "mount",
            "mvn",
            "netstat",
            "nmap",
            "ntpdate",
            "php",
            "ping",
            "ping2",
            "proftpd",
            "ps",
            "pv",
            "semanage",
            "sensors",
            "showmount",
            "sockstat",
            "sql",
            "ss",
            "stat",
            "sysctl",
            "systemctl",
            "tcpdump",
            "traceroute",
            "tune2fs",
            "ulimit",
            "uptime",
            "vmstat",
            "wdiff",
            "whois",
            "yaml",
            "docker",
            "go",
            "iostat",
        ] {
            let mut except_aliases = except_aliases.iter().map(|s| s.split(',')).flatten();
            if !except_aliases.any(|s| s == *cmd) && (show_all_aliases || which::which(cmd).is_ok())
            {
                println!("alias {}='{} {}';", cmd, grc, cmd);
            }
        }
        std::process::exit(0);
    }

    if command.is_empty() {
        eprintln!("No command specified.");
        std::process::exit(1);
    }

    match colour {
        ColourMode::On => console::set_colors_enabled(true),
        ColourMode::Off => console::set_colors_enabled(false),
        _ => (),
    }

    let pseudo_command = command.join(" ");

    if pseudo_command.is_empty() {}

    let f = File::open("/etc/grc.conf")?;
    let br = std::io::BufReader::new(f);
    let mut cr = GrcConfigReader::new(br.lines());
    let config = cr.find(|(re, _config)| re.is_match(&pseudo_command).unwrap_or(false));
    let rules: Vec<GrcatConfigEntry> = if let Some((_, config)) = config {
        let filename = format!("/usr/share/grc/{}", config);
        let f2 = File::open(filename)?;
        let br = std::io::BufReader::new(f2);
        let cr = GrcatConfigReader::new(br.lines());
        cr.collect()
    } else {
        Vec::default()
    };

    let mut args = command.iter();
    let mut cmd = Command::new(args.next().unwrap());
    cmd.args(args);
    cmd.stdout(Stdio::piped());
    let mut child = cmd.spawn().expect("failed to spawn comamnd");
    let mut stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");

    colourise(&mut stdout, &mut std::io::stdout(), &rules)?;

    Ok(())
}
