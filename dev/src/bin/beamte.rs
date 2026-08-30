//! The development binary. It lives in `beamte-dev`, so it is never
//! built for a consumer.
//!
//! `check` is the inner loop. `explain` is the one that matters: when a rule
//! misfires, the finding tells you nothing and the tree tells you everything.

use std::process::ExitCode;

use beamte::node::{Node, Unit, Visit, walk};
use beamte::{TestModel, inspect};
use beamte_dev::Parsed;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let ["rules"] = args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        rules();
        return ExitCode::SUCCESS;
    }

    let (command, path) = match args.as_slice() {
        [command, path] => (command.as_str(), path.as_str()),
        _ => {
            eprintln!("usage: beamte <check|explain> <file.py>");
            eprintln!("       beamte rules");
            return ExitCode::from(2);
        }
    };

    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{path}: {error}");
            return ExitCode::from(2);
        }
    };

    let parsed = match Parsed::python(&source) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("{path}: {error}");
            return ExitCode::from(2);
        }
    };

    match command {
        "check" => check(path, &parsed),
        "explain" => {
            explain(&parsed);
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown command `{other}`");
            ExitCode::from(2)
        }
    }
}

fn check(path: &str, parsed: &Parsed) -> ExitCode {
    let unit = Unit::new(path, parsed.source(), parsed.root());
    let findings = inspect(&unit, &TestModel::python());

    for finding in &findings {
        println!(
            "{path}:{}:{}  [{}]  {}",
            finding.span.line, finding.span.column, finding.rule, finding.message
        );
        if let Some(help) = &finding.help {
            println!("  help: {help}");
        }
    }
    println!("beamte: {} finding(s)", findings.len());

    if findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// What the analysis saw: the tree, with roles.
fn explain(parsed: &Parsed) {
    let mut depth = 0usize;
    let mut stack: Vec<usize> = Vec::new();

    walk(parsed.root(), &mut |node| {
        while let Some(&end) = stack.last() {
            if node.span().start_byte >= end {
                stack.pop();
                depth -= 1;
            } else {
                break;
            }
        }

        let roles: Vec<&str> = node.roles().iter().map(|role| role.as_str()).collect();
        let annotation = if roles.is_empty() {
            String::new()
        } else {
            format!("  {}", roles.join(" "))
        };
        println!(
            "{:indent$}{} [{}:{}]{}",
            "",
            node.kind(),
            node.span().line,
            node.span().column,
            annotation,
            indent = depth * 2
        );

        stack.push(node.span().end_byte);
        depth += 1;
        Visit::Descend
    });
}

fn rules() {
    for rule in beamte::catalogue() {
        println!("{}  [{}]", rule.id, rule.property.as_str());
        println!("  {}", rule.summary);
        println!("  {} ({})", rule.citation.title, rule.citation.date);
        println!("  {}", rule.citation.url);
    }
}
