use crate::watchdog::model::{FileEvent, WatchSpec};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn read_watch_spec(path: impl AsRef<Path>) -> Result<WatchSpec> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing watch spec {}", path.display()))
}

pub fn read_file_events_jsonl(path: impl AsRef<Path>) -> Result<Vec<FileEvent>> {
    let path = path.as_ref();
    let f = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let reader = BufReader::new(f);
    let mut events = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let event: FileEvent = serde_json::from_str(&line)
            .with_context(|| format!("parsing {} line {}", path.display(), idx + 1))?;
        events.push(event);
    }
    Ok(events)
}
