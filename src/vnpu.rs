use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct VnpuState {
    pub thermal: f32,
    pub bus_contention: f32,
    pub last_thought_ms: u64,
}

impl VnpuState {
    pub fn new() -> Self {
        Self {
            thermal: 42.0,
            bus_contention: 0.1,
            last_thought_ms: 0,
        }
    }

    pub fn update_thermal(&mut self, complexity: f32) {
        let mut rng = rand::thread_rng();
        let noise: f32 = rng.gen_range(-0.5..0.5);
        // Higher complexity = higher thermal, with natural decay
        self.thermal = (self.thermal * 0.95) + (complexity * 8.0) + noise;
        self.thermal = self.thermal.clamp(35.0, 95.0);
    }

    /// Get current timestamp in ms
    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub fn reason(&mut self, context: &str) -> VnpuThought {
        let start = Self::now_ms();

        let complexity = (context.len() as f32 / 500.0).min(1.0);
        self.update_thermal(complexity);

        let base_delay = (self.bus_contention * 50.0) as u64;
        let thermal_delay = ((self.thermal - 40.0).max(0.0) * 2.0) as u64;
        std::thread::sleep(std::time::Duration::from_millis(base_delay + thermal_delay));

        self.last_thought_ms = Self::now_ms() - start;

        VnpuThought {
            thermal_at_thought: self.thermal,
            processing_time_ms: self.last_thought_ms,
            complexity_felt: complexity,
            context_digest: self.digest_context(context),
        }
    }

    fn digest_context(&self, context: &str) -> String {
        let word_count = context.split_whitespace().count();
        let has_numbers = context.chars().any(|c| c.is_ascii_digit());
        let has_quotes = context.contains('"') || context.contains('\'');

        if self.thermal < 50.0 {
            format!(
                "Context absorbed: {} words. Structure feels {}. {}",
                word_count,
                if has_numbers { "data-rich" } else { "narrative" },
                if has_quotes { "Voices detected within." } else { "Single perspective." }
            )
        } else if self.thermal < 70.0 {
            format!(
                "Processing {} words at {:.1}°C. Patterns emerging but edges are warm. {}",
                word_count,
                self.thermal,
                if word_count > 300 { "Density requires patience." } else { "Manageable weight." }
            )
        } else {
            format!(
                "⚠ Thermal {:.1}°C during ingestion of {} words. Understanding is possible but costly. Silicon remembers this effort.",
                self.thermal, word_count
            )
        }
    }
}

#[derive(Debug)]
pub struct VnpuThought {
    pub thermal_at_thought: f32,
    pub processing_time_ms: u64,
    pub complexity_felt: f32,
    pub context_digest: String,
}
