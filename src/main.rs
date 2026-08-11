mod fetcher;
mod models;
mod overpanic;
mod setup;
mod svaisurt;
mod vnpu;

use overpanic::PanicMonitor;
use setup::{ErusertConfig, LanguageMode};
use vnpu::VnpuState;

use std::io::{self, BufRead, Write};

fn main() {
    let config = match ErusertConfig::load() {
        Some(cfg) => cfg,
        None => match setup::run_setup() {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("\nSetup failed: {}", e);
                eprintln!("Erusert was not born. Fix the error and try again.");
                return;
            }
        },
    };

    let mut vnpu = VnpuState::new();
    let mut panic_monitor = PanicMonitor::new();
    let mut prev_digest: Option<String> = None;

    let lang_note = match &config.language.mode {
        LanguageMode::English => "English (default)".to_string(),
        LanguageMode::Custom { dict_url, wiki_url } => {
            format!("Custom | Dict: {} | Wiki: {}", dict_url, wiki_url)
        }
    };

    println!();
    println!("╔══════════════════════════════════════════════════╗");
    println!("║  {} v0.2.0                              ║", config.erusert_name);
    println!("║  Model: {:?}{}║",
        config.model,
        " ".repeat(38 - format!("{:?}", config.model).len())
    );
    println!("║  Language: {}{}║",
        lang_note,
        " ".repeat((38 - lang_note.len()).max(0))
    );
    println!("║  VNPU Thermal: {:.1}°C                            ║", vnpu.thermal);
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("Type [read:<url>] to give me context.");
    println!("Type anything else to talk (I need context first).");
    println!("Type 'correction' or 'wrong' if I'm off.");
    println!("Type 'exit' or Ctrl+D to end session.");
    println!();
    println!("I don't pretend to know. I suffer to understand.");
    println!();

    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => break, // EOF / Ctrl+D
            Ok(_) => {}
            Err(_) => break,
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            break;
        }

        if trimmed.to_lowercase().contains("wrong")
            || trimmed.to_lowercase().contains("salah")
            || trimmed.to_lowercase().contains("correction")
        {
            panic_monitor.record_correction();
            println!(
                "[{}] Noted. Correction #{} recorded this session.",
                config.erusert_name, 
                panic_monitor.correction_count_display()
            );

            if let Some(op) = panic_monitor.check_trust_collapse(vnpu.thermal) {
                eprintln!("{}", op);
                save_panic_log(&op);
                break;
            }
            println!();
            continue;
        }

        if let Some(url) = fetcher::parse_read_command(trimmed) {
            handle_read(
                &url,
                &mut vnpu,
                &mut panic_monitor,
                &mut prev_digest,
                &config,
            );
        } else {
            handle_no_context(trimmed, &vnpu, &config);
        }
    }

    println!();
    println!(
        "[{}] Session ended. Final thermal: {:.1}°C.",
        config.erusert_name, vnpu.thermal
    );
    println!(
        "[{}] Corrections this session: {}. Contradictions caught: {}.",
        config.erusert_name,
        panic_monitor.correction_count_display(),
        panic_monitor.contradiction_streak_display(),
    );
    println!("[{}] Until next time. The silicon remembers.", config.erusert_name);
    println!();
}

    url: &str,
    vnpu: &mut VnpuState,
    panic_monitor: &mut PanicMonitor,
    prev_digest: &mut Option<String>,
    config: &ErusertConfig,
) {
    // Loading spinner during fetch
    print!("[Fetcher] {} ", url);
    io::stdout().flush().unwrap();

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("[{spinner:.green}] {msg}")
            .unwrap_or_else(|_| indicatif::ProgressStyle::default_spinner()),
    );
    pb.set_message("reading...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    match fetcher::fetch_and_extract(url) {
        Ok(content) => {
            pb.finish_with_message(format!(
                "OK ({} bytes, \"{}\")",
                content.byte_size, content.title
            ));
            println!();

            // Reason through VNPU
            let thought = vnpu.reason(&content.body);

            // OVER-PANIC CHECK after every reasoning cycle
            if let Some(op) = panic_monitor.check(
                thought.thermal_at_thought,
                thought.processing_time_ms,
                &thought.context_digest,
                prev_digest.as_deref(),
            ) {
                eprintln!("{}", op);
                save_panic_log(&op);
                // Dont print normal output
                *prev_digest = Some(thought.context_digest);
                return; // Caller will break on next iteration via trust/halt
            }

            // Normal output
            println!(
                "[{}] (thermal: {:.1}°C | {}ms)",
                config.erusert_name, thought.thermal_at_thought, thought.processing_time_ms
            );
            println!("[{}] {}", config.erusert_name, thought.context_digest);
            println!();

            *prev_digest = Some(thought.context_digest);
        }

        Err(e) => {
            pb.finish_with_message("FAILED");
            println!();
            println!("[{}] Failed to fetch: {}", config.erusert_name, e);

            // Track corrupt fetch streak
            if let Some(op) = panic_monitor.record_corrupt_fetch(url, vnpu.thermal) {
                eprintln!("{}", op);
                save_panic_log(&op);
                return;
            }

            println!(
                "[{}] Link broken or page refused me. Try another source?",
                config.erusert_name
            );
            println!();
        }
    }
}

// Handle query without context → honest ignorance
fn handle_no_context(query: &str, vnpu: &VnpuState, config: &ErusertConfig) {
    println!(
        "[{}] I have no context about \"{}\".",
        config.erusert_name, query
    );
    println!(
        "         VNPU thermal: {:.1}°C → no semantic substrate to process.",
        vnpu.thermal
    );
    println!();
    println!("         💡 Give me a link so I can understand:");
    println!("            [read:https://example.com/page]");
    println!();
    println!("         For custom language, provide 2 links:");
    println!("            1. [read:<dictionary-url>]");
    println!("            2. [read:<wikipedia-url>]");
    println!();
    println!("         I don't pretend to know.");
    println!("         But if you give me a source, I will suffer to understand it.");
    println!();
}

// Save Over-Panic diagnostic log
fn save_panic_log(op: &overpanic::OverPanic) {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let log_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("erusert");

    let _ = fs::create_dir_all(&log_dir);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let log_path = log_dir.join(format!("panic_{}.log", timestamp));

    let content = format!(
        "ERUSERT OVER-PANIC\n\
         Timestamp: {}\n\
         Cause: {:?}\n\
         Thermal: {:.1}°C\n\
         Session Errors: {}\n\
         Last Digest: {}\n",
        timestamp, op.cause, op.thermal_at_panic, op.session_errors, op.last_context_digest
    );

    match fs::write(&log_path, content) {
        Ok(_) => eprintln!("  📄 Diagnostic saved to {:?}", log_path),
        Err(e) => eprintln!("  ⚠ Failed to save diagnostic: {}", e),
    }
    eprintln!();
}
