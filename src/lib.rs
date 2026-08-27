#![deny(unsafe_code)]

use std::collections::BTreeMap;

const DEFAULT_INTERVAL_SECS: f64 = 10.0;
const DEFAULT_PIPE_NAME: &str = "pipe_resources";
const DEFAULT_CPU_GLYPH: &str = "";
const DEFAULT_MEMORY_GLYPH: &str = "";
const DEFAULT_NEUTRAL_COLOR: &str = "#cdd6f4";
const DEFAULT_LOW_COLOR: &str = "#a6e3a1";
const DEFAULT_MEDIUM_COLOR: &str = "#f9e2af";
const DEFAULT_HIGH_COLOR: &str = "#f38ba8";
const DEFAULT_BACKGROUND_COLOR: &str = "#1A1B26";
const DEFAULT_MEDIUM_THRESHOLD: f64 = 70.0;
const DEFAULT_HIGH_THRESHOLD: f64 = 90.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    interval_secs: f64,
    pipe_name: String,
    cpu_glyph: String,
    memory_glyph: String,
    neutral_color: String,
    low_color: String,
    medium_color: String,
    high_color: String,
    background_color: String,
    cpu_medium_threshold: f64,
    cpu_high_threshold: f64,
    memory_medium_threshold: f64,
    memory_high_threshold: f64,
    zjstatus_plugin_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interval_secs: DEFAULT_INTERVAL_SECS,
            pipe_name: DEFAULT_PIPE_NAME.to_string(),
            cpu_glyph: DEFAULT_CPU_GLYPH.to_string(),
            memory_glyph: DEFAULT_MEMORY_GLYPH.to_string(),
            neutral_color: DEFAULT_NEUTRAL_COLOR.to_string(),
            low_color: DEFAULT_LOW_COLOR.to_string(),
            medium_color: DEFAULT_MEDIUM_COLOR.to_string(),
            high_color: DEFAULT_HIGH_COLOR.to_string(),
            background_color: DEFAULT_BACKGROUND_COLOR.to_string(),
            cpu_medium_threshold: DEFAULT_MEDIUM_THRESHOLD,
            cpu_high_threshold: DEFAULT_HIGH_THRESHOLD,
            memory_medium_threshold: DEFAULT_MEDIUM_THRESHOLD,
            memory_high_threshold: DEFAULT_HIGH_THRESHOLD,
            zjstatus_plugin_url: None,
        }
    }
}

impl Config {
    pub fn from_zellij_config(configuration: &BTreeMap<String, String>) -> Self {
        let defaults = Self::default();
        let interval_secs = parse_f64(configuration, "interval_secs")
            .filter(|seconds| *seconds >= DEFAULT_INTERVAL_SECS)
            .unwrap_or(defaults.interval_secs);

        Self {
            interval_secs,
            pipe_name: non_empty(configuration, "pipe_name").unwrap_or(defaults.pipe_name),
            cpu_glyph: non_empty(configuration, "cpu_glyph").unwrap_or(defaults.cpu_glyph),
            memory_glyph: non_empty(configuration, "memory_glyph").unwrap_or(defaults.memory_glyph),
            neutral_color: non_empty(configuration, "neutral_color")
                .unwrap_or(defaults.neutral_color),
            low_color: non_empty(configuration, "low_color").unwrap_or(defaults.low_color),
            medium_color: non_empty(configuration, "medium_color").unwrap_or(defaults.medium_color),
            high_color: non_empty(configuration, "high_color").unwrap_or(defaults.high_color),
            background_color: non_empty(configuration, "background_color")
                .unwrap_or(defaults.background_color),
            cpu_medium_threshold: parse_threshold(
                configuration,
                "cpu_medium_threshold",
                defaults.cpu_medium_threshold,
            ),
            cpu_high_threshold: parse_threshold(
                configuration,
                "cpu_high_threshold",
                defaults.cpu_high_threshold,
            ),
            memory_medium_threshold: parse_threshold(
                configuration,
                "memory_medium_threshold",
                defaults.memory_medium_threshold,
            ),
            memory_high_threshold: parse_threshold(
                configuration,
                "memory_high_threshold",
                defaults.memory_high_threshold,
            ),
            zjstatus_plugin_url: non_empty(configuration, "zjstatus_plugin_url"),
        }
    }

    pub fn pipe_payload(&self, output: &str) -> String {
        format!("zjstatus::pipe::{}::{}", self.pipe_name, output)
    }

    pub fn interval_secs(&self) -> f64 {
        self.interval_secs
    }

    pub fn zjstatus_plugin_url(&self) -> Option<&str> {
        self.zjstatus_plugin_url.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CpuSample {
    usage_percent: f64,
}

impl CpuSample {
    pub fn from_iostat(output: &str) -> Option<Self> {
        output
            .lines()
            .rev()
            .find_map(parse_iostat_cpu_line)
            .map(|usage_percent| Self { usage_percent })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemorySample {
    used_bytes: u64,
    total_bytes: u64,
}

impl MemorySample {
    pub fn from_vm_stat_and_total(vm_stat: &str, total_bytes: u64) -> Option<Self> {
        if total_bytes == 0 {
            return None;
        }

        let page_size = parse_page_size(vm_stat)?;
        let pages = parse_vm_stat_pages(vm_stat);
        let used_pages = match (
            pages.get("Anonymous pages"),
            pages.get("Pages wired down"),
            pages.get("Pages occupied by compressor"),
        ) {
            (Some(anonymous), Some(wired), Some(compressor)) => {
                anonymous.saturating_add(*wired).saturating_add(*compressor)
            }
            _ => reclaimable_excluded_used_pages(&pages, total_bytes / page_size),
        };
        let used_bytes = used_pages.saturating_mul(page_size).min(total_bytes);

        Some(Self {
            used_bytes,
            total_bytes,
        })
    }

    fn usage_percent(self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        self.used_bytes as f64 / self.total_bytes as f64 * 100.0
    }
}

pub fn format_resource_output(cpu: CpuSample, memory: MemorySample, config: &Config) -> String {
    let cpu_value = format!("{:.0}%", cpu.usage_percent.clamp(0.0, 100.0));
    let memory_value = format!(
        "{:.1}/{:.1}G",
        bytes_to_gib(memory.used_bytes),
        bytes_to_gib(memory.total_bytes)
    );

    format!(
        "#[fg={neutral},bg={bg}]{cpu_glyph} #[fg={cpu_color},bg={bg}]{cpu_value} #[fg={neutral},bg={bg}]{memory_glyph} #[fg={memory_color},bg={bg}]{memory_value}",
        neutral = config.neutral_color,
        bg = config.background_color,
        cpu_glyph = config.cpu_glyph,
        cpu_color = threshold_color(
            cpu.usage_percent,
            config.cpu_medium_threshold,
            config.cpu_high_threshold,
            config
        ),
        cpu_value = cpu_value,
        memory_glyph = config.memory_glyph,
        memory_color = threshold_color(
            memory.usage_percent(),
            config.memory_medium_threshold,
            config.memory_high_threshold,
            config
        ),
        memory_value = memory_value
    )
}

fn non_empty(configuration: &BTreeMap<String, String>, key: &str) -> Option<String> {
    configuration
        .get(key)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_f64(configuration: &BTreeMap<String, String>, key: &str) -> Option<f64> {
    configuration.get(key)?.trim().parse::<f64>().ok()
}

fn parse_threshold(configuration: &BTreeMap<String, String>, key: &str, default: f64) -> f64 {
    parse_f64(configuration, key)
        .filter(|value| (0.0..=100.0).contains(value))
        .unwrap_or(default)
}

fn parse_iostat_cpu_line(line: &str) -> Option<f64> {
    let values = line
        .split_whitespace()
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if values.len() < 6 {
        return None;
    }

    let cpu_columns = &values[values.len() - 6..values.len() - 3];
    let idle = cpu_columns[2];
    Some((100.0 - idle).clamp(0.0, 100.0))
}

fn parse_page_size(vm_stat: &str) -> Option<u64> {
    let header = vm_stat.lines().next()?;
    let marker = "page size of ";
    let start = header.find(marker)? + marker.len();
    let digits: String = header[start..]
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect();
    digits.parse::<u64>().ok().filter(|bytes| *bytes > 0)
}

fn parse_vm_stat_pages(vm_stat: &str) -> BTreeMap<String, u64> {
    vm_stat
        .lines()
        .filter_map(|line| {
            let (key, raw_value) = line.split_once(':')?;
            let digits: String = raw_value
                .chars()
                .filter(|character| character.is_ascii_digit())
                .collect();
            let value = digits.parse::<u64>().ok()?;
            Some((key.trim_matches('"').trim().to_string(), value))
        })
        .collect()
}

fn reclaimable_excluded_used_pages(pages: &BTreeMap<String, u64>, total_pages: u64) -> u64 {
    let reclaimable = pages
        .get("Pages free")
        .copied()
        .unwrap_or_default()
        .saturating_add(pages.get("Pages speculative").copied().unwrap_or_default())
        .saturating_add(pages.get("File-backed pages").copied().unwrap_or_default())
        .saturating_add(pages.get("Pages purgeable").copied().unwrap_or_default());
    total_pages.saturating_sub(reclaimable)
}

fn threshold_color(usage_percent: f64, medium: f64, high: f64, config: &Config) -> &str {
    if usage_percent >= high {
        &config.high_color
    } else if usage_percent >= medium {
        &config.medium_color
    } else {
        &config.low_color
    }
}

fn bytes_to_gib(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_last_iostat_cpu_sample_without_top() {
        let output = "              disk0       cpu    load average\n    KB/t  tps  MB/s  us sy id   1m   5m   15m\n   15.02  207  3.03  11  6 84  5.66 7.11 7.07\n    7.92  438  3.39  16 10 74  5.66 7.11 7.07\n";

        let sample = CpuSample::from_iostat(output).unwrap();

        assert_eq!(sample.usage_percent, 26.0);
    }

    #[test]
    fn parses_activity_monitor_like_memory_from_vm_stat() {
        let vm_stat = "Mach Virtual Memory Statistics: (page size of 16384 bytes)\nPages free:                                    36428.\nPages active:                                 744022.\nPages inactive:                               733747.\nPages speculative:                              9311.\nPages wired down:                             237816.\nPages purgeable:                               23855.\nFile-backed pages:                            449573.\nAnonymous pages:                             1037507.\nPages occupied by compressor:                 281651.\n";
        let total_bytes = 34_359_738_368;

        let sample = MemorySample::from_vm_stat_and_total(vm_stat, total_bytes).unwrap();

        assert_eq!(sample.used_bytes, (1_037_507 + 237_816 + 281_651) * 16_384);
    }

    #[test]
    fn renders_cpu_before_memory_with_value_only_threshold_colors() {
        let config = Config::default();
        let output = format_resource_output(
            CpuSample {
                usage_percent: 72.0,
            },
            MemorySample {
                used_bytes: 24 * 1024 * 1024 * 1024,
                total_bytes: 32 * 1024 * 1024 * 1024,
            },
            &config,
        );

        assert_eq!(
            output,
            "#[fg=#cdd6f4,bg=#1A1B26] #[fg=#f9e2af,bg=#1A1B26]72% #[fg=#cdd6f4,bg=#1A1B26] #[fg=#f9e2af,bg=#1A1B26]24.0/32.0G"
        );
        assert!(output.contains("bg=#1A1B26"));
    }

    #[test]
    fn builds_zjstatus_pipe_payload_for_resources_widget() {
        let config = Config::default();

        assert_eq!(
            config.pipe_payload("#[fg=#a6e3a1]ok"),
            "zjstatus::pipe::pipe_resources::#[fg=#a6e3a1]ok"
        );
    }

    #[test]
    fn rejects_intervals_below_ten_seconds() {
        let config = Config::from_zellij_config(&BTreeMap::from([(
            "interval_secs".to_string(),
            "1".to_string(),
        )]));

        assert_eq!(config.interval_secs, DEFAULT_INTERVAL_SECS);
    }
}
