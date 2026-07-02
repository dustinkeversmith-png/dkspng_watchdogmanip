use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TestLogWriter {
    output_dir: PathBuf,
    test_name: String,
}

impl TestLogWriter {
    pub fn new(
        output_dir: impl Into<PathBuf>,
        test_name: impl Into<String>,
        _root_path: Option<PathBuf>,
    ) -> Result<Self> {
        let output_dir = output_dir.into();
        let test_name = test_name.into();

        fs::create_dir_all(&output_dir)
            .with_context(|| format!("failed to create log dir {}", output_dir.display()))?;

        Ok(Self {
            output_dir,
            test_name,
        })
    }

    pub fn write_json<T: Serialize>(&self, value: &T) -> Result<PathBuf> {
        let path = self.output_dir.join(format!(
            "{}_{}.json",
            sanitize_name(&self.test_name),
            now_unix_ms()
        ));

        fs::write(&path, serde_json::to_string_pretty(value)?)?;
        Ok(path)
    }
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}