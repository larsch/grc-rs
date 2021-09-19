use regex::Regex;
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("/etc/grc.conf")?;
    let br = std::io::BufReader::new(f);
    let cr = ConfigReader::new(br.lines());
    for (regex, config) in cr {
        // println!("{} -> {}", regex, config);
    }

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
        for m in re.captures_iter(&line) {
            let colors = [
                console::Color::Black,
                console::Color::Green,
                console::Color::Yellow,
                console::Color::Blue,
                console::Color::Magenta,
            ];

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
                        console::style(&line[cap.start()..cap.end()]).fg(colors[index])
                    );
                    offset = cap.end();
                }
            }
            if offset < line.len() {
                print!("!{}", &line[offset..line.len()]);
            }
            println!("");
        }
        if offset == 0 {
            println!("> {}", line);
        }
    }

    Ok(())
}
