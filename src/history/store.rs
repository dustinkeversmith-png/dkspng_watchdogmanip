use crate::history::model::HistoryEvent;
use anyhow::{Context, Result};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct JsonlEventStore {
    pub path: PathBuf,
}

impl JsonlEventStore {
    pub fn new(path: impl Into<PathBuf>) -> Self { Self { path: path.into() } }

    pub fn append(&self, event: &HistoryEvent) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        let mut f = OpenOptions::new().create(true).append(true).open(&self.path)
            .with_context(|| format!("opening {}", self.path.display()))?;
        let line = serde_json::to_string(event)?;
        writeln!(f, "{}", line)?;
        Ok(())
    }

    pub fn append_many(&self, events: &[HistoryEvent]) -> Result<()> {
        for event in events { self.append(event)?; }
        Ok(())
    }

    pub fn read_all(&self) -> Result<Vec<HistoryEvent>> {
        read_jsonl_events(&self.path)
    }
}

pub fn read_jsonl_events(path: impl AsRef<Path>) -> Result<Vec<HistoryEvent>> {
    let path = path.as_ref();
    if !path.exists() { return Ok(vec![]); }
    let f = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let reader = BufReader::new(f);
    let mut events = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let event: HistoryEvent = serde_json::from_str(&line)
            .with_context(|| format!("parsing {} line {}", path.display(), idx + 1))?;
        events.push(event);
    }
    Ok(events)
}
