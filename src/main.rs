use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser as CliParser;

mod engine;
mod rules;

use engine::{Finding, Scanner};

#[derive(CliParser, Debug)]
#[command(name = "php-doctor")]
#[command(about = "Scan PHP code for likely O(n^2) complexity smells.")]
struct Args {
    /// File or directory to scan.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Include hidden files and directories.
    #[arg(long)]
    hidden: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut scanner = Scanner::new()?;
    let findings = scanner.scan_path(&args.path, args.hidden)?;

    for finding in &findings {
        print_finding(finding)?;
    }

    if findings.is_empty() {
        println!("No O(n^2) complexity warnings found.");
    }

    Ok(())
}

fn print_finding(finding: &Finding) -> Result<()> {
    let source = fs::read_to_string(&finding.path)
        .with_context(|| format!("failed to reread {}", finding.path.display()))?;
    let (line_number, column_number) = line_col(&source, finding.span.start);
    let Some((line_start, line_end, line_text)) = line_at(&source, finding.span.start) else {
        return Ok(());
    };
    let gutter_width = line_number.to_string().len();
    let caret_start = finding.span.start.saturating_sub(line_start);
    let caret_end = if finding.span.end <= line_end {
        finding.span.end.saturating_sub(line_start)
    } else {
        line_end.saturating_sub(line_start)
    };
    let caret_len = caret_end.saturating_sub(caret_start).max(1);

    println!("warning[{}]: {}", finding.code, finding.message);
    println!(
        "{:>width$}┌─ {}:{}:{}",
        "",
        finding.path.display(),
        line_number,
        column_number,
        width = gutter_width + 1
    );
    println!("{:>width$}│", "", width = gutter_width + 1);
    println!("{line_number:>gutter_width$} │ {line_text}");
    println!(
        "{:>width$}│ {}{} {}",
        "",
        " ".repeat(caret_start),
        "^".repeat(caret_len),
        finding.label,
        width = gutter_width + 1
    );
    println!("{:>width$}│", "", width = gutter_width + 1);
    for note in finding.notes {
        println!("{:>width$}= {note}", "", width = gutter_width + 1);
    }
    println!();

    Ok(())
}

fn line_col(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for (index, ch) in source.char_indices() {
        if index >= byte_offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

fn line_at(source: &str, byte_offset: usize) -> Option<(usize, usize, &str)> {
    if byte_offset > source.len() {
        return None;
    }

    let line_start = source[..byte_offset]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let line_end = source[byte_offset..]
        .find('\n')
        .map_or(source.len(), |index| byte_offset + index);

    Some((line_start, line_end, &source[line_start..line_end]))
}
