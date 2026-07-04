use crate::parse::database::model::DatabaseStats;
use crate::test_logging::model::{
    CommandRef, CrossRefDatabaseSection, CrossRefDatabaseTableLog, CrossRefParseCommandRecord,
    CrossRefParseFileRecord, CrossRefParseSection, CrossRefSearchHit, CrossRefSearchLogRecord,
    CrossRefWalkRecord, CrossRefWalkSection, DatabaseTableLog, DbRef, FileRef,
    ParseCommandLogRecord, ParseFileLogRecord, SearchLogRecord, SourceSpan, TestOutputDiagnostic,
    TestOutputDocument, TestOutputIndexes, TestOutputLink, TestOutputSchema, TestOutputSections,
    TestOutputSummary, TestRunInfo, WalkEfficacySummary, WalkLogRecord,
};
use crate::test_logging::refs::{
    command_ref, db_ref_for_row, file_ref, parse_file_ref, run_ref, search_hit_ref, search_ref,
    table_ref,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub struct TestOutputBuilder {
    run: TestRunInfo,
    walk_summary: Option<WalkEfficacySummary>,
    walk_files: Vec<WalkLogRecord>,
    parse_files: Vec<ParseFileLogRecord>,
    parse_commands: Vec<ParseCommandLogRecord>,
    database_stats: Option<DatabaseStats>,
    database_tables: Vec<DatabaseTableLog>,
    searches: Vec<SearchLogRecord>,
    inserted_command_count: usize,
}

impl TestOutputBuilder {
    pub fn new(mut run: TestRunInfo) -> Self {
        run.run_ref = run_ref(&run.test_name);
        Self {
            run,
            walk_summary: None,
            walk_files: Vec::new(),
            parse_files: Vec::new(),
            parse_commands: Vec::new(),
            database_stats: None,
            database_tables: Vec::new(),
            searches: Vec::new(),
            inserted_command_count: 0,
        }
    }

    pub fn add_walk_summary(&mut self, summary: WalkEfficacySummary) {
        self.walk_summary = Some(summary);
    }

    pub fn add_walk_records(&mut self, records: Vec<WalkLogRecord>) {
        self.walk_files = records;
    }

    pub fn add_parse_file_records(&mut self, records: Vec<ParseFileLogRecord>) {
        self.parse_files = records;
    }

    pub fn add_parse_command_records(&mut self, records: Vec<ParseCommandLogRecord>) {
        self.parse_commands = records;
    }

    pub fn add_inserted_command_count(&mut self, count: usize) {
        self.inserted_command_count = count;
    }

    pub fn add_database_stats(&mut self, stats: DatabaseStats) {
        self.database_stats = Some(stats);
    }

    pub fn add_database_tables(&mut self, tables: Vec<DatabaseTableLog>) {
        self.database_tables = tables;
    }

    pub fn add_search_records(&mut self, records: Vec<SearchLogRecord>) {
        self.searches = records;
    }

    pub fn build(self) -> TestOutputDocument {
        let walk_summary = self.walk_summary.unwrap_or(WalkEfficacySummary {
            root_path: self.run.root_path.clone().unwrap_or_default(),
            total_files_seen: self.walk_files.len(),
            included_files: self.walk_files.len(),
            skipped_files: 0,
            max_depth_seen: self
                .walk_files
                .iter()
                .map(|file| file.depth)
                .max()
                .unwrap_or(0),
            extensions_seen: Vec::new(),
        });

        let mut diagnostics = Vec::new();
        let mut links = Vec::new();
        let mut indexes = TestOutputIndexes::default();

        let cross_walk_files: Vec<CrossRefWalkRecord> = self
            .walk_files
            .iter()
            .enumerate()
            .map(|(index, file)| CrossRefWalkRecord {
                file_ref: file_ref(index),
                source_name: file.source_name.clone(),
                path: file.path.clone(),
                depth: file.depth,
                extension: file.extension.clone(),
                size_bytes: file.size_bytes,
                included: file.included,
                reason: file.reason.clone(),
            })
            .collect();

        for file in &cross_walk_files {
            indexes
                .files_by_source_name
                .insert(file.source_name.clone(), file.file_ref.clone());
        }

        let file_ref_by_source_name = indexes.files_by_source_name.clone();
        let mut parse_file_index_by_source = HashMap::<String, usize>::new();

        let cross_parse_files: Vec<CrossRefParseFileRecord> = self
            .parse_files
            .iter()
            .enumerate()
            .map(|(index, file)| {
                parse_file_index_by_source.insert(file.source_name.clone(), index);
                let matched_file_ref = file_ref_by_source_name
                    .get(&file.source_name)
                    .cloned()
                    .unwrap_or_else(|| file_ref(index));

                CrossRefParseFileRecord {
                    parse_file_ref: parse_file_ref(index),
                    file_ref: matched_file_ref.clone(),
                    source_name: file.source_name.clone(),
                    file_path: file.file_path.clone(),
                    command_count: file.command_count,
                    warning_count: file.warning_count,
                    error_count: file.error_count,
                    command_refs: Vec::new(),
                }
            })
            .collect();

        let mut cross_parse_files = cross_parse_files;
        for parse_file in &cross_parse_files {
            indexes.parse_files_by_file_ref.insert(
                parse_file.file_ref.clone(),
                parse_file.parse_file_ref.clone(),
            );
            links.push(TestOutputLink {
                from: parse_file.file_ref.clone(),
                to: parse_file.parse_file_ref.clone(),
                relation: "parsed_as".to_string(),
            });
        }

        let mut commands_by_source_name: BTreeMap<String, Vec<CommandRef>> = BTreeMap::new();
        let mut commands_by_file_ref: BTreeMap<FileRef, Vec<CommandRef>> = BTreeMap::new();

        let cross_parse_commands: Vec<CrossRefParseCommandRecord> = self
            .parse_commands
            .iter()
            .map(|command| {
                let cmd_ref = command_ref(&command.source_name, &command.command_id);
                let matched_file_ref = file_ref_by_source_name
                    .get(&command.source_name)
                    .cloned()
                    .unwrap_or_default();
                let parse_file_ref_value = indexes
                    .parse_files_by_file_ref
                    .get(&matched_file_ref)
                    .cloned()
                    .unwrap_or_default();

                let source_span = source_span_from_location(&command.location)
                    .or_else(|| parse_source_span(&command.source_trace));
                if source_span.is_none() && !command.source_trace.trim().is_empty() {
                    diagnostics.push(TestOutputDiagnostic {
                        severity: "warn".to_string(),
                        code: "source_trace_unparseable".to_string(),
                        message: format!(
                            "source_trace could not be converted to source_span: {}",
                            command.source_trace
                        ),
                        refs: vec![cmd_ref.clone()],
                    });
                }

                commands_by_source_name
                    .entry(command.source_name.clone())
                    .or_default()
                    .push(cmd_ref.clone());
                commands_by_file_ref
                    .entry(matched_file_ref.clone())
                    .or_default()
                    .push(cmd_ref.clone());

                CrossRefParseCommandRecord {
                    command_ref: cmd_ref,
                    file_ref: matched_file_ref,
                    parse_file_ref: parse_file_ref_value,
                    source_name: command.source_name.clone(),
                    file_path: command.file_path.clone(),
                    command_id: command.command_id.clone(),
                    kind: command.kind.clone(),
                    raw_identity: command.raw_identity.clone(),
                    title: command.title.clone(),
                    source_trace: command.source_trace.clone(),
                    location: command.location.clone(),
                    source_span,
                    content_preview: command.content_preview.clone(),
                    parameters: command.parameters.clone(),
                    tags: command.tags.clone(),
                    references: command.references.clone(),
                    statuses: command.statuses.clone(),
                    db_refs: Vec::new(),
                }
            })
            .collect();

        for command in &cross_parse_commands {
            if let Some(parse_file_index) = parse_file_index_by_source.get(&command.source_name) {
                if let Some(parse_file) = cross_parse_files.get_mut(*parse_file_index) {
                    parse_file.command_refs.push(command.command_ref.clone());
                }
            }

            links.push(TestOutputLink {
                from: command.parse_file_ref.clone(),
                to: command.command_ref.clone(),
                relation: "produced_command".to_string(),
            });
        }

        indexes.commands_by_source_name = commands_by_source_name;
        indexes.commands_by_file_ref = commands_by_file_ref;

        let source_id_to_name = build_source_id_map(&self.database_tables);
        let command_key_to_db_ref =
            build_command_key_to_db_ref(&self.database_tables, &source_id_to_name);

        let cross_database_tables = self
            .database_tables
            .iter()
            .map(|table| {
                let annotated_rows = table
                    .rows
                    .iter()
                    .map(|row| annotate_row_with_db_ref(&table.table_name, row))
                    .collect::<Vec<_>>();

                CrossRefDatabaseTableLog {
                    table_ref: table_ref(&table.table_name),
                    table_name: table.table_name.clone(),
                    row_count: table.row_count,
                    rows: annotated_rows,
                }
            })
            .collect::<Vec<_>>();

        for table in &cross_database_tables {
            let mut row_refs = Vec::new();
            for row in &table.rows {
                if let Some(db_ref) = row
                    .as_object()
                    .and_then(|object| object.get("_db_ref"))
                    .and_then(Value::as_str)
                {
                    row_refs.push(db_ref.to_string());
                }
            }
            indexes
                .database_rows_by_table
                .insert(table.table_name.clone(), row_refs);
        }

        let mut cross_parse_commands = cross_parse_commands;
        for command in &mut cross_parse_commands {
            let key = (command.source_name.clone(), command.command_id.clone());
            if let Some(db_ref) = command_key_to_db_ref.get(&key) {
                command.db_refs.push(db_ref.clone());
                indexes
                    .commands_by_db_ref
                    .insert(db_ref.clone(), command.command_ref.clone());
                indexes
                    .db_refs_by_command_ref
                    .entry(command.command_ref.clone())
                    .or_default()
                    .push(db_ref.clone());

                links.push(TestOutputLink {
                    from: command.command_ref.clone(),
                    to: db_ref.clone(),
                    relation: "inserted_as".to_string(),
                });
            } else {
                diagnostics.push(TestOutputDiagnostic {
                    severity: "warn".to_string(),
                    code: "parsed_command_missing_db_ref".to_string(),
                    message: "Parsed command did not resolve to a parsed_commands database row."
                        .to_string(),
                    refs: vec![command.command_ref.clone()],
                });
            }
        }

        for table in &cross_database_tables {
            if table.table_name != "parsed_commands" {
                continue;
            }

            for row in &table.rows {
                let Some(object) = row.as_object() else {
                    continue;
                };
                let Some(db_ref) = object.get("_db_ref").and_then(Value::as_str) else {
                    continue;
                };
                let source_id = object.get("source_id").and_then(json_as_i64);
                let command_id = object
                    .get("command_id")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let source_name = source_id.and_then(|id| source_id_to_name.get(&id).cloned());

                let resolved = match (source_name, command_id) {
                    (Some(source_name), Some(command_id)) => {
                        command_key_to_db_ref.contains_key(&(source_name, command_id))
                    }
                    _ => false,
                };

                if !resolved {
                    diagnostics.push(TestOutputDiagnostic {
                        severity: "warn".to_string(),
                        code: "database_row_missing_command".to_string(),
                        message: "Database parsed_commands row has no matching parsed command."
                            .to_string(),
                        refs: vec![db_ref.to_string()],
                    });
                }
            }
        }

        let mut unresolved_reference_count = 0usize;
        let cross_searches: Vec<CrossRefSearchLogRecord> = self
            .searches
            .iter()
            .map(|search| {
                let search_ref_value = search_ref(&search.query);
                let mut cross_hits = Vec::new();

                for (hit_index, hit) in search.hits.iter().enumerate() {
                    let search_hit_ref_value = search_hit_ref(&search.query, hit_index);
                    let (resolved_command_ref, resolved_db_ref) =
                        resolve_search_hit(hit, &command_key_to_db_ref);

                    if resolved_command_ref.is_none() && resolved_db_ref.is_none() {
                        unresolved_reference_count += 1;
                        diagnostics.push(TestOutputDiagnostic {
                            severity: "warn".to_string(),
                            code: "search_hit_unresolved_command_ref".to_string(),
                            message: "Search hit could not resolve to command_ref or db_ref."
                                .to_string(),
                            refs: vec![search_hit_ref_value.clone()],
                        });
                    } else if resolved_command_ref.is_none() {
                        unresolved_reference_count += 1;
                        diagnostics.push(TestOutputDiagnostic {
                            severity: "warn".to_string(),
                            code: "search_hit_unresolved_command_ref".to_string(),
                            message: "Search hit resolved to db_ref but not command_ref."
                                .to_string(),
                            refs: vec![search_hit_ref_value.clone()],
                        });
                    }

                    if let Some(command_ref_value) = &resolved_command_ref {
                        indexes
                            .search_hits_by_command_ref
                            .entry(command_ref_value.clone())
                            .or_default()
                            .push(search_hit_ref_value.clone());
                    }

                    links.push(TestOutputLink {
                        from: search_ref_value.clone(),
                        to: search_hit_ref_value.clone(),
                        relation: "produced_hit".to_string(),
                    });

                    if let Some(db_ref) = &resolved_db_ref {
                        links.push(TestOutputLink {
                            from: search_hit_ref_value.clone(),
                            to: db_ref.clone(),
                            relation: "matched_database_row".to_string(),
                        });
                    }

                    if let Some(command_ref_value) = &resolved_command_ref {
                        links.push(TestOutputLink {
                            from: search_hit_ref_value.clone(),
                            to: command_ref_value.clone(),
                            relation: "matched_command".to_string(),
                        });
                    }

                    cross_hits.push(CrossRefSearchHit {
                        search_hit_ref: search_hit_ref_value,
                        command_ref: resolved_command_ref,
                        db_ref: resolved_db_ref,
                        hit: hit.clone(),
                    });
                }

                CrossRefSearchLogRecord {
                    search_ref: search_ref_value,
                    query: search.query.clone(),
                    hit_count: search.hit_count,
                    hits: cross_hits,
                }
            })
            .collect();

        let parsed_source_names: BTreeSet<_> = cross_parse_files
            .iter()
            .map(|file| file.source_name.clone())
            .collect();

        for file in &cross_walk_files {
            if !parsed_source_names.contains(&file.source_name) {
                diagnostics.push(TestOutputDiagnostic {
                    severity: "warn".to_string(),
                    code: "walked_file_missing_parse_file".to_string(),
                    message: "Walked file had no parse_file record.".to_string(),
                    refs: vec![file.file_ref.clone()],
                });
            }
        }

        for parse_file in &cross_parse_files {
            if parse_file.command_count != parse_file.command_refs.len() {
                diagnostics.push(TestOutputDiagnostic {
                    severity: "warn".to_string(),
                    code: "parse_file_command_count_mismatch".to_string(),
                    message: format!(
                        "parse_file command_count ({}) did not match command_refs length ({}).",
                        parse_file.command_count,
                        parse_file.command_refs.len()
                    ),
                    refs: vec![parse_file.parse_file_ref.clone()],
                });
            }
        }

        let database_stats = self.database_stats.unwrap_or(DatabaseStats {
            source_count: 0,
            command_count: 0,
            tag_count: 0,
            reference_count: 0,
        });

        if self.inserted_command_count > 0
            && database_stats.command_count != self.inserted_command_count as i64
        {
            diagnostics.push(TestOutputDiagnostic {
                severity: "warn".to_string(),
                code: "stats_command_count_mismatch".to_string(),
                message: format!(
                    "stats.command_count ({}) did not match inserted command count ({}).",
                    database_stats.command_count, self.inserted_command_count
                ),
                refs: vec![self.run.run_ref.clone()],
            });
        }

        if let Some(parsed_commands_table) = cross_database_tables
            .iter()
            .find(|table| table.table_name == "parsed_commands")
        {
            let dumped_command_rows = parsed_commands_table.rows.len();
            if dumped_command_rows < cross_parse_commands.len() {
                diagnostics.push(TestOutputDiagnostic {
                    severity: "warn".to_string(),
                    code: "database_dump_truncated".to_string(),
                    message: format!(
                        "parsed_commands dump contains {dumped_command_rows} rows but {command_count} parsed commands were recorded; increase dump limit.",
                        command_count = cross_parse_commands.len()
                    ),
                    refs: vec![table_ref("parsed_commands")],
                });
            }
        }

        let search_hit_count = cross_searches.iter().map(|search| search.hits.len()).sum();

        let summary = TestOutputSummary {
            file_count: cross_walk_files.len(),
            parse_file_count: cross_parse_files.len(),
            parsed_command_count: cross_parse_commands.len(),
            inserted_command_count: self.inserted_command_count,
            database_command_count: database_stats.command_count,
            search_count: cross_searches.len(),
            search_hit_count,
            diagnostic_count: diagnostics.len(),
            unresolved_reference_count,
        };

        TestOutputDocument {
            schema: TestOutputSchema::default(),
            run: self.run,
            sections: TestOutputSections {
                walk: CrossRefWalkSection {
                    summary: walk_summary,
                    files: cross_walk_files,
                },
                parse: CrossRefParseSection {
                    files: cross_parse_files,
                    commands: cross_parse_commands,
                },
                database: CrossRefDatabaseSection {
                    stats: database_stats,
                    tables: cross_database_tables,
                },
                searches: cross_searches,
            },
            indexes,
            links,
            diagnostics,
            summary,
        }
    }
}

fn source_span_from_location(location: &crate::parse::model::SourceLocation) -> Option<SourceSpan> {
    Some(SourceSpan {
        start_line: location.start_line as u32,
        end_line: location.end_line.unwrap_or(location.start_line) as u32,
    })
}

fn parse_source_span(source_trace: &str) -> Option<SourceSpan> {
    if let Some(rest) = source_trace.strip_prefix("lines ") {
        let mut parts = rest.split('-');
        let start = parts.next()?.trim().parse().ok()?;
        let end = parts.next()?.trim().parse().ok()?;
        return Some(SourceSpan {
            start_line: start,
            end_line: end,
        });
    }

    let (_, line_part) = source_trace.rsplit_once(':')?;
    if let Some((start, end)) = line_part.split_once('-') {
        return Some(SourceSpan {
            start_line: start.trim().parse().ok()?,
            end_line: end.trim().parse().ok()?,
        });
    }
    let line = line_part.trim().parse().ok()?;
    Some(SourceSpan {
        start_line: line,
        end_line: line,
    })
}

fn build_source_id_map(tables: &[DatabaseTableLog]) -> BTreeMap<i64, String> {
    let mut map = BTreeMap::new();
    let Some(sources) = tables.iter().find(|table| table.table_name == "sources") else {
        return map;
    };

    for row in &sources.rows {
        let Some(object) = row.as_object() else {
            continue;
        };
        let Some(id) = object.get("id").and_then(json_as_i64) else {
            continue;
        };
        let Some(source_name) = object.get("source_name").and_then(Value::as_str) else {
            continue;
        };
        map.insert(id, source_name.to_string());
    }

    map
}

fn build_command_key_to_db_ref(
    tables: &[DatabaseTableLog],
    source_id_to_name: &BTreeMap<i64, String>,
) -> HashMap<(String, String), DbRef> {
    let mut map = HashMap::new();
    let Some(parsed_commands) = tables
        .iter()
        .find(|table| table.table_name == "parsed_commands")
    else {
        return map;
    };

    for row in &parsed_commands.rows {
        let Some(object) = row.as_object() else {
            continue;
        };
        let Some(id) = object.get("id").and_then(json_as_i64) else {
            continue;
        };
        let Some(command_id) = object.get("command_id").and_then(Value::as_str) else {
            continue;
        };
        let Some(source_id) = object.get("source_id").and_then(json_as_i64) else {
            continue;
        };
        let Some(source_name) = source_id_to_name.get(&source_id) else {
            continue;
        };

        map.insert(
            (source_name.clone(), command_id.to_string()),
            format!("db:parsed_commands:{id}"),
        );
    }

    map
}

fn annotate_row_with_db_ref(table_name: &str, row: &Value) -> Value {
    let mut object = row.as_object().cloned().unwrap_or_default();
    if let Some(db_ref) = db_ref_for_row(table_name, &object) {
        object.insert("_db_ref".to_string(), Value::String(db_ref));
    }
    Value::Object(object)
}

fn resolve_search_hit(
    hit: &Value,
    command_key_to_db_ref: &HashMap<(String, String), DbRef>,
) -> (Option<CommandRef>, Option<DbRef>) {
    let Some(object) = hit.as_object() else {
        return (None, None);
    };

    if let (Some(source_name), Some(command_id)) = (
        object.get("source_name").and_then(Value::as_str),
        object.get("command_id").and_then(Value::as_str),
    ) {
        let cmd_ref = command_ref(source_name, command_id);
        let db_ref = command_key_to_db_ref
            .get(&(source_name.to_string(), command_id.to_string()))
            .cloned()
            .or_else(|| {
                object
                    .get("id")
                    .and_then(json_as_i64)
                    .map(|id| format!("db:parsed_commands:{id}"))
            });
        return (Some(cmd_ref), db_ref);
    }

    if let Some(id) = object.get("id").and_then(json_as_i64) {
        return (None, Some(format!("db:parsed_commands:{id}")));
    }

    (None, None)
}

fn json_as_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_logging::model::TestRunInfo;
    use serde_json::json;

    #[test]
    fn builder_assigns_cross_references() {
        let mut builder = TestOutputBuilder::new(TestRunInfo {
            run_ref: String::new(),
            test_name: "sample_test".to_string(),
            started_at_unix_ms: 1,
            root_path: None,
            temporary_sqlite_db_path: None,
            log_kind: "fixture".to_string(),
        });

        builder.add_walk_records(vec![WalkLogRecord {
            source_name: "demo.txt".to_string(),
            path: "demo.txt".into(),
            depth: 0,
            extension: Some("txt".to_string()),
            size_bytes: 10,
            included: true,
            reason: "included".to_string(),
        }]);

        builder.add_parse_file_records(vec![ParseFileLogRecord {
            source_name: "demo.txt".to_string(),
            file_path: "demo.txt".into(),
            command_count: 1,
            warning_count: 0,
            error_count: 0,
        }]);

        builder.add_parse_command_records(vec![ParseCommandLogRecord {
            source_name: "demo.txt".to_string(),
            file_path: "demo.txt".into(),
            command_id: "cmd_0001".to_string(),
            kind: "Reference".to_string(),
            raw_identity: "@".to_string(),
            title: Some("Title".to_string()),
            source_trace: "demo.txt:1-2".to_string(),
            location: crate::parse::model::SourceLocation {
                source_name: "demo.txt".to_string(),
                file_path: Some("demo.txt".into()),
                start_line: 1,
                start_column: 0,
                end_line: Some(2),
                end_column: None,
            },
            content_preview: "preview".to_string(),
            parameters: vec![],
            tags: vec![],
            references: vec![],
            statuses: vec![],
        }]);

        builder.add_inserted_command_count(1);
        builder.add_database_stats(DatabaseStats {
            source_count: 1,
            command_count: 1,
            tag_count: 0,
            reference_count: 0,
        });
        builder.add_database_tables(vec![
            DatabaseTableLog {
                table_name: "sources".to_string(),
                row_count: 1,
                rows: vec![json!({"id": 1, "source_name": "demo.txt"})],
            },
            DatabaseTableLog {
                table_name: "parsed_commands".to_string(),
                row_count: 1,
                rows: vec![json!({
                    "id": 7,
                    "source_id": 1,
                    "command_id": "cmd_0001"
                })],
            },
        ]);
        builder.add_search_records(vec![SearchLogRecord {
            query: "demo".to_string(),
            hit_count: 1,
            hits: vec![json!({
                "id": 7,
                "source_name": "demo.txt",
                "command_id": "cmd_0001"
            })],
        }]);

        let document = builder.build();

        assert_eq!(document.schema.format, "cross_referenced_test_dump");
        assert_eq!(document.summary.parsed_command_count, 1);
        assert_eq!(
            document
                .indexes
                .files_by_source_name
                .get("demo.txt")
                .unwrap(),
            "file:0001"
        );
        assert_eq!(
            document.sections.parse.commands[0].db_refs,
            vec!["db:parsed_commands:7"]
        );
        assert!(!document.links.is_empty());
    }
}
