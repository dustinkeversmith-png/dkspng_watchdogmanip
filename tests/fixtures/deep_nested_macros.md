# Macro Processor Plan

Some loose thought: revisit the parser database after the context tree is stable.

@Project MacroProcessor
Title: Macro Processor OS Layer
Description: Handles parser, context, navigation, history, and watchdog engines.

    @in project/src/parser
    The parser area has local notes under it.

        @Task Build deeply nested boundary solver (building)
        Description: Make commands survive inside prose, markdown, and weird indentation.
        Content: Search should find boundary and parser words.
        See ./src/parse/passes/boundary.rs #parser #boundary

            @Q/A
            - How do we keep nested command content separate?
            Answer: until the next command boundary or outdent policy.

        Random sentence: compose a routine to refresh aliases after parse.

        @Alias parser-core ./src/parse/pipeline/mod.rs type=file context=parser line=12 marker=MacroPipeline

    ## Watch Area

    @Task Wire watchdog routine expansion
        Title: Routine expansion for docs
        Description: timer routines should reindex docs and refresh aliases.
        @Reference ./examples/watch_spec.json

@Deferred
Title: Window adapter
Content: Later add Windows focus events and Explorer locations.

@current
Finish nested parser database search tests
