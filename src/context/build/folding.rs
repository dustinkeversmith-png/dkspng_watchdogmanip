use crate::context::error::Result;
use crate::context::index::ContextIndex;
use crate::walk::WalkedFile;
use std::collections::BTreeMap;
use std::path::Path;

pub fn attach_walked_files(index: &mut ContextIndex, files: &[WalkedFile]) -> Result<()> {
    for file in files {
        if let Some(context_id) = resolve_context_for_source(index, &file.source_name) {
            index.attach_local_file(&context_id, file.path.clone())?;
        }
    }
    Ok(())
}

pub fn fold_small_contexts(
    index: &mut ContextIndex,
    files: &[WalkedFile],
    min_files_per_context: usize,
) -> Result<usize> {
    if min_files_per_context <= 1 {
        return Ok(0);
    }

    let direct_counts = count_direct_files_per_context(index, files);
    let mut folded = 0usize;

    let mut candidates: Vec<_> = index
        .contexts
        .values()
        .filter(|ctx| ctx.parent_id.is_some())
        .map(|ctx| {
            let depth = ctx
                .metadata
                .get("relative_path")
                .map(|path| path.matches('/').count())
                .unwrap_or(0);
            (depth, ctx.id.clone())
        })
        .collect();
    candidates.sort_by(|a, b| b.0.cmp(&a.0));

    for (_, context_id) in candidates {
        let Some(parent_id) = index.context(&context_id)?.parent_id.clone() else {
            continue;
        };
        let count = direct_counts.get(&context_id).copied().unwrap_or(0);
        if count >= min_files_per_context {
            continue;
        }

        if let Ok(node) = index.context(&context_id) {
            let local_files = node.local_files.clone();
            index.remove_context(&context_id)?;
            for file in local_files {
                index.attach_local_file(&parent_id, file)?;
            }
            folded += 1;
        }
    }

    Ok(folded)
}

fn count_direct_files_per_context(
    index: &ContextIndex,
    files: &[WalkedFile],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();

    for file in files {
        if let Some(context_id) = resolve_context_for_source(index, &file.source_name) {
            *counts.entry(context_id).or_default() += 1;
        }
    }

    counts
}

fn resolve_context_for_source(index: &ContextIndex, source_name: &str) -> Option<String> {
    let normalized = source_name.replace('\\', "/");
    let parent_dir = Path::new(&normalized)
        .parent()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();

    let mut best: Option<(usize, String)> = None;

    for ctx in index.contexts.values() {
        let relative = ctx
            .metadata
            .get("relative_path")
            .cloned()
            .unwrap_or_default()
            .replace('\\', "/");

        let matches = if relative.is_empty() {
            !normalized.contains('/')
        } else {
            parent_dir == relative
        };

        if matches {
            let depth = relative.matches('/').count();
            if best.as_ref().map(|(d, _)| depth > *d).unwrap_or(true) {
                best = Some((depth, ctx.id.clone()));
            }
        }
    }

    best.map(|(_, id)| id)
}
