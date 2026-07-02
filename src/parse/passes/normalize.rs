use crate::parse::model::{CommandKind, ParsedCommand};

pub fn normalize_command(cmd: &mut ParsedCommand) {
    cmd.tags.sort();
    cmd.tags.dedup();
    cmd.references.sort();
    cmd.references.dedup();
    cmd.statuses.sort();
    cmd.statuses.dedup();

    if let CommandKind::Unknown(raw) = &cmd.kind {
        let normalized = raw.to_ascii_lowercase().replace('_', " ");
        cmd.members.insert(
            "unknown_identity".to_string(),
            serde_json::json!(normalized),
        );
    }

    if cmd
        .title
        .as_ref()
        .map(|s| s.trim().is_empty())
        .unwrap_or(true)
    {
        cmd.title = Some(match &cmd.kind {
            CommandKind::Task => "Untitled Task".to_string(),
            CommandKind::Idea => "Untitled Idea".to_string(),
            CommandKind::Reference => "Reference".to_string(),
            CommandKind::Unknown(k) => format!("Unknown Command: {}", k),
            CommandKind::Inferred(k) => format!("Inferred: {}", k),
            other => format!("{:?}", other),
        });
    }
}
