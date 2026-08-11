mod fetcher;
mod models;
mod setup;
mod svaisurt;
mod vnpu;

use setup::ErusertConfig;
use vnpu::VnpuState;
use std::io::{self, BufRead, Write};

fn main() {
    let config = match ErusertConfig::load() {
        Some(cfg) => cfg,
        None => {
            match setup::run_setup() {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("Setup failed: {}", e);
                    return;
                }
            }
        }
    };

    let lang_note = match &config.language.mode {
        setup::LanguageMode::English => "English (default)".to_string(),
        setup::LanguageMode::Custom { dict_url, wiki_url } => {
            format!("Custom | Dict: {} | Wiki: {}", dict_url, wiki_url)
        }
    };

    let mut vnpu = VnpuState::new();

    println!("\n{} v0.2.0 | Model: {:?}", config.erusert_name, config.model);
    println!("Language: {}", lang_note);
    println!("VNPU thermal: {:.1}°C | Terminal companion", vnpu.thermal);
    println!("Type [read:<url>] to give me context. Type anything else to talk.");
    println!("I don't pretend to know. I suffer to understand.\n");

    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() || input.trim().is_empty() {
            break;
        }

        let trimmed = input.trim();

        if let Some(url) = fetcher::parse_read_command(trimmed) {
            handle_read(&url, &mut vnpu, &config);
        } else {
            handle_no_context(trimmed, &vnpu, &config);
        }
    }

    println!(
        "\n[{}] Session ended. Final thermal: {:.1}°C. See you soon.",
        config.erusert_name, vnpu.thermal
    );
}

fn handle_read(url: &str, vnpu: &mut VnpuState, config: &ErusertConfig) {
    print!("[Fetcher] Fetching {}... ", url);
    io::stdout().flush().unwrap();

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(indicatif::ProgressStyle::default_spinner().template("[{spinner}] {msg}").unwrap());
    pb.set_message("reading...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    match fetcher::fetch_and_extract(url) {
        Ok(content) => {
            pb.finish_and_clear();
            println!("OK ({} bytes, \"{}\")", content.byte_size, content.title);
            let thought = vnpu.reason(&content.body);
            println!("[{}] {}", config.erusert_name, thought.context_digest);
            println!();
        }
        Err(e) => {
            pb.finish_and_clear();
            println!("FAILED");
            println!("[{}] Failed to fetch {}: {}", config.erusert_name, url, e);
            println!();
        }
    }
}

fn handle_no_context(query: &str, vnpu: &VnpuState, config: &ErusertConfig) {
    println!("[{}] I have no context about \"{}\".", config.erusert_name, query);
    println!("         VNPU thermal: {:.1}°C → no semantic substrate.", vnpu.thermal);
    println!();
    println!("          Tip: Give me a link so I can understand.");
    println!("            Format: [read:https://example.com/page]");
    println!();
    println!("         I don't pretend to know. But if you give me a source,");
    println!("         I will suffer to understand it correctly.");
    println!();
}
