use std::io::{BufRead, BufReader, Read, Write};

use crate::grc::GrcatConfigEntry;

/// Read lines from 'reader' and apply colouring.
///
/// The approach taken here is currently the same as in 'grcat'. Keep an array
/// of styles for each character and paint each match until all regexp have been
/// processed. Then find ranges of same style in this array and wrap the
/// substrings in console escape codes.
pub fn colourise<R: ?Sized, W: ?Sized>(
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
    use fancy_regex::Regex;

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
