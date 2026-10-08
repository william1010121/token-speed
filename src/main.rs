mod app;
mod args;
mod data;
#[cfg(test)]
mod tests;
mod ui;

use anyhow::{Result, bail};
use args::{Args, Command, Format};
use clap::Parser;
use data::{Clock, Record};
use serde_json::{Value, json};
use std::{
    io::{self, IsTerminal, Write},
    time::Instant,
};

fn main() {
    if let Err(e) = entry() {
        if e.downcast_ref::<io::Error>()
            .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe)
        {
            return;
        }
        eprintln!("token-speed: {e:#}");
        std::process::exit(1);
    }
}
fn entry() -> Result<()> {
    let args = Args::parse();
    if args.days == 0
        || args.limit == 0
        || !args.max_gap.is_finite()
        || args.max_gap <= 0.0
        || !args.interval.is_finite()
        || args.interval <= 0.0
    {
        bail!("days, limit, max-gap and interval must be finite and positive");
    }
    let clock = Clock::new(
        args.timezone
            .as_deref()
            .or(std::env::var("TZ").ok().as_deref()),
    )?;
    data::bounds(&args, clock)?;
    if matches!(args.command, Command::Tui | Command::Watch)
        && args.format == Format::Table
        && io::stdout().is_terminal()
        && io::stdin().is_terminal()
    {
        struct Restore;
        impl Drop for Restore {
            fn drop(&mut self) {
                let _ = crossterm::execute!(io::stdout(), crossterm::event::DisableMouseCapture);
                ratatui::restore();
            }
        }
        let mut terminal = ratatui::init();
        let _restore = Restore;
        crossterm::execute!(io::stdout(), crossterm::event::EnableMouseCapture)?;
        return app::run(args, clock, &mut terminal);
    }
    if args.command == Command::Watch {
        bail!("watch requires an interactive terminal; use summary --format json for scripts");
    }
    report(&args, clock)
}
fn enriched(r: &Record, clock: Clock, max_gap: f64) -> Value {
    let mut value = serde_json::to_value(r).unwrap();
    let status = if r.duration(max_gap).is_some() {
        "estimated"
    } else if r.start.is_none() {
        "missing"
    } else {
        "excluded"
    };
    let o = value.as_object_mut().unwrap();
    o.insert("seconds".into(), json!(r.duration(max_gap)));
    o.insert("tokens_per_second".into(), json!(r.rate(max_gap)));
    o.insert("timing_status".into(), json!(status));
    o.insert(
        "time".into(),
        json!(clock.format(r.timestamp, "%Y-%m-%dT%H:%M:%S%:z")),
    );
    value
}
fn report(args: &Args, clock: Clock) -> Result<()> {
    let mut last = Instant::now();
    let (rows, meta) = data::collect(args, |p, c| {
        if last.elapsed().as_secs() >= 10 {
            eprintln!("Indexing local logs: {p} parsed, {c} cached…");
            last = Instant::now();
        }
    })?;
    let rows = data::select(&rows, args, clock)?;
    let recent = args.command == Command::Recent;
    let selected = if recent {
        &rows[rows.len().saturating_sub(args.limit)..]
    } else {
        &rows
    };
    let summary = data::aggregate(&rows, args.group, clock, args.max_gap);
    let output = if recent {
        selected
            .iter()
            .map(|r| enriched(r, clock, args.max_gap))
            .collect::<Vec<_>>()
    } else {
        summary
            .iter()
            .map(|s| serde_json::to_value(s).unwrap())
            .collect()
    };
    let mut out = io::BufWriter::new(io::stdout().lock());
    match args.format {
        Format::Json => {
            serde_json::to_writer_pretty(
                &mut out,
                &json!({"metric":"estimated_request_output_tokens_per_second","max_gap_seconds":args.max_gap,"timezone":clock.label(),"metadata":meta,"data":output}),
            )?;
            writeln!(out)?;
        }
        Format::Csv => {
            if let Some(first) = output.first().and_then(Value::as_object) {
                let columns: Vec<_> = first.keys().cloned().collect();
                let mut writer = csv::Writer::from_writer(&mut out);
                writer.write_record(&columns)?;
                for row in &output {
                    writer.write_record(columns.iter().map(|c| {
                        let v = &row[c];
                        if let Some(s) = v.as_str() {
                            s.into()
                        } else if v.is_null() {
                            String::new()
                        } else {
                            v.to_string()
                        }
                    }))?;
                }
                writer.flush()?;
            }
        }
        Format::Table => {
            writeln!(
                out,
                "Token Speed · estimated request throughput · output tokens/s"
            )?;
            writeln!(
                out,
                "Timing includes prefill, reasoning, network and request gaps; intervals > {}s excluded.",
                args.max_gap
            )?;
            let (headers, table) = if recent {
                (
                    vec![
                        "Time",
                        "Provider",
                        "Model",
                        "Output",
                        "Seconds",
                        "Est. tok/s",
                        "Session",
                    ],
                    selected
                        .iter()
                        .map(|r| {
                            vec![
                                clock.format(r.timestamp, "%Y-%m-%d %H:%M:%S"),
                                r.provider.clone(),
                                r.model.clone(),
                                r.output_tokens.to_string(),
                                ui::rate(r.duration(args.max_gap)),
                                ui::rate(r.rate(args.max_gap)),
                                r.session.chars().take(12).collect(),
                            ]
                        })
                        .collect::<Vec<_>>(),
                )
            } else {
                (
                    vec![
                        "Period",
                        "Provider",
                        "Model",
                        "Req",
                        "Timed",
                        "Input",
                        "Cached",
                        "Output",
                        "Est. tok/s",
                        "Median",
                    ],
                    summary
                        .iter()
                        .map(|s| {
                            vec![
                                s.period.clone(),
                                s.provider.clone(),
                                s.model.clone(),
                                s.requests.to_string(),
                                format!("{}/{}", s.timed_requests, s.requests),
                                s.input_tokens.to_string(),
                                s.cached_tokens.to_string(),
                                s.output_tokens.to_string(),
                                ui::rate(s.tokens_per_second),
                                ui::rate(s.median_tokens_per_second),
                            ]
                        })
                        .collect(),
                )
            };
            if table.is_empty() {
                writeln!(out, "No matching token records.")?;
            } else {
                let widths: Vec<_> = headers
                    .iter()
                    .enumerate()
                    .map(|(i, h)| {
                        table
                            .iter()
                            .map(|r| r[i].chars().count())
                            .max()
                            .unwrap_or(0)
                            .max(h.len())
                    })
                    .collect();
                for row in
                    std::iter::once(headers.iter().map(|h| h.to_string()).collect::<Vec<_>>())
                        .chain(table.into_iter())
                {
                    writeln!(
                        out,
                        "{}",
                        row.iter()
                            .zip(&widths)
                            .map(|(v, w)| format!(
                                "{v}{}",
                                " ".repeat(w.saturating_sub(v.chars().count()))
                            ))
                            .collect::<Vec<_>>()
                            .join("  ")
                    )?;
                }
            }
            writeln!(
                out,
                "\nFiles: {} parsed, {} cached.",
                meta.parsed_files, meta.cached_files
            )?;
            if !meta.warnings.is_empty() {
                eprintln!(
                    "{} warnings; use --format json for details",
                    meta.warnings.len()
                );
            }
        }
    }
    out.flush()?;
    Ok(())
}
