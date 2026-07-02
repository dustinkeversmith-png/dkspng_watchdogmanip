// Create a tree walker module which accepts root directory or multiple root directory with inclusion and exclusion rules for traversal then add a attachment layer so that you can run custom code commands like parsing commands on the attachment

// Example would be like 
// walk (
    // recursive:true
    // exclude_directories: {"node_modules", ...} or like exclude directories with partial text in them etc
    // then can add like a lambda callback here [] ( directory.. ) => { 
        new parser
        you know parse all files etc etc
    }
)