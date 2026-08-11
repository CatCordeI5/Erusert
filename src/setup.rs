use crate::models::EruModel;
use dialoguer::{Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErusertConfig {
    pub language: LanguageConfig,
    pub erusert_name: String,
    pub personalization: Personalization,
    pub model: EruModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub mode: LanguageMode,
    pub dictionary_url: Option<String>,
    pub wikipedia_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LanguageMode {
    English,
    Custom { dict_url: String, wiki_url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Personalization {
    pub tone: String,
    pub humor_level: String,
    pub formality: String,
    pub custom_instructions: String,
}

impl ErusertConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("erusert")
            .join("config.json")
    }

    pub fn load() -> Option<Self> {
        let path = Self::config_path();
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())
    }
}

pub fn run_setup() -> Result<ErusertConfig, String> {
    println!("\n╔══════════════════════════════════════╗");
    println!("║       ERUSERT SETUP WIZARD           ║");
    println!("║   Be different. Do your alternate.   ║");
    println!("╚══════════════════════════════════════╝\n");

    // 1. Language Setup
    println!("1. Setup your language!");
    let lang_mode = if Confirm::new()
        .with_prompt("   Use English as default language?")
        .default(true)
        .interact()
        .map_err(|e| e.to_string())?
    {
        LanguageMode::English
    } else {
        println!("   Custom language requires 2 links:");
        println!("   1. Language dictionary URL");
        println!("   2. Wikipedia URL for that language\n");
        let dict: String = Input::new()
            .with_prompt("   [read:Dictionary-URL]")
            .interact_text()
            .map_err(|e| e.to_string())?;
        let wiki: String = Input::new()
            .with_prompt("   [read:Wikipedia-URL]")
            .interact_text()
            .map_err(|e| e.to_string())?;
        LanguageMode::Custom {
            dict_url: dict,
            wiki_url: wiki,
        }
    };

    let language = LanguageConfig {
        mode: lang_mode.clone(),
        dictionary_url: match &lang_mode {
            LanguageMode::English => None,
            LanguageMode::Custom { dict_url, .. } => Some(dict_url.clone()),
        },
        wikipedia_url: match &lang_mode {
            LanguageMode::English => None,
            LanguageMode::Custom { wiki_url, .. } => Some(wiki_url.clone()),
        },
    };
    println!("   ✓ Language configured!\n");

    println!("2. Create your Erusert name!");
    let erusert_name: String = Input::new()
        .with_prompt("   Name your companion")
        .default("Erusert".to_string())
        .interact_text()
        .map_err(|e| e.to_string())?;
    println!("   ✓ Name set to \"{}\"!\n", erusert_name);

    println!("3. Create your AI/{} personalization!", erusert_name);
    println!("   Launching Svisurt editor...\n");
    let personalization = crate::svaisurt::run_svisurt(&erusert_name)?;
    println!("   ✓ Personalization saved!\n");

    println!("4. Download type:");
    let models = [
        EruModel::FullStrium,
        EruModel::Assistant,
        EruModel::TerminalEru,
        EruModel::Exathry,
        EruModel::AvlEru,
    ];
    for m in &models {
        println!("   {}", m.label());
    }
    println!();

    let selection: usize = Select::new()
        .with_prompt("   Choose your Eru model")
        .items(&models.iter().map(|m| m.label()).collect::<Vec<_>>())
        .default(1)
        .interact()
        .map_err(|e| e.to_string())?;

    let model = models[selection].clone();

    if model.is_dangerous() {
        println!("\n   ⚠ WARNING: Full Strium gives FULL SYSTEM ACCESS.");
        println!("   This can modify, delete, or expose ANY file on your computer.");
        let sure = Confirm::new()
            .with_prompt("   Are you ABSOLUTELY sure?")
            .default(false)
            .interact()
            .map_err(|e| e.to_string())?;
        if !sure {
            println!("   Aborted. Please re-run setup and choose a safer model.");
            return Err("User aborted dangerous model selection".to_string());
        }
    }
    println!("   ✓ Model selected: {:?}\n", model);

    let download = Confirm::new()
        .with_prompt("5. Download now?")
        .default(true)
        .interact()
        .map_err(|e| e.to_string())?;

    if download {
        simulate_download(&model);
    } else {
        println!("   Skipped download. You can download later.\n");
    }

    let config = ErusertConfig {
        language,
        erusert_name,
        personalization,
        model,
    };

    config.save()?;
    println!("\n Setup complete! Config saved to {:?}", ErusertConfig::config_path());
    println!("   Run `cargo run` again to start {}.\n", config.erusert_name);

    Ok(config)
}

fn simulate_download(model: &EruModel) {
    let size = model.download_size_mb();
    let pb = ProgressBar::new(size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.green/red}] {pos}/{len} MB | ETA: {eta}")
            .unwrap()
            .progress_chars("██░"),
    );
    pb.set_message(format!("Downloading {:?}...", model));

    for _ in 0..size {
        std::thread::sleep(Duration::from_millis(8));
        pb.inc(1);
    }

    pb.finish_with_message(format!("{:?} downloaded! ✅", model));
    println!();
}
