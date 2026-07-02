use crate::watchdog::model::*;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

#[derive(Debug)]
pub struct CompiledWatchSpec {
    pub spec: WatchSpec,
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

impl CompiledWatchSpec {
    pub fn new(spec: WatchSpec) -> Result<Self> {
        let include = build_globset(&spec.include)?;
        let exclude = build_globset(&spec.exclude)?;
        Ok(Self { spec, include, exclude })
    }

    pub fn path_allowed(&self, path: &Path) -> bool {
        if !self.spec.enabled { return false; }
        let match_path = path.strip_prefix(&self.spec.root).unwrap_or(path);
        if !self.spec.recursive && match_path.components().count() > 1 {
            return false;
        }
        if let Some(exclude) = &self.exclude {
            if exclude.is_match(match_path) || exclude.is_match(path) { return false; }
        }
        if let Some(include) = &self.include {
            return include.is_match(match_path) || include.is_match(path);
        }
        true
    }

    pub fn matching_rules<'a>(&'a self, event: &'a FileEvent) -> Vec<(&'a WatchRule, Vec<String>)> {
        if !self.path_allowed(&event.path) { return vec![]; }
        self.spec.rules.iter()
            .filter_map(|rule| {
                if rule.trigger != event.trigger { return None; }
                let mut reasons = vec![format!("trigger matched: {:?}", rule.trigger)];
                for condition in &rule.conditions {
                    if !condition_matches(condition, event, &mut reasons) {
                        return None;
                    }
                }
                Some((rule, reasons))
            })
            .collect()
    }
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() { return Ok(None); }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(Some(builder.build()?))
}

fn condition_matches(condition: &WatchCondition, event: &FileEvent, reasons: &mut Vec<String>) -> bool {
    match condition {
        WatchCondition::PathContains { text } => {
            let ok = event.path.to_string_lossy().contains(text);
            if ok { reasons.push(format!("path contains {}", text)); }
            ok
        }
        WatchCondition::ExtensionIs { extension } => {
            let ext = event.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let expected = extension.trim_start_matches('.');
            let ok = ext.eq_ignore_ascii_case(expected);
            if ok { reasons.push(format!("extension is {}", expected)); }
            ok
        }
        WatchCondition::MetadataEquals { key, value } => {
            let ok = event.metadata.get(key).map(|v| v == value).unwrap_or(false);
            if ok { reasons.push(format!("metadata {} == {}", key, value)); }
            ok
        }
        WatchCondition::ContextIs { context_id } => {
            let ok = event.context_id.as_deref() == Some(context_id.as_str());
            if ok { reasons.push(format!("context is {}", context_id)); }
            ok
        }
    }
}
