use anyhow::{Context, Result};
use chrono::{Datelike, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub timestamp: String,
    pub level: String,
    pub component: String,
    pub message: String,
    #[serde(default)]
    pub fields: BTreeMap<String, Value>,
}

impl LogLine {
    pub fn new(
        level: impl Into<String>,
        component: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            level: level.into(),
            component: component.into(),
            message: message.into(),
            fields: BTreeMap::new(),
        }
    }
}

pub trait LogSink: Send + Sync {
    fn write(&self, line: &LogLine) -> Result<()>;
    fn tail(&self, limit: usize) -> Result<Vec<LogLine>>;
    fn export_diagnostics_bundle(&self, output_path: &Path) -> Result<PathBuf>;
    fn log_dir(&self) -> &Path;
}

#[derive(Debug, Clone)]
pub struct LogSinkConfig {
    pub dir: PathBuf,
    pub max_file_bytes: u64,
    pub max_files: usize,
}

impl LogSinkConfig {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            max_file_bytes: 8 * 1024 * 1024,
            max_files: 14,
        }
    }
}

struct WriterState {
    day: String,
    index: u32,
    file_path: PathBuf,
    file: File,
}

pub struct JsonlLogSink {
    config: LogSinkConfig,
    state: Mutex<WriterState>,
}

impl JsonlLogSink {
    pub fn new(config: LogSinkConfig) -> Result<Self> {
        fs::create_dir_all(&config.dir)
            .with_context(|| format!("failed to create log directory {}", config.dir.display()))?;

        let day = current_day();
        let (path, index) = latest_file_for_day(&config.dir, &day)?;
        let file = open_append(&path)?;

        Ok(Self {
            config,
            state: Mutex::new(WriterState {
                day,
                index,
                file_path: path,
                file,
            }),
        })
    }

    fn rotate_if_needed(&self, state: &mut WriterState) -> Result<()> {
        let now_day = current_day();
        let mut should_rotate = now_day != state.day;

        if !should_rotate {
            let size = state
                .file
                .metadata()
                .map(|m| m.len())
                .unwrap_or(self.config.max_file_bytes + 1);
            should_rotate = size >= self.config.max_file_bytes;
        }

        if !should_rotate {
            return Ok(());
        }

        if now_day != state.day {
            state.day = now_day;
            state.index = 0;
        } else {
            state.index = state.index.saturating_add(1);
        }

        state.file_path = self
            .config
            .dir
            .join(format!("agent-{}-{:03}.jsonl", state.day, state.index));
        state.file = open_append(&state.file_path)?;

        self.prune_old_files()?;
        Ok(())
    }

    fn prune_old_files(&self) -> Result<()> {
        let mut files = list_log_files(&self.config.dir)?;
        if files.len() <= self.config.max_files {
            return Ok(());
        }

        files.sort();
        let delete_count = files.len().saturating_sub(self.config.max_files);
        for path in files.into_iter().take(delete_count) {
            let _ = fs::remove_file(path);
        }
        Ok(())
    }
}

impl LogSink for JsonlLogSink {
    fn write(&self, line: &LogLine) -> Result<()> {
        let mut state = self.state.lock();
        self.rotate_if_needed(&mut state)?;

        let redacted = redact_log_line(line.clone());
        let payload = serde_json::to_string(&redacted).context("failed to serialize log line")?;

        state
            .file
            .write_all(payload.as_bytes())
            .context("failed to write log line")?;
        state
            .file
            .write_all(b"\n")
            .context("failed to write newline")?;
        state.file.flush().context("failed to flush log line")?;
        Ok(())
    }

    fn tail(&self, limit: usize) -> Result<Vec<LogLine>> {
        let capped_limit = limit.max(1).min(10_000);
        let mut files = list_log_files(&self.config.dir)?;
        files.sort();

        let mut out: Vec<LogLine> = Vec::new();
        for file in files.into_iter().rev() {
            let handle = match File::open(&file) {
                Ok(handle) => handle,
                Err(_) => continue,
            };
            let reader = BufReader::new(handle);
            let mut parsed: Vec<LogLine> = reader
                .lines()
                .map_while(|line| line.ok())
                .filter_map(|line| serde_json::from_str::<LogLine>(&line).ok())
                .collect();
            parsed.reverse();
            out.extend(parsed);
            if out.len() >= capped_limit {
                break;
            }
        }

        out.truncate(capped_limit);
        out.reverse();
        Ok(out)
    }

    fn export_diagnostics_bundle(&self, output_path: &Path) -> Result<PathBuf> {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create diagnostics directory {}",
                    parent.display()
                )
            })?;
        }

        let mut bundle_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(output_path)
            .with_context(|| {
                format!(
                    "failed to create diagnostics bundle {}",
                    output_path.display()
                )
            })?;

        writeln!(
            bundle_file,
            "{{\"generated_at\":\"{}\",\"format\":\"jsonl\"}}",
            Utc::now().to_rfc3339()
        )
        .context("failed to write diagnostics header")?;

        for entry in self.tail(20_000)? {
            let line = redact_log_line(entry);
            let payload =
                serde_json::to_string(&line).context("failed to serialize diagnostics log")?;
            writeln!(bundle_file, "{payload}").context("failed to write diagnostics line")?;
        }

        bundle_file
            .flush()
            .context("failed to flush diagnostics bundle")?;
        Ok(output_path.to_path_buf())
    }

    fn log_dir(&self) -> &Path {
        &self.config.dir
    }
}

fn open_append(path: &Path) -> Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open log file {}", path.display()))
}

fn current_day() -> String {
    let now = Utc::now();
    format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day())
}

fn latest_file_for_day(dir: &Path, day: &str) -> Result<(PathBuf, u32)> {
    let mut highest = 0_u32;
    for path in list_log_files(dir)? {
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        let prefix = format!("agent-{day}-");
        if !name.starts_with(&prefix) {
            continue;
        }
        let suffix = name.trim_end_matches(".jsonl");
        if let Some(index) = suffix
            .rsplit('-')
            .next()
            .and_then(|raw| raw.parse::<u32>().ok())
        {
            highest = highest.max(index);
        }
    }

    let path = dir.join(format!("agent-{day}-{highest:03}.jsonl"));
    Ok((path, highest))
}

fn list_log_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }

    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let path = match entry {
            Ok(item) => item.path(),
            Err(_) => continue,
        };
        let is_jsonl = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "jsonl");
        if is_jsonl {
            out.push(path);
        }
    }

    Ok(out)
}

fn redact_log_line(mut line: LogLine) -> LogLine {
    line.message = redact_string(&line.message);

    for (key, value) in &mut line.fields {
        let key_lower = key.to_ascii_lowercase();
        if key_lower.contains("token")
            || key_lower.contains("secret")
            || key_lower.contains("password")
            || key_lower.contains("api_key")
            || key_lower.contains("authorization")
        {
            *value = Value::String("[REDACTED]".into());
            continue;
        }

        if let Value::String(raw) = value {
            *raw = redact_string(raw);
        }
    }

    line
}

fn redact_string(input: &str) -> String {
    let mut out = input.to_string();
    for marker in ["sk-", "rk_live_", "Bearer ", "api_key="] {
        if let Some(idx) = out.find(marker) {
            let tail = out[idx + marker.len()..]
                .chars()
                .take(24)
                .collect::<String>();
            if !tail.is_empty() {
                out = out.replace(&format!("{marker}{tail}"), &format!("{marker}[REDACTED]"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn writes_and_tails_logs() {
        let tmp = TempDir::new().unwrap();
        let sink = JsonlLogSink::new(LogSinkConfig::new(tmp.path().to_path_buf())).unwrap();

        sink.write(&LogLine::new("info", "agent", "hello")).unwrap();
        let lines = sink.tail(10).unwrap();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].component, "agent");
    }

    #[test]
    fn diagnostics_bundle_redacts_sensitive_values() {
        let tmp = TempDir::new().unwrap();
        let sink = JsonlLogSink::new(LogSinkConfig::new(tmp.path().join("logs"))).unwrap();

        let mut line = LogLine::new("error", "security", "Authorization: Bearer sk-test-secret");
        line.fields
            .insert("api_key".into(), Value::String("sk-real-key".into()));
        sink.write(&line).unwrap();

        let bundle = sink
            .export_diagnostics_bundle(&tmp.path().join("bundle").join("diag.jsonl"))
            .unwrap();
        let body = fs::read_to_string(bundle).unwrap();

        assert!(body.contains("[REDACTED]"));
        assert!(!body.contains("sk-real-key"));
    }
}
