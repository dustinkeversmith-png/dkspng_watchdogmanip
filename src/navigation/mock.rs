use crate::navigation::alias::{AliasDefinition, SymbolDefinition};
use crate::navigation::index::NavigationIndex;
use crate::navigation::target::*;
use std::path::PathBuf;

pub fn mock_navigation_index() -> NavigationIndex {
    let mut index = NavigationIndex::new();
    index.add_scope("global", None);
    index.add_scope("project", Some("global".to_string()));
    index.add_scope("docs", Some("project".to_string()));
    index.add_scope("src", Some("project".to_string()));
    index.add_scope("parser", Some("src".to_string()));

    index.add_alias(AliasDefinition::new("project", "readme", NavigationTarget::File(FileTarget {
        path: PathBuf::from("./README.md"), line: Some(1), column: None, marker: None, open_mode: OpenMode::Editor,
    })));
    index.add_alias(AliasDefinition::new("project", "docs", NavigationTarget::Folder(FolderTarget {
        path: PathBuf::from("./docs"), open_mode: OpenMode::Explorer,
    })));
    index.add_alias(AliasDefinition::new("docs", "parser", NavigationTarget::File(FileTarget {
        path: PathBuf::from("./docs/PARSER_PIPELINE.md"), line: None, column: None, marker: Some("BoundarySolver".to_string()), open_mode: OpenMode::Editor,
    })));
    index.add_alias(AliasDefinition::new("parser", "parser", NavigationTarget::File(FileTarget {
        path: PathBuf::from("./src/parser/mod.rs"), line: Some(12), column: None, marker: None, open_mode: OpenMode::Editor,
    })));
    index.add_alias(AliasDefinition::new("global", "terminal", NavigationTarget::Command(CommandTarget {
        command_name: "open_terminal".to_string(), args: vec![".".to_string()],
    })));
    index.add_alias(AliasDefinition::new("project", "parser-workspace", NavigationTarget::Multi {
        targets: vec![
            NavigationTarget::file("./src/parser/mod.rs"),
            NavigationTarget::file("./docs/PARSER_PIPELINE.md"),
            NavigationTarget::folder("./fixtures"),
        ],
    }));

    index.add_symbol(SymbolDefinition {
        name: "BoundarySolver".to_string(),
        scope_id: "docs".to_string(),
        target: NavigationTarget::Symbol(SymbolTarget {
            name: "BoundarySolver".to_string(),
            kind: Some("section".to_string()),
            source_path: Some(PathBuf::from("./docs/PARSER_PIPELINE.md")),
            line: Some(42),
            scope_id: Some("docs".to_string()),
        }),
    });

    index
}
