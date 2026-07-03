use crate::navigation::error::{NavigationError, Result};
use crate::navigation::index::NavigationIndex;
use crate::navigation::target::{NavigationTarget, SearchTarget};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavigationAction {
    Resolve,
    Open,
    Preview,
    List,
    Jump,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationRequest {
    pub action: NavigationAction,
    pub query: String,
    pub scope_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavigationPlan {
    pub request: NavigationRequest,
    pub scope_order: Vec<String>,
    pub resolved_targets: Vec<NavigationTarget>,
    pub dry_run_steps: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResolutionOptions {
    pub strict_ambiguity: bool,
    pub fallback_to_symbols: bool,
    pub fallback_to_search: bool,
}

impl Default for ResolutionOptions {
    fn default() -> Self {
        Self {
            strict_ambiguity: true,
            fallback_to_symbols: true,
            fallback_to_search: true,
        }
    }
}

pub struct NavigationResolver<'a> {
    pub index: &'a NavigationIndex,
    pub options: ResolutionOptions,
}

impl<'a> NavigationResolver<'a> {
    pub fn new(index: &'a NavigationIndex) -> Self {
        Self {
            index,
            options: ResolutionOptions::default(),
        }
    }

    pub fn with_options(index: &'a NavigationIndex, options: ResolutionOptions) -> Self {
        Self { index, options }
    }

    pub fn resolve(&self, query: &str, scope_id: &str) -> Result<Vec<NavigationTarget>> {
        let query = query.trim();
        let order = self.index.scope_order(scope_id)?;

        for scope in &order {
            let matches = self.index.aliases_in_scope(scope, query);
            if matches.len() > 1 && self.options.strict_ambiguity {
                return Err(NavigationError::AmbiguousAlias {
                    query: query.to_string(),
                    scope: scope.clone(),
                    candidates: matches
                        .iter()
                        .map(|alias| format!("{}:{}", alias.scope_id, alias.name))
                        .collect(),
                });
            }
            if !matches.is_empty() {
                return Ok(matches
                    .into_iter()
                    .map(|alias| alias.target.clone())
                    .collect());
            }
        }

        if self.options.fallback_to_symbols {
            for scope in &order {
                let matches = self.index.symbols_in_scope(scope, query);
                if !matches.is_empty() {
                    return Ok(matches
                        .into_iter()
                        .map(|symbol| symbol.target.clone())
                        .collect());
                }
            }
        }

        if self.options.fallback_to_search {
            return Ok(vec![NavigationTarget::Search(SearchTarget {
                query: query.to_string(),
                scope: Some(scope_id.to_string()),
                filters: vec![],
            })]);
        }

        Err(NavigationError::AliasNotFound(query.to_string()))
    }

    pub fn plan(&self, request: NavigationRequest) -> Result<NavigationPlan> {
        let scope_order = self.index.scope_order(&request.scope_id)?;
        let resolved_targets = self.resolve(&request.query, &request.scope_id)?;
        let dry_run_steps = resolved_targets
            .iter()
            .map(|target| match target {
                NavigationTarget::File(file) => format!(
                    "Open file {:?} at line {:?}, column {:?}, marker {:?}",
                    file.path, file.line, file.column, file.marker
                ),
                NavigationTarget::Folder(folder) => format!("Open folder {:?}", folder.path),
                NavigationTarget::Symbol(symbol) => {
                    format!("Jump to symbol {} in {:?}", symbol.name, symbol.source_path)
                }
                NavigationTarget::Context(ctx) => {
                    format!("Open context {} at {:?}", ctx.context_id, ctx.root_path)
                }
                NavigationTarget::Workspace(ws) => format!(
                    "Open workspace {} with {} files and {} folders",
                    ws.workspace_id,
                    ws.files.len(),
                    ws.folders.len()
                ),
                NavigationTarget::Command(cmd) => {
                    format!("Run command {} with args {:?}", cmd.command_name, cmd.args)
                }
                NavigationTarget::Search(search) => {
                    format!("Search {:?} in scope {:?}", search.query, search.scope)
                }
                NavigationTarget::Multi { targets } => {
                    format!("Resolve multi target with {} entries", targets.len())
                }
            })
            .collect();

        Ok(NavigationPlan {
            request,
            scope_order,
            resolved_targets,
            dry_run_steps,
            warnings: Vec::new(),
        })
    }
}
