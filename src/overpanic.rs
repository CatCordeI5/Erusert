use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct OverPanic {
    pub cause: PanicCause,
    pub thermal_at_panic: f32,
    pub timestamp_ms: u64,
    pub last_context_digest: String,
    pub session_errors: u32,
}

#[derive(Debug, Clone)]
pub enum PanicCause {
    ThermalMeltdown(f32),
    CognitiveContradiction { count: u32 },
    ProcessingSeizure { expected_ms: u64, actual_ms: u64 },
    CorruptedInput { url: String },
    TrustCollapse { corrections: u32 },
}

impl fmt::Display for OverPanic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n╔══════════════════════════════════════════════════╗")?;
        writeln!(f, "║           ⚠ ERUSERT OVER-PANIC ⚠                ║")?;
        writeln!(f, "║   Cognitive halt initiated. Not a crash.         ║")?;
        writeln!(f, "║   I chose to stop before I failed worse.         ║")?;
        writeln!(f, "╚══════════════════════════════════════════════════╝")?;
        writeln!(f)?;
        writeln!(f, "  Cause: {}", self.cause_description())?;
        writeln!(f, "  Thermal at panic: {:.1}°C", self.thermal_at_panic)?;
        writeln!(f, "  Session errors before halt: {}", self.session_errors)?;
        writeln!(f, "  Last context digest: {}", self.last_context_digest)?;
        writeln!(f)?;
        writeln!(f, "  All reasoning suspended. Restart to resume.")?;
        writeln!(f, "  Diagnostic saved to ~/.config/erusert/panic.log")?;
        writeln!(f)
    }
}

impl OverPanic {
    pub fn new(
        cause: PanicCause,
        thermal: f32,
        last_digest: String,
        session_errors: u32,
    ) -> Self {
        Self {
            cause,
            thermal_at_panic: thermal,
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            last_context_digest: last_digest,
            session_errors,
        }
    }

    fn cause_description(&self) -> String {
        match &self.cause {
            PanicCause::ThermalMeltdown(temp) => {
                format!("Virtual thermal meltdown ({:.1}°C). Silicon cannot think at this heat.", temp)
            }
            PanicCause::CognitiveContradiction { count } => {
                format!("{} contradictory outputs in sequence. My reasoning is untrustworthy.", count)
            }
            PanicCause::ProcessingSeizure { expected_ms, actual_ms } => {
                format!("Processing seizure: expected {}ms, took {}ms. Cognitive loop detected.", expected_ms, actual_ms)
            }
            PanicCause::CorruptedInput { url } => {
                format!("Corrupted input from {}. Cannot distinguish signal from noise.", url)
            }
            PanicCause::TrustCollapse { corrections } => {
                format!("Trust collapse: corrected {} times this session. I am failing you.", corrections)
            }
        }
    }
.
    pub fn halt(self) -> Result<!, String> {
        print!("{}", self);
        io::stdout().flush().map_err(|e| e.to_string())?;

        let log_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("erusert")
            .join("panic.log");

        if let Some(parent) = log_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let log_content = format!(
            "OVER-PANIC @ {}\nCause: {:?}\nThermal: {:.1}°C\nErrors: {}\nLast digest: {}\n",
            self.timestamp_ms, self.cause, self.thermal_at_panic,
            self.session_errors, self.last_context_digest
        );
        let _ = fs::write(&log_path, log_content);

        Err(format!("OVER-PANIC: {}", self.cause_description()))
    }
}

pub struct PanicMonitor {
    contradiction_streak: u32,
    correction_count: u32,
    corrupt_fetch_streak: u32,
    baseline_processing_ms: u64,
    last_digest: String,
}

impl PanicMonitor {
    pub fn new() -> Self {
        Self {
            contradiction_streak: 0,
            correction_count: 0,
            corrupt_fetch_streak: 0,
            baseline_processing_ms: 50,
            last_digest: String::new(),
        }
    }
.
    pub fn check(
        &mut self,
        thermal: f32,
        processing_ms: u64,
        digest: &str,
        prev_digest: Option<&str>,
    ) -> Option<OverPanic> {
        self.last_digest = digest.to_string();

        self.baseline_processing_ms =
            (self.baseline_processing_ms as f64 * 0.9 + processing_ms as f64 * 0.1) as u64;

        if thermal > 92.0 {
            return Some(OverPanic::new(
                PanicCause::ThermalMeltdown(thermal),
                thermal,
                self.last_digest.clone(),
                self.correction_count + self.contradiction_streak + self.corrupt_fetch_streak,
            ));
        }

        if let Some(prev) = prev_digest {
            if Self::digests_contradict(prev, digest) {
                self.contradiction_streak += 1;
            } else {
                self.contradiction_streak = 0;
            }
        }

        if self.contradiction_streak >= 3 {
            return Some(OverPanic::new(
                PanicCause::CognitiveContradiction {
                    count: self.contradiction_streak,
                },
                thermal,
                self.last_digest.clone(),
                self.correction_count + self.contradiction_streak + self.corrupt_fetch_streak,
            ));
        }

        if processing_ms > self.baseline_processing_ms * 10 && self.baseline_processing_ms > 0 {
            return Some(OverPanic::new
