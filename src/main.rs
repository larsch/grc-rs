use debug_print::debug_println;
use fancy_regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Read, Write};
use std::process::{Command, Stdio};
use std::str::FromStr;

/// 'grc' configuration reader
struct ConfigReader<A> {
    inner: Lines<A>,
}

impl<A: BufRead> ConfigReader<A> {
    /// Construct a new grcat ConfigReader
    fn new(inner: Lines<A>) -> Self {
        ConfigReader { inner }
    }

    /// Read the next line with some actual content
    fn next_content_line(&mut self) -> Option<String> {
        let re = Regex::new("^[- \t]*(#|$)").unwrap();
        for line in &mut self.inner {
            match line {
                Ok(line2) => {
                    if !re.is_match(&line2).unwrap() {
                        return Some(line2.trim().to_string());
                    }
                }
                Err(_) => break,
            }
        }
        None
    }
}

/// Iterator for ConfigReader that yield the next entry (regex, config) where 'regex' is the
/// command line regexp and 'config' is the file name of the 'grcat' configuration file.
impl<A: BufRead> Iterator for ConfigReader<A> {
    type Item = (Regex, String);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(regexp) = self.next_content_line() {
            if let Some(filename) = self.next_content_line() {
                if let Ok(re) = Regex::new(&regexp) {
                    Some((re, filename))
                } else {
                    self.next()
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// 'grcat' configuration reader
struct GrcatConfigReader<A> {
    inner: Lines<A>,
}

impl<A: BufRead> GrcatConfigReader<A> {
    /// Construct a new grcat configuration reader
    fn new(inner: Lines<A>) -> Self {
        GrcatConfigReader { inner }
    }

    /// Get the next alpha-numeric line (any non-alphanumeric line are ignored in grcat).
    fn next_alphanumeric(&mut self) -> Option<String> {
        let alphanumeric = Regex::new("^[a-zA-Z0-9]").unwrap();
        for line in (&mut self.inner).flatten() {
            if alphanumeric.is_match(&line).unwrap_or(false) {
                return Some(line.trim().to_string());
            }
        }
        None
    }

    /// Get the following alpha-numeric line, or None if next line is to be ignored and signifies
    /// the end of the configuration entry.
    fn following(&mut self) -> Option<String> {
        let alphanumeric = Regex::new("^[a-zA-Z0-9]").unwrap();
        if let Some(Ok(line)) = self.inner.next() {
            if alphanumeric.is_match(&line).unwrap_or(false) {
                Some(line)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// A 'grcat' configuration entry consisting of a matching regexp and set of optional options. See
/// 'man grcat' for details.
#[derive(Debug)]
struct GrcatConfigEntry {
    regex: Regex,
    colors: Vec<console::Style>,
}

impl<A: BufRead> Iterator for GrcatConfigReader<A> {
    type Item = GrcatConfigEntry;

    /// Advances the iterator and returns the next GrcatConfigEntry. The definition of the
    /// configuration file format in 'man grcat' says that consecutive lines starting with an
    /// alphanumeric character are entries and anything else is ignored.
    fn next(&mut self) -> Option<Self::Item> {
        let re = Regex::new("^([a-z_]+)\\s*=\\s*(.*)$").unwrap();
        let mut ln: String;
        while let Some(line) = self.next_alphanumeric() {
            ln = line;
            let mut regex: Option<Regex> = None;
            let mut colors: Option<Vec<console::Style>> = None;

            // Loop over all consecutive alpha-numeric lines
            loop {
                let cap = re.captures(&ln).unwrap().unwrap();
                let key = cap.get(1).unwrap().as_str();
                let value = cap.get(2).unwrap().as_str();
                match key {
                    "regexp" => match Regex::new(value) {
                        Ok(re) => {
                            regex = Some(re);
                        }
                        Err(exc) => {
                            debug_println!("Failed regexp: {:?}", exc);
                        }
                    },
                    "colours" => {
                        colors = Some(styles_from_str(value).unwrap());
                    }
                    _ => (), // Ignore unsupported options
                };

                if let Some(nline) = self.following() {
                    ln = nline;
                } else {
                    break;
                }
            }
            if let Some(regex) = regex {
                return Some(GrcatConfigEntry {
                    regex,
                    colors: colors.unwrap_or_default(),
                });
            }
            // Section did not have a 'regexp' entry. Ignore and continue to next.
        }
        None
    }
}

/// Convert a grcat 'colours' option string element into a corresponding 'console::Style' value.
fn style_from_str(text: &str) -> Result<console::Style, ()> {
    text.split(' ')
        .try_fold(console::Style::new(), |style, word| match word {
            "" => Ok(style),
            "unchanged" => Ok(style),
            "underline" => Ok(style.underlined()),
            "default" => Ok(style),
            "black" => Ok(style.black()),
            "red" => Ok(style.red()),
            "green" => Ok(style.green()),
            "yellow" => Ok(style.yellow()),
            "blue" => Ok(style.blue()),
            "magenta" => Ok(style.magenta()),
            "cyan" => Ok(style.cyan()),
            "white" => Ok(style.white()),
            "on_black" => Ok(style.on_black()),
            "on_red" => Ok(style.on_red()),
            "on_green" => Ok(style.on_green()),
            "on_yellow" => Ok(style.on_yellow()),
            "on_blue" => Ok(style.on_blue()),
            "on_magenta" => Ok(style.on_magenta()),
            "on_cyan" => Ok(style.on_cyan()),
            "on_white" => Ok(style.on_white()),
            "bold" => Ok(style.bold()),
            "dark" => Ok(style),
            "bright_black" => Ok(style.bright().black()),
            "bright_red" => Ok(style.bright().red()),
            "bright_green" => Ok(style.bright().green()),
            "bright_yellow" => Ok(style.bright().yellow()),
            "bright_blue" => Ok(style.bright().blue()),
            "bright_magenta" => Ok(style.bright().magenta()),
            "bright_cyan" => Ok(style.bright().cyan()),
            "bright_white" => Ok(style.bright().white()),
            "blink" => Ok(style.blink()),
            _ => {
                println!("unhandled style: {}", word);
                Err(())
            }
        })
}

/// Convert a grcat 'colours' comma-separated option string into a vector of styles.
fn styles_from_str(text: &str) -> Result<Vec<console::Style>, ()> {
    text.split(',').map(|e| style_from_str(e)).collect()
}

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

    let except_aliases: Vec<&str> = except_aliases
        .iter()
        .map(|s| s.split(','))
        .flatten()
        .collect();

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
            if !except_aliases.contains(cmd) && (show_all_aliases || which::which(cmd).is_ok()) {
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
    let mut cr = ConfigReader::new(br.lines());
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

/// Read lines from 'reader' and apply colouring.
///
/// The approach taken here is currently the same as in 'grcat'. Keep an array
/// of styles for each character and paint each match until all regexp have been
/// processed. Then find ranges of same style in this array and wrap the
/// substrings in console escape codes.
fn colourise<R: ?Sized, W: ?Sized>(
    reader: &mut R,
    writer: &mut W,
    rules: &[GrcatConfigEntry],
) -> Result<(), Box<dyn std::error::Error>>
where
    R: Read,
    W: Write,
{
    let reader = BufReader::new(reader).lines();
    for line in reader {
        let line = line?;
        let mut style_ranges: Vec<(usize, usize, &console::Style)> = Vec::new();
        for rule in rules {
            let mut offset = 0;
            while offset < line.len() {
                if let Ok(Some(matches)) = rule.regex.captures_from_pos(&line, offset) {
                    for (i, mmatch) in matches.iter().enumerate() {
                        if let Some(mmatch) = mmatch {
                            let start = mmatch.start();
                            let end = mmatch.end();
                            if i < rule.colors.len() {
                                let style = &rule.colors[i];
                                let range = (start, end, style);
                                style_ranges.push(range);
                            }
                        }
                    }
                    let maybe_match = matches.get(0).unwrap();
                    if maybe_match.end() > maybe_match.start() {
                        offset = maybe_match.end();
                    } else {
                        offset = maybe_match.end() + 1; // skip a char to prevent infinite loop
                    }
                } else {
                    break; // break on no more matches
                }
            }
        }
        let mut char_styles: Vec<&console::Style> = Vec::with_capacity(line.len());
        let default_style = console::Style::new();
        for _ in 0..line.len() {
            char_styles.push(&default_style);
        }
        for (start, end, style) in style_ranges {
            for item in char_styles.iter_mut().take(end).skip(start) {
                *item = style;
            }
        }

        let mut prev_style = &default_style;
        let mut offset = 0;
        for i in 0..line.len() {
            let this_style = char_styles[i];
            if this_style != prev_style {
                if i > 0 {
                    write!(writer, "{}", prev_style.apply_to(&line[offset..i]))?;
                }
                prev_style = this_style;
                offset = i;
            }
        }
        if offset < line.len() {
            write!(writer, "{}", prev_style.apply_to(&line[offset..line.len()]))?;
        }
        writeln!(writer)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(
        input: &str,
        output: &str,
        rules: &[GrcatConfigEntry],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut writer = Vec::new();
        colourise(&mut input.as_bytes(), &mut writer, rules)?;
        assert_eq!(output, String::from_utf8(writer)?);
        Ok(())
    }

    #[test]
    fn test_no_rules() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        test("foo\n", "foo\n", &[])?;
        test("foo\nbar\nbaz\n", "foo\nbar\nbaz\n", &[])?;
        Ok(())
    }

    #[test]
    fn test_simple() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        test(
            "foobarbaz",
            "foo\x1b[34mbar\x1b[0mbaz\n",
            &[GrcatConfigEntry {
                regex: Regex::new("bar")?,
                colors: [console::Style::new().blue()].to_vec(),
            }],
        )
    }

    #[test]
    fn test_adjacent() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        test(
            "foobarbarbaz",
            "foo\x1b[34mbarbar\x1b[0mbaz\n",
            &[GrcatConfigEntry {
                regex: Regex::new("bar")?,
                colors: [console::Style::new().blue()].to_vec(),
            }],
        )
    }

    #[test]
    fn test_multiple() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        test(
            "foobarbazfoobarbazfoobarbaz",
            "foo\x1b[34mbar\x1b[0mbazfoo\x1b[34mbar\x1b[0mbazfoo\x1b[34mbar\x1b[0mbaz\n",
            &[GrcatConfigEntry {
                regex: Regex::new("bar")?,
                colors: [console::Style::new().blue()].to_vec(),
            }],
        )
    }

    #[test]
    fn test_overlap() -> Result<(), Box<dyn std::error::Error>> {
        console::set_colors_enabled(true);
        test(
            "foobarbaz",
            "\x1b[34mfoo\x1b[0m\x1b[31mbarbaz\x1b[0m\n",
            &[
                GrcatConfigEntry {
                    regex: Regex::new("foobar")?,
                    colors: [console::Style::new().blue()].to_vec(),
                },
                GrcatConfigEntry {
                    regex: Regex::new("barbaz")?,
                    colors: [console::Style::new().red()].to_vec(),
                },
            ],
        )
    }
}
