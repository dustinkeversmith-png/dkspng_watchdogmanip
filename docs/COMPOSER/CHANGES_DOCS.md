# Context

1. Get rid of the embedded parser module in parser.rs
Should get a context database and test context resolution from file path or location/project identifier

Generate a test fixture expand with context specific commands, use a integration test using the parser and the navigation context to build/retrieve context details


in fs_indexer use the walk utility for building the context, need to also add like configs for building the context and rename this to build, configs like min amount of files to stop adding context layers, possible parsing for getting context specific details such as like context names or building some kind of relationship

in index the context index might need some more smart resolving system or parameter systems for locating contexts quickly changing them etc



1. Add a test for the context, resolve @current, prevs and other context dependent



# Database
need to split this up or make it more modular depending on the module so contexts will have its own tables, and like documents for like parsed commands etc etc


# History 

Add a test for history and a designated history folder for rows of events

Should move the suggestion engine to its own like module for suggesting different things

Need to migrate to a database rather than a json store

# Parse
need to move its own individual database into its /database folder for temporary storage etc and tests

Should probably move boundary and boundary resolution into its into its own file outisde of commands ince that will be important for detecting more unknown formats, 

also a parse output may need some kind hierarchical structure even in the document itself some times for example there should be a hierarchy decider when parsing like a command is meant to be a sub block of something, if its a numbered etc etc should have some kind of familial information

boundary pass should be expanded and include its types, and command, it might be useful to also add a system for multiple looks like trying different boundaries to see if different commands make sense, possibly some command seeds might expect a kind of format style or looking for correct block layouts or if the data or syntax jumbled can try different methods, possibly looking backwards with possiblility of the content being defined behind the command seed or to the right of it

in the future non linear boundary collection might be neccessary possibly searching for relevant section, or some kind of relevancy detection but deferr that for now

for detection this should really be in more a seed sub folder or specifically for seeds, use some seed detection like sub objects which might contain regex or some other detection scheme for detection, so you can append different types

instead of hard coding this seed detection and argument detection

same with the extractions and parsing between the boundaries, need some more modular approach to parsing different elements even possibly associating different extractors for like association with different command types or something

the inference section will need to be fixed later but for now if we could keep the inference out of the detection and save that for later

the registry command spec is bit too aggressive and always assumes that parameters are expected by they are really just like optional most of the time, the registration needs like more dynamic members and optional stuff



