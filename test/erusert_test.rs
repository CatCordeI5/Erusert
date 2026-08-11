use erusert::vnpu::VnpuState;
use erusert::fetcher::{fetch_and_extract, parse_read_command};
use erusert::setup::{ErusertConfig, LanguageMode};
use std::fs;
use erusert::overpanic::{PanicMonitor, PanicCause, OverPanic};

#[test]
fn overpanic_triggers_on_sustained_high_thermal() {
    let mut monitor = PanicMonitor::new();
    let result = monitor.check(93.0, 50, "test digest", None);
    assert!(result.is_some(), "Should trigger at 93°C");
    let op = result.unwrap();
    assert!(matches!(op.cause, PanicCause::ThermalMeltdown(_)));
}

#[test]
fn overpanic_does_not_trigger_on_normal_thermal() {
    let mut monitor = PanicMonitor::new();
    let result = monitor.check(55.0, 50, "test digest", None);
    assert!(result.is_none(), "Should NOT trigger at 55°C");
}

#[test]
fn overpanic_triggers_on_contradiction_streak() {
    let mut monitor = PanicMonitor::new();

    // Feed contradictory digests
    monitor.check(50.0, 50, "Short calm analysis of the topic.", None);
    monitor.check(50.0, 50, "MASSIVE CHAOTIC UNRELATED WALL OF TEXT THAT CONTRADICTS EVERYTHING PREVIOUSLY SAID ABOUT THIS ENTIRELY DIFFERENT SUBJECT WITH NO RELATION WHATSOEVER TO THE ORIGINAL QUERY AT ALL ABSOLUTELY NOTHING IN COMMON", Some("Short calm analysis of the topic."));
    monitor.check(50.0, 50, "ANOTHER COMPLETELY DIFFERENT GIGANTIC BLOCK OF TEXT THAT SHARES ZERO WORDS WITH ANYTHING BEFORE IT AND IS ABOUT SOMETHING ELSE ENTIRELY LIKE BANANAS OR SPACECRAFT OR WHATEVER", Some("MASSIVE CHAOTIC UNRELATED WALL OF TEXT"));

    let result = monitor.check(
        50.0, 50,
        "YET ANOTHER TOTALLY UNRELATED MASSIVE TEXT BLOCK ABOUT OCEANS OR MOUNTAINS OR SOMETHING THAT HAS ABSOLUTELY NOTHING TO DO WITH PREVIOUS DIGESTS",
        Some("ANOTHER COMPLETELY DIFFERENT GIGANTIC BLOCK"),
    );

    assert!(result.is_some(), "3+ contradictions should trigger Over-Panic");
}

#[test]
fn overpanic_triggers_on_processing_seizure() {
    let mut monitor = PanicMonitor::new();

    // Establish baseline
    for _ in 0..10 {
        monitor.check(50.0, 50, "normal", None);
    }

    // Now feed seizure-level processing time
    let result = monitor.check(50.0, 5000, "seizure test", Some("normal"));
    assert!(result.is_some(), "10x baseline should trigger seizure panic");
}

#[test]
fn overpanic_triggers_on_trust_collapse() {
    let mut monitor = PanicMonitor::new();
    for _ in 0..5 {
        monitor.record_correction();
    }
    let result = monitor.check_trust_collapse(50.0);
    assert!(result.is_some(), "5 corrections should trigger trust collapse");
}

#[test]
fn overpanic_halt_returns_err_never_ok() {
    let op = OverPanic::new(
        PanicCause::ThermalMeltdown(94.0),
        94.0,
        "last digest".to_string(),
        3,
    );
    let result = op.halt();
    assert!(result.is_err(), "Over-Panic halt must NEVER return Ok");
}

#[test]
fn vnpu_low_thermal_produces_calm_digest() {
    let mut vnpu = VnpuState::new();
    // Force low thermal
    vnpu.thermal = 40.0;

    let thought = vnpu.reason("A short context about cats.");

    assert!(thought.thermal_at_thought < 55.0);
    assert!(
        thought.context_digest.contains("absorbed") || thought.context_digest.contains("Structure feels"),
        "Low thermal should produce calm language. Got: {}",
        thought.context_digest
    );
}

#[test]
fn vnpu_high_complexity_raises_thermal() {
    let mut vnpu = VnpuState::new();
    vnpu.thermal = 40.0;

    // Feed dense context
    let dense: String = "word ".repeat(800);
    let thought = vnpu.reason(&dense);

    assert!(
        thought.thermal_at_thought > 50.0,
        "Dense context should raise thermal. Got: {:.1}",
        thought.thermal_at_thought
    );
    assert!(
        thought.complexity_felt > 0.5,
        "Complexity should be felt. Got: {:.2}",
        thought.complexity_felt
    );
}

#[test]
fn vnpu_same_input_different_state_different_output() {

    let context = "Elden Ring lore is fragmented by design.";

    let mut vnpu_cold = VnpuState::new();
    vnpu_cold.thermal = 38.0;
    let thought_cold = vnpu_cold.reason(context);

    let mut vnpu_hot = VnpuState::new();
    vnpu_hot.thermal = 80.0;
    let thought_hot = vnpu_hot.reason(context);

    assert_ne!(
        thought_cold.context_digest, thought_hot.context_digest,
        "Same input at different thermal MUST produce different output.\nCold: {}\nHot: {}",
        thought_cold.context_digest, thought_hot.context_digest
    );
}

#[test]
fn vnpu_processing_time_increases_with_thermal() {
    let mut vnpu = VnpuState::new();

    vnpu.thermal = 40.0;
    let fast = vnpu.reason("test");

    vnpu.thermal = 85.0;
    let slow = vnpu.reason("test");

    assert!(
        slow.processing_time_ms >= fast.processing_time_ms,
        "Higher thermal = longer processing. Fast: {}ms, Slow: {}ms",
        fast.processing_time_ms, slow.processing_time_ms
    );
}


#[test]
fn parse_read_command_valid() {
    let input = "[read:https://eldenring.fandom.com/wiki/Lore]";
    let url = parse_read_command(input);
    assert_eq!(url, Some("https://eldenring.fandom.com/wiki/Lore".to_string()));
}

#[test]
fn parse_read_command_invalid() {
    assert_eq!(parse_read_command("hello world"), None);
    assert_eq!(parse_read_command("[read:]"), Some("".to_string())); // edge case
    assert_eq!(parse_read_command(""), None);
}

#[test]
fn fetch_real_fandom_page() {
    // Integration test: actually hit a real page
    let result = fetch_and_extract("https://eldenring.fandom.com/wiki/Elden_Ring");
    assert!(result.is_ok(), "Should fetch real Fandom page. Err: {:?}", result.err());

    let content = result.unwrap();
    assert!(!content.title.is_empty(), "Title should not be empty");
    assert!(content.byte_size > 100, "Body should have meaningful content. Got {} bytes", content.byte_size);
    assert!(
        content.body.to_lowercase().contains("elden ring"),
        "Body should contain relevant content"
    );
}

#[test]
fn fetch_invalid_url_returns_error() {
    let result = fetch_and_extract("not-a-valid-url");
    assert!(result.is_err(), "Invalid URL should return error");
}


#[test]
fn config_save_and_load_roundtrip() {
    use erusert::models::EruModel;
    use erusert::setup::{LanguageConfig, Personalization};

    let config = ErusertConfig {
        language: LanguageConfig {
            mode: LanguageMode::English,
            dictionary_url: None,
            wikipedia_url: None,
        },
        erusert_name: "TestEru".to_string(),
        personalization: Personalization {
            tone: "calm".to_string(),
            humor_level: "lanang".to_string(),
            formality: "casual".to_string(),
            custom_instructions: "be kind".to_string(),
        },
        model: EruModel::Assistant,
    };

    // Save
    config.save().expect("Config save should succeed");

    // Load
    let loaded = ErusertConfig::load().expect("Config load should succeed");

    assert_eq!(loaded.erusert_name, "TestEru");
    assert_eq!(loaded.personalization.humor_level, "lanang");
    assert!(matches!(loaded.model, EruModel::Assistant));

    // Cleanup
    let _ = fs::remove_file(ErusertConfig::config_path());
}

#[test]
fn model_from_key_mapping() {
    use erusert::models::EruModel;

    assert!(matches!(EruModel::from_key('p'), Some(EruModel::FullStrium)));
    assert!(matches!(EruModel::from_key('u'), Some(EruModel::Assistant)));
    assert!(matches!(EruModel::from_key('q'), Some(EruModel::TerminalEru)));
    assert!(matches!(EruModel::from_key('z'), Some(EruModel::Exathry)));
    assert!(matches!(EruModel::from_key('2'), Some(EruModel::AvlEru)));
    assert!(EruModel::from_key('x').is_none());
}

#[test]
fn full_strium_is_dangerous_others_are_not() {
    use erusert::models::EruModel;

    assert!(EruModel::FullStrium.is_dangerous());
    assert!(!EruModel::Assistant.is_dangerous());
    assert!(!EruModel::TerminalEru.is_dangerous());
    assert!(!EruModel::Exathry.is_dangerous());
    assert!(!EruModel::AvlEru.is_dangerous());
}
