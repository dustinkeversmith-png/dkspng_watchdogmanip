use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ContextBuildConfig {
    pub root: PathBuf,
    pub include_extensions: Vec<String>,
    pub ignore_dirs: Vec<String>,
    pub min_files_per_context: usize,
    pub max_depth: Option<usize>,
    pub create_context_for_every_folder: bool,
    pub parse_context_commands: bool,
}

impl Default for ContextBuildConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            include_extensions: vec!["md".to_string(), "txt".to_string()],
            ignore_dirs: vec![
                ".git".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
            min_files_per_context: 1,
            max_depth: None,
            create_context_for_every_folder: true,
            parse_context_commands: false,
        }
    }
}
