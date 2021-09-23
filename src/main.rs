use itertools::Itertools;
use regex::Regex;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{BufRead, Lines};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

use tokio::process::Command;

struct ConfigReader<A> {
    inner: Lines<A>,
}

impl<A: BufRead> ConfigReader<A> {
    fn new(inner: Lines<A>) -> Self {
        ConfigReader { inner }
    }

    fn next_content_line(&mut self) -> Option<String> {
        let re = Regex::new("^[- \t]*(#|$)").unwrap();
        while let Some(line) = self.inner.next() {
            match line {
                Ok(line2) => {
                    let line2 = line2.replace("\\/", "/");
                    if !re.is_match(&line2) {
                        return Some(line2);
                    }
                }
                Err(_) => break,
            }
        }
        None
    }
}

impl<A: BufRead> Iterator for ConfigReader<A> {
    type Item = (regex::Regex, String);

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

struct GrcatConfigReader<A> {
    inner: Lines<A>,
}

impl<A: BufRead> GrcatConfigReader<A> {
    fn new(inner: Lines<A>) -> Self {
        GrcatConfigReader { inner }
    }

    fn next_alphanumeric(&mut self) -> Option<String> {
        let alphanumeric = Regex::new("^[a-zA-Z0-9]").unwrap();
        while let Some(line) = self.inner.next() {
            match line {
                Ok(line) => {
                    if alphanumeric.is_match(&line) {
                        return Some(line);
                    }
                }
                Err(_) => (),
            }
        }
        None
    }

    fn following(&mut self) -> Option<String> {
        let alphanumeric = Regex::new("^[a-zA-Z0-9]").unwrap();
        if let Some(Ok(line)) = self.inner.next() {
            println!("line: {}", line);
            if alphanumeric.is_match(&line) {
                Some(line)
            } else {
                None
            }
        } else {
            None
        }
    }
}

struct GrcatConfigEntryReader<'a, A> {
    inner: &'a GrcatConfigReader<A>,
}

impl<'a, A: BufRead> GrcatConfigEntryReader<'a, A> {
    fn new(inner: &'a GrcatConfigReader<A>) -> Self {
        GrcatConfigEntryReader { inner }
    }
}

#[derive(Debug)]
struct GrcatConfigEntry {
    regex: regex::Regex,
    colors: Vec<console::Style>,
}

fn style_from_str(text: &str) -> Result<console::Style, ()> {
    Ok(text
        .split(" ")
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
            _ => {
                println!("unhandled style: {}", word);
                Err(())
            }
        })?)
}

fn color_from_str(text: &str) -> Result<console::Color, ()> {
    match text {
        "default" => Ok(console::Color::White),
        "green" => Ok(console::Color::Green),
        "yellow" => Ok(console::Color::Yellow),
        "blue" => Ok(console::Color::Blue),
        "magenta" => Ok(console::Color::Magenta),
        "cyan" => Ok(console::Color::Cyan),
        _ => {
            println!("failed to recognize {}", text);
            Err(())
        }
    }
}

fn try_from_str(text: &str) -> Result<Vec<console::Style>, ()> {
    text.split(",").map(|e| Ok(style_from_str(e)?)).collect()
}

impl<A: BufRead> Iterator for GrcatConfigReader<A> {
    type Item = GrcatConfigEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let re = Regex::new("^([a-z_]+)\\s*=\\s*(.*)$").unwrap();
        let mut ln: String = String::new();
        if let Some(line) = self.next_alphanumeric() {
            ln = line;
            let mut regex: Option<Regex> = None;
            let mut colors: Option<Vec<console::Style>> = None;
            loop {
                let cap = re.captures(&ln).unwrap();
                let key = cap.get(1).unwrap().as_str();
                let value = cap.get(2).unwrap().as_str();
                println!("{:?} -> {:?}", key, value);
                if key == "regexp" {
                    let value = value.replace("\\/", "/");
                    if let Ok(re) = Regex::new(&value) {
                        regex = Some(re);
                    }
                }
                if key == "colours" {
                    println!("found colors {}", value);
                    colors = Some(try_from_str(value).unwrap());
                }

                if let Some(nline) = self.following() {
                    ln = nline;
                    println!("nextline: {}", ln);
                } else {
                    println!("no more lines");
                    break;
                }
            }
            if let Some(regex) = regex {
                println!("{:?}", colors);
                Some(GrcatConfigEntry {
                    regex,
                    colors: colors.unwrap_or(Vec::default()),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    args.next();
    let pseudo_command = args.join(" ");
    println!("cmd: {}", pseudo_command);

    println!("Loading config");
    let f = File::open("/etc/grc.conf")?;
    let br = std::io::BufReader::new(f);
    let mut cr = ConfigReader::new(br.lines());
    let command = cr.find(|(re, config)| re.is_match(&pseudo_command));
    println!("Looking for command");
    let rules: Vec<GrcatConfigEntry> = if let Some((_, config)) = command {
        println!("found match for {}", config);

        let filename = format!("/usr/share/grc/{}", config);
        println!("reading {}", filename);
        let f2 = File::open(filename)?;
        let br = std::io::BufReader::new(f2);
        println!("Loading config");
        let cr = GrcatConfigReader::new(br.lines());
        cr.collect()
    } else {
        Vec::default()
    };
    // for (regex, config) in cr {
    //     if regex.is_match(&pseudo_command) {
    //         println!("found match for {}", config);
    //     }
    // }

    let mut cmd = Command::new("mount");
    cmd.stdout(Stdio::piped());
    let mut child = cmd.spawn().expect("failed to spawn comamnd");
    let stdout = child
        .stdout
        .take()
        .expect("cihld did not have a handle to stdout");
    let mut reader = BufReader::new(stdout).lines();
    tokio::spawn(async move {
        let status = child
            .wait()
            .await
            .expect("child process encountered an error");
        println!("child status was: {}", status);
    });

    let re = regex::Regex::new("^(.*) on (.*) type (.*) \\((.*)\\)")?;
    while let Some(line) = reader.next_line().await? {
        let mut offset = 0;
        //while offset < line.len() {
        //    let entry = &rules.find{ |&r| r.

        for entry in &rules {
        for m in entry.regex.captures_iter(&line) {
            // let colors = [
            //     console::Color::Black,
            //     console::Color::Green,
            //     console::Color::Yellow,
            //     console::Color::Blue,
            //     console::Color::Magenta,
            // ];

            for (index, cap) in m.iter().enumerate() {
                if index == 0 {
                    continue;
                }
                if let Some(cap) = cap {
                    if cap.start() > offset {
                        print!("{}", &line[offset..cap.start()]);
                    }

                    print!(
                        "{}",
                        entry.colors[index].apply_to(
                            // console::style(
                                &line[cap.start()..cap.end()]) // 
                        // .apply(entry.colors[index])
                        // fg(colors[index])
                    );
                    offset = cap.end();
                }
            }
            if offset < line.len() {
                print!("!{}", &line[offset..line.len()]);
            }
            println!("");
        }
        }
        if offset == 0 {
            println!("> {}", line);
        }
    }

    Ok(())
}
