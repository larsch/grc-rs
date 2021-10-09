use std::io::{BufRead, Lines};

use debug_print::debug_println;
use fancy_regex::Regex;

/// Convert a grcat 'colours' option string element into a corresponding
/// 'console::Style' value.
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

/// Convert a grcat 'colours' comma-separated option string into a vector of
/// styles.
fn styles_from_str(text: &str) -> Result<Vec<console::Style>, ()> {
    text.split(',').map(|e| style_from_str(e)).collect()
}

/// 'grc' configuration reader
pub struct GrcConfigReader<A> {
    inner: Lines<A>,
}

impl<A: BufRead> GrcConfigReader<A> {
    /// Construct a new grcat ConfigReader
    pub fn new(inner: Lines<A>) -> Self {
        GrcConfigReader { inner }
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

/// Iterator for ConfigReader that yield the next entry (regex, config) where
/// 'regex' is the command line regexp and 'config' is the file name of the
/// 'grcat' configuration file.
impl<A: BufRead> Iterator for GrcConfigReader<A> {
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
pub struct GrcatConfigReader<A> {
    inner: Lines<A>,
}

impl<A: BufRead> GrcatConfigReader<A> {
    /// Construct a new grcat configuration reader
    pub fn new(inner: Lines<A>) -> Self {
        GrcatConfigReader { inner }
    }

    /// Get the next alpha-numeric line (any non-alphanumeric line are ignored
    /// in grcat).
    fn next_alphanumeric(&mut self) -> Option<String> {
        let alphanumeric = Regex::new("^[a-zA-Z0-9]").unwrap();
        for line in (&mut self.inner).flatten() {
            if alphanumeric.is_match(&line).unwrap_or(false) {
                return Some(line.trim().to_string());
            }
        }
        None
    }

    /// Get the following alpha-numeric line, or None if next line is to be
    /// ignored and signifies the end of the configuration entry.
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

/// A 'grcat' configuration entry consisting of a matching regexp and set of
/// optional options. See 'man grcat' for details.
#[derive(Debug, Clone)]
pub struct GrcatConfigEntry {
    pub regex: Regex,
    pub colors: Vec<console::Style>,
}

impl<A: BufRead> Iterator for GrcatConfigReader<A> {
    type Item = GrcatConfigEntry;

    /// Advances the iterator and returns the next GrcatConfigEntry. The
    /// definition of the configuration file format in 'man grcat' says that
    /// consecutive lines starting with an alphanumeric character are entries
    /// and anything else is ignored.
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
