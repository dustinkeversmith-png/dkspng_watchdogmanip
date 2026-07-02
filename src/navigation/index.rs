use crate::navigation::alias::{AliasDefinition, ScopeNode, SymbolDefinition};
use crate::navigation::error::{NavigationError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationIndex {
    pub scopes: BTreeMap<String, ScopeNode>,
    pub aliases: Vec<AliasDefinition>,
    pub symbols: Vec<SymbolDefinition>,
    pub global_scope_id: String,
}

impl Default for NavigationIndex {
    fn default() -> Self {
        Self { scopes: BTreeMap::new(), aliases: Vec::new(), symbols: Vec::new(), global_scope_id: "global".to_string() }
    }
}

impl NavigationIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_scope(&mut self, id: impl Into<String>, parent_id: Option<String>) {
        let id = id.into();
        self.scopes.insert(id.clone(), ScopeNode { id, parent_id });
    }

    pub fn add_alias(&mut self, alias: AliasDefinition) {
        self.aliases.push(alias);
    }

    pub fn add_symbol(&mut self, symbol: SymbolDefinition) {
        self.symbols.push(symbol);
    }

    pub fn scope_order(&self, scope_id: &str) -> Result<Vec<String>> {
        if !self.scopes.contains_key(scope_id) {
            return Err(NavigationError::ScopeNotFound(scope_id.to_string()));
        }
        let mut order = Vec::new();
        let mut seen = HashSet::new();
        let mut cursor = Some(scope_id.to_string());
        while let Some(id) = cursor {
            if !seen.insert(id.clone()) {
                break;
            }
            let scope = self.scopes.get(&id).ok_or_else(|| NavigationError::ScopeNotFound(id.clone()))?;
            order.push(scope.id.clone());
            cursor = scope.parent_id.clone();
        }
        if self.scopes.contains_key(&self.global_scope_id) && !order.iter().any(|id| id == &self.global_scope_id) {
            order.push(self.global_scope_id.clone());
        }
        Ok(order)
    }

    pub fn aliases_in_scope<'a>(&'a self, scope_id: &str, name: &str) -> Vec<&'a AliasDefinition> {
        self.aliases.iter().filter(|alias| alias.scope_id == scope_id && alias.name.eq_ignore_ascii_case(name)).collect()
    }

    pub fn symbols_in_scope<'a>(&'a self, scope_id: &str, name: &str) -> Vec<&'a SymbolDefinition> {
        self.symbols.iter().filter(|symbol| symbol.scope_id == scope_id && symbol.name.eq_ignore_ascii_case(name)).collect()
    }

    pub fn export_json_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn import_json(input: &str) -> Result<Self> {
        Ok(serde_json::from_str(input)?)
    }
}
