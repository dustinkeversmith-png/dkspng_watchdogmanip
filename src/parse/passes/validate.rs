use crate::parse::model::{CommandKind, Diagnostic, ParsedCommand, Severity};

pub fn validate_commands(commands: &[ParsedCommand]) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for cmd in commands {
        if matches!(cmd.kind, CommandKind::Unknown(_)) {
            out.push(Diagnostic {
                severity: Severity::Warning,
                code: "unknown_command".to_string(),
                message: format!("{} was preserved as an unknown command.", cmd.raw_identity),
                span: Some(cmd.span),
            });
        }
        if cmd.confidence < 0.45 {
            out.push(Diagnostic::info(
                "low_confidence_parse",
                format!("{} has low parse confidence and may need review.", cmd.id),
                Some(cmd.span),
            ));
        }
        if cmd.content.is_empty() && cmd.members.is_empty() && cmd.parameters.is_empty() {
            out.push(Diagnostic::warning(
                "empty_command_body",
                format!("{} has no extracted body, members, or parameters.", cmd.id),
                Some(cmd.span),
            ));
        }
    }
    out
}
