use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use csv::WriterBuilder;
use notify_rust::Notification;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use winlog::cli::{Cli, Commands, OutputFormat};
use winlog::config::PresetsConfig;
use winlog::record::EventRecord;
use winlog::sigma::{SigmaEngine, SigmaRule};
use winlog::win_api::{EventLogQuery, EventLogSubscription};

fn create_writer(output_path: Option<&PathBuf>) -> Result<Box<dyn Write>> {
    match output_path {
        Some(path) => {
            let file = File::create(path)?;
            Ok(Box::new(BufWriter::new(file)))
        }
        None => Ok(Box::new(io::stdout())),
    }
}

fn resolve_preset_or_channel(
    channel_or_preset: &str,
    preset_file: Option<&PathBuf>,
) -> (String, Option<u32>) {
    let default_path = Path::new("presets.yaml");
    let target_path = preset_file.map(|p| p.as_path()).unwrap_or(default_path);

    if target_path.exists() {
        if let Ok(config) = PresetsConfig::load_from_file(target_path) {
            if let Some(preset) = config.get_preset(channel_or_preset) {
                println!(
                    "{} Using preset '{}' ({}) from {}",
                    "★".yellow().bold(),
                    preset.name.cyan(),
                    channel_or_preset.bold(),
                    target_path.display().to_string().dimmed()
                );
                return (preset.channel.clone(), Some(preset.limit));
            }
        }
    }

    if let Ok(builtin_config) = PresetsConfig::load_embedded_defaults() {
        if let Some(preset) = builtin_config.get_preset(channel_or_preset) {
            println!(
                "{} Using built-in preset '{}' ({})",
                "★".yellow().bold(),
                preset.name.cyan(),
                channel_or_preset.bold()
            );
            return (preset.channel.clone(), Some(preset.limit));
        }
    }

    (channel_or_preset.to_string(), None)
}

fn load_sigma_engine(path: &Path) -> Result<SigmaEngine> {
    let mut engine = SigmaEngine::new();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || entry_path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                if let Ok(rule) = SigmaEngine::load_rule_file(&entry_path) {
                    engine.add_rule(rule);
                }
            }
        }
    } else {
        let rule = SigmaEngine::load_rule_file(path)?;
        engine.add_rule(rule);
    }

    Ok(engine)
}

fn handle_sigma_matches(
    matches: &[&SigmaRule],
    record: &EventRecord,
    notify: bool,
    hook: Option<&str>,
) {
    for rule in matches {
        println!(
            "{} [{}] Sigma Match: '{}' (ID: {})",
            "🚨".red().bold(),
            rule.level.as_deref().unwrap_or("HIGH").red(),
            rule.title.yellow().bold(),
            record.event_id
        );

        if notify {
            let _ = Notification::new()
                .summary(&format!("Sigma Rule Triggered: {}", rule.title))
                .body(&format!(
                    "Event ID: {}\nProvider: {}\nChannel: {}",
                    record.event_id, record.provider, record.channel
                ))
                .icon("dialog-warning")
                .show();
        }

        if let Some(hook_cmd) = hook {
            let _ = Command::new(hook_cmd)
                .env("SIGMA_RULE_TITLE", &rule.title)
                .env("SIGMA_EVENT_ID", record.event_id.to_string())
                .env("SIGMA_PROVIDER", &record.provider)
                .env("SIGMA_CHANNEL", &record.channel)
                .spawn();
        }
    }
}

fn run_static_query(
    input: &str,
    limit: u32,
    format: OutputFormat,
    output_path: Option<&PathBuf>,
) -> Result<()> {
    let query = EventLogQuery::open_path_or_channel(input)?;
    let raw_events = query.next_events(limit)?;

    let mut writer = create_writer(output_path)?;

    match format {
        OutputFormat::Json => {
            let mut records = Vec::new();
            for handle in raw_events {
                let xml = handle.to_xml()?;
                if let Ok(record) = EventRecord::from_xml(&xml) {
                    records.push(record);
                }
            }
            serde_json::to_writer_pretty(&mut writer, &records)?;
            writeln!(writer)?;
        }
        OutputFormat::Ndjson => {
            for handle in raw_events {
                let xml = handle.to_xml()?;
                if let Ok(record) = EventRecord::from_xml(&xml) {
                    serde_json::to_writer(&mut writer, &record)?;
                    writeln!(writer)?;
                }
            }
        }
        OutputFormat::Csv => {
            let mut wtr = WriterBuilder::new().from_writer(writer);
            EventRecord::write_csv_header(&mut wtr)?;
            for handle in raw_events {
                let xml = handle.to_xml()?;
                if let Ok(record) = EventRecord::from_xml(&xml) {
                    record.write_csv_row(&mut wtr)?;
                }
            }
        }
        OutputFormat::Xml => {
            for handle in raw_events {
                let xml = handle.to_xml()?;
                writeln!(writer, "{}\n---", xml)?;
            }
        }
        OutputFormat::Text => {
            for handle in raw_events {
                let xml = handle.to_xml()?;
                if let Ok(record) = EventRecord::from_xml(&xml) {
                    record.print_formatted();
                }
            }
        }
    }

    if let Some(path) = output_path {
        println!(
            "{} Exported events to {}",
            "✔".green().bold(),
            path.display().to_string().cyan()
        );
    }

    Ok(())
}

fn run_tail_stream(
    channel: &str,
    format: OutputFormat,
    output_path: Option<&PathBuf>,
    sigma_rules: Option<&PathBuf>,
    hook: Option<&str>,
    notify: bool,
) -> Result<()> {
    let sigma_engine = if let Some(rules_path) = sigma_rules {
        let engine = load_sigma_engine(rules_path)?;
        println!(
            "{} Loaded Sigma rule engine from {}",
            "🛡".cyan().bold(),
            rules_path.display().to_string().bold()
        );
        Some(engine)
    } else {
        None
    };

    if output_path.is_none() && format == OutputFormat::Text {
        println!(
            "{} Streaming events from channel '{}'. Press {} to stop.\n",
            "●".green().bold(),
            channel.cyan(),
            "Ctrl+C".bold()
        );
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let _ = ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    });

    let sub = EventLogSubscription::subscribe(channel)?;
    let receiver = sub.receiver();
    let writer = create_writer(output_path)?;

    match format {
        OutputFormat::Csv => {
            let mut wtr = WriterBuilder::new().from_writer(writer);
            EventRecord::write_csv_header(&mut wtr)?;

            while running.load(Ordering::SeqCst) {
                if let Ok(xml) = receiver.recv_timeout(std::time::Duration::from_millis(200)) {
                    if let Ok(record) = EventRecord::from_xml(&xml) {
                        if let Some(engine) = &sigma_engine {
                            let matches = engine.matches(&record);
                            if !matches.is_empty() {
                                handle_sigma_matches(&matches, &record, notify, hook);
                            }
                        }
                        record.write_csv_row(&mut wtr)?;
                    }
                }
            }
        }
        OutputFormat::Ndjson | OutputFormat::Json => {
            let mut writer = writer;
            let mut first = true;
            writeln!(writer, "[")?;

            while running.load(Ordering::SeqCst) {
                if let Ok(xml) = receiver.recv_timeout(std::time::Duration::from_millis(200)) {
                    if let Ok(record) = EventRecord::from_xml(&xml) {
                        if let Some(engine) = &sigma_engine {
                            let matches = engine.matches(&record);
                            if !matches.is_empty() {
                                handle_sigma_matches(&matches, &record, notify, hook);
                            }
                        }
                        if !first {
                            writeln!(writer, ",")?;
                        }
                        serde_json::to_writer(&mut writer, &record)?;
                        writer.flush()?;
                        first = false;
                    }
                }
            }

            writeln!(writer, "\n]")?;
            writer.flush()?;
        }
        OutputFormat::Xml => {
            let mut writer = writer;
            while running.load(Ordering::SeqCst) {
                if let Ok(xml) = receiver.recv_timeout(std::time::Duration::from_millis(200)) {
                    if let Some(engine) = &sigma_engine {
                        if let Ok(record) = EventRecord::from_xml(&xml) {
                            let matches = engine.matches(&record);
                            if !matches.is_empty() {
                                handle_sigma_matches(&matches, &record, notify, hook);
                            }
                        }
                    }
                    writeln!(writer, "{}\n---", xml)?;
                    writer.flush()?;
                }
            }
        }
        OutputFormat::Text => {
            while running.load(Ordering::SeqCst) {
                if let Ok(xml) = receiver.recv_timeout(std::time::Duration::from_millis(200)) {
                    if let Ok(record) = EventRecord::from_xml(&xml) {
                        if let Some(engine) = &sigma_engine {
                            let matches = engine.matches(&record);
                            if !matches.is_empty() {
                                handle_sigma_matches(&matches, &record, notify, hook);
                            }
                        }
                        record.print_formatted();
                    }
                }
            }
        }
    }

    if let Some(path) = output_path {
        println!(
            "\n{} Stopped tailing. Saved logs to {}",
            "✔".green().bold(),
            path.display().to_string().cyan()
        );
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Tui { channel, limit }) => {
            let (target_channel, preset_limit) =
                resolve_preset_or_channel(channel, cli.config.as_ref());
            let final_limit = preset_limit.unwrap_or(*limit);
            winlog::tui::run_tui(&target_channel, final_limit)?;
        }
        Some(
            cmd @ Commands::Tail {
                channel,
                output,
                sigma_rules,
                hook,
                notify,
                ..
            },
        ) => {
            let (target_channel, _) = resolve_preset_or_channel(channel, cli.config.as_ref());
            run_tail_stream(
                &target_channel,
                cmd.resolved_tail_format(),
                output.as_ref(),
                sigma_rules.as_ref(),
                hook.as_deref(),
                *notify,
            )?;
        }
        None => {
            let (target_channel, preset_limit) =
                resolve_preset_or_channel(&cli.channel, cli.config.as_ref());
            let final_limit = preset_limit.unwrap_or(cli.limit);

            run_static_query(
                &target_channel,
                final_limit,
                cli.resolved_format(),
                cli.output.as_ref(),
            )?;
        }
    }

    Ok(())
}
