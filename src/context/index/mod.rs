pub mod resolver;

use crate::context::error::{ContextError, Result};
use crate::context::model::{AliasRecord, ContextNode, InheritancePolicy, QueueItem, SymbolRecord};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

pub use resolver::{ContextResolution, ContextResolver};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContextIndex {
    pub contexts: BTreeMap<String, ContextNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextTreeNode {
    pub id: String,
    pub name: String,
    pub children: Vec<ContextTreeNode>,
}

impl ContextIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_context(&mut self, mut ctx: ContextNode) -> Result<()> {
        if self.contexts.contains_key(&ctx.id) {
            return Err(ContextError::DuplicateContext(ctx.id));
        }
        if let Some(parent_id) = &ctx.parent_id {
            let parent =
                self.contexts
                    .get_mut(parent_id)
                    .ok_or_else(|| ContextError::ParentNotFound {
                        child: ctx.id.clone(),
                        parent: parent_id.clone(),
                    })?;
            if !parent.child_ids.contains(&ctx.id) {
                parent.child_ids.push(ctx.id.clone());
            }
        }
        ctx.child_ids.sort();
        self.contexts.insert(ctx.id.clone(), ctx);
        Ok(())
    }

    pub fn upsert_context(&mut self, ctx: ContextNode) {
        if let Some(parent_id) = &ctx.parent_id {
            if let Some(parent) = self.contexts.get_mut(parent_id) {
                if !parent.child_ids.contains(&ctx.id) {
                    parent.child_ids.push(ctx.id.clone());
                }
            }
        }
        self.contexts.insert(ctx.id.clone(), ctx);
    }

    pub fn remove_context(&mut self, context_id: &str) -> Result<Option<String>> {
        let Some(removed) = self.contexts.remove(context_id) else {
            return Err(ContextError::ContextNotFound(context_id.to_string()));
        };

        if let Some(parent_id) = &removed.parent_id {
            if let Some(parent) = self.contexts.get_mut(parent_id) {
                parent.child_ids.retain(|id| id != context_id);
            }
        }

        let child_ids = removed.child_ids.clone();
        let parent_id = removed.parent_id.clone();

        for child_id in child_ids {
            if let Some(child) = self.contexts.get_mut(&child_id) {
                child.parent_id = parent_id.clone();
            }
            if let Some(parent) = parent_id.as_ref().and_then(|id| self.contexts.get_mut(id)) {
                if !parent.child_ids.contains(&child_id) {
                    parent.child_ids.push(child_id);
                }
            }
        }

        Ok(parent_id)
    }

    pub fn attach_local_file(&mut self, context_id: &str, file_path: PathBuf) -> Result<()> {
        self.context_mut(context_id)?.local_files.push(file_path);
        Ok(())
    }

    pub fn context(&self, id: &str) -> Result<&ContextNode> {
        self.contexts
            .get(id)
            .ok_or_else(|| ContextError::ContextNotFound(id.to_string()))
    }

    pub fn context_mut(&mut self, id: &str) -> Result<&mut ContextNode> {
        self.contexts
            .get_mut(id)
            .ok_or_else(|| ContextError::ContextNotFound(id.to_string()))
    }

    pub fn add_alias(&mut self, context_id: &str, alias: AliasRecord) -> Result<()> {
        self.context_mut(context_id)?.aliases.push(alias);
        Ok(())
    }

    pub fn add_queue_item(&mut self, context_id: &str, item: QueueItem) -> Result<()> {
        self.context_mut(context_id)?.queues.push(item);
        Ok(())
    }

    pub fn add_symbol(&mut self, context_id: &str, mut symbol: SymbolRecord) -> Result<()> {
        symbol.context_id = Some(context_id.to_string());
        self.context_mut(context_id)?.symbols.push(symbol);
        Ok(())
    }

    pub fn ancestor_ids(&self, context_id: &str) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let mut cursor = Some(context_id.to_string());
        while let Some(id) = cursor {
            if !seen.insert(id.clone()) {
                break;
            }
            let ctx = self.context(&id)?;
            cursor = ctx.parent_id.clone();
            out.push(id);
        }
        Ok(out)
    }

    pub fn root_context_id(&self, context_id: &str) -> Result<String> {
        self.ancestor_ids(context_id)?
            .last()
            .cloned()
            .ok_or_else(|| ContextError::ContextNotFound(context_id.to_string()))
    }

    pub fn context_lookup_order(&self, context_id: &str) -> Result<Vec<String>> {
        let ctx = self.context(context_id)?;
        let mut order = vec![ctx.id.clone()];
        match ctx.inheritance {
            InheritancePolicy::None => {}
            InheritancePolicy::ParentOnly => {
                if let Some(parent) = &ctx.parent_id {
                    order.push(parent.clone());
                }
            }
            InheritancePolicy::Ancestors => {
                let mut cursor = ctx.parent_id.clone();
                while let Some(parent_id) = cursor {
                    let parent = self.context(&parent_id)?;
                    order.push(parent.id.clone());
                    cursor = parent.parent_id.clone();
                }
            }
            InheritancePolicy::ProjectRoot | InheritancePolicy::WorkspaceRoot => {
                let root = self.root_context_id(context_id)?;
                if root != ctx.id {
                    order.push(root);
                }
            }
            InheritancePolicy::ExplicitImportsOnly => {
                for import in &ctx.imports {
                    if self.contexts.contains_key(import) {
                        order.push(import.clone());
                    }
                }
            }
        }
        if self.contexts.contains_key("global") && !order.iter().any(|id| id == "global") {
            order.push("global".to_string());
        }
        Ok(order)
    }

    pub fn aliases_visible_from(&self, context_id: &str) -> Result<Vec<(&str, &AliasRecord)>> {
        let mut out = Vec::new();
        for id in self.context_lookup_order(context_id)? {
            let ctx = self.context(&id)?;
            for alias in &ctx.aliases {
                out.push((ctx.id.as_str(), alias));
            }
        }
        Ok(out)
    }

    pub fn symbols_visible_from(&self, context_id: &str) -> Result<Vec<(&str, &SymbolRecord)>> {
        let mut out = Vec::new();
        for id in self.context_lookup_order(context_id)? {
            let ctx = self.context(&id)?;
            for symbol in &ctx.symbols {
                out.push((ctx.id.as_str(), symbol));
            }
        }
        Ok(out)
    }

    pub fn tree_from(&self, root_id: &str) -> Result<ContextTreeNode> {
        let root = self.context(root_id)?;
        let mut children = Vec::new();
        for child_id in &root.child_ids {
            children.push(self.tree_from(child_id)?);
        }
        Ok(ContextTreeNode {
            id: root.id.clone(),
            name: root.name.clone(),
            children,
        })
    }

    pub fn direct_child_ids(&self, context_id: &str) -> Result<Vec<String>> {
        Ok(self.context(context_id)?.child_ids.clone())
    }

    pub fn descendant_ids(&self, context_id: &str) -> Result<Vec<String>> {
        let mut out = Vec::new();
        self.collect_descendants(context_id, &mut out)?;
        Ok(out)
    }

    fn collect_descendants(&self, context_id: &str, out: &mut Vec<String>) -> Result<()> {
        let ctx = self.context(context_id)?;
        for child_id in &ctx.child_ids {
            out.push(child_id.clone());
            self.collect_descendants(child_id, out)?;
        }
        Ok(())
    }

    pub fn local_context_ids(&self) -> Vec<String> {
        self.contexts
            .values()
            .filter(|ctx| {
                ctx.metadata
                    .get("local_context")
                    .map(|v| v == "true")
                    .unwrap_or(false)
            })
            .map(|ctx| ctx.id.clone())
            .collect()
    }

    pub fn export_json_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn import_json(input: &str) -> Result<Self> {
        Ok(serde_json::from_str(input)?)
    }
}
