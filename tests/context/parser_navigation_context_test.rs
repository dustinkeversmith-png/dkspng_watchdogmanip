use macro_os_engines::context::{build_context_index, ContextBuildConfig, ContextStore};
use macro_os_engines::database::new_record_from_parsed_command;
use macro_os_engines::parse::MacroPipeline;
use macro_os_engines::walk::{TreeWalker, TreeWalkerConfig};
use std::fs;

#[test]
fn parser_navigation_context_integration_from_fixture() {
    let root = std::path::PathBuf::from("tests/fixtures/example_docs/planner");
    if !root.exists() {
        eprintln!("skipping: example_docs fixture missing");
        return;
    }

    let walker = TreeWalker::new(
        TreeWalkerConfig::new(&root)
            .recursive(true)
            .include_extensions(["md", "txt"])
            .ignore_dirs([".git", "node_modules", "target"]),
    );
    let walked = walker.walk().expect("walk fixture");

    let pipeline = MacroPipeline::default();
    let parse_db_dir = tempfile::tempdir().unwrap();
    let parse_db =
        macro_os_engines::parse::ParseCommandStore::open(parse_db_dir.path().join("parse.sqlite"))
            .expect("parse db");

    for file in &walked.files {
        let text = fs::read_to_string(&file.path).expect("read fixture file");
        let output = pipeline.parse(file.source_name.clone(), text);
        for command in output.commands {
            let record = new_record_from_parsed_command(file.source_name.clone(), command);
            parse_db.insert_command(&record).expect("insert command");
        }
    }

    let context_index = build_context_index(ContextBuildConfig {
        root: root.clone(),
        include_extensions: vec!["md".to_string(), "txt".to_string()],
        parse_context_commands: true,
        ..Default::default()
    })
    .expect("build context");

    let context_store = ContextStore::open_memory().expect("context db");
    for context_id in context_index.local_context_ids() {
        let node = context_index.context(&context_id).expect("context node");
        context_store
            .insert_context(&macro_os_engines::context::ContextRecord {
                id: node.id.clone(),
                name: node.name.clone(),
                root_path: node.root_path.clone(),
                parent_id: node.parent_id.clone(),
                metadata: node
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect(),
            })
            .expect("insert context");
    }

    assert!(context_index.context_count() >= 1);
    assert!(parse_db.stats().expect("stats").command_count >= 1);
    assert!(context_store.context_count().expect("count") >= 1);
}

trait ContextIndexCount {
    fn context_count(&self) -> usize;
}

impl ContextIndexCount for macro_os_engines::context::ContextIndex {
    fn context_count(&self) -> usize {
        self.local_context_ids().len()
    }
}
