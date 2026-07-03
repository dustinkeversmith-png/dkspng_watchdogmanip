use crate::watchdog::matcher::CompiledWatchSpec;
use crate::watchdog::model::*;
use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct WatchdogPlanner;

impl WatchdogPlanner {
    pub fn plan(spec: &WatchSpec, events: &[FileEvent]) -> Result<Vec<PlannedAction>> {
        let compiled = CompiledWatchSpec::new(spec.clone())?;
        let mut planned = Vec::new();
        for event in events {
            for (rule, reasons) in compiled.matching_rules(event) {
                for action in &rule.actions {
                    planned.push(PlannedAction {
                        watch_id: spec.id.clone(),
                        rule_id: rule.id.clone(),
                        event_id: event.id.clone(),
                        action: action.clone(),
                        reason: reasons.clone(),
                    });
                }
            }
        }
        Ok(planned)
    }

    pub fn expand_routine_actions(
        spec: &WatchSpec,
        planned: &[PlannedAction],
    ) -> Vec<PlannedAction> {
        let mut expanded = Vec::new();
        for action in planned {
            expanded.push(action.clone());
            if let WatchAction::RunRoutine { routine_id } = &action.action {
                if let Some(routine) = spec.routines.iter().find(|r| &r.id == routine_id) {
                    for step in &routine.steps {
                        expanded.push(PlannedAction {
                            watch_id: action.watch_id.clone(),
                            rule_id: action.rule_id.clone(),
                            event_id: action.event_id.clone(),
                            action: routine_step_to_action(step),
                            reason: vec![format!("expanded routine {}", routine.id)],
                        });
                    }
                }
            }
        }
        expanded
    }
}

fn routine_step_to_action(step: &RoutineStep) -> WatchAction {
    match step {
        RoutineStep::RunCommand { command, cwd } => WatchAction::RunCommand {
            command: command.clone(),
            cwd: cwd.clone(),
        },
        RoutineStep::ReindexContext { context_id } => WatchAction::ReindexContext {
            context_id: context_id.clone(),
        },
        RoutineStep::ScanFiles { root } => WatchAction::WriteLog {
            message: format!("scan files under {}", root.display()),
        },
        RoutineStep::ParseMacros { files } => WatchAction::WriteLog {
            message: format!("parse macros in {} files", files.len()),
        },
        RoutineStep::RefreshAliases { context_id } => WatchAction::RefreshAliases {
            context_id: context_id.clone(),
        },
        RoutineStep::EmitEvent { event_type } => WatchAction::EmitHistoryEvent {
            event_type: event_type.clone(),
        },
    }
}
