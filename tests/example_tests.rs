use std::fs;
use hbf::{compile_to_bfo, build_bf};

#[test]
fn test_all_examples() {
    let entries = fs::read_dir("examples").expect("Could not read examples directory");
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "hbf") {
                if let Some(path_str) = path.to_str() {
                    println!("Testing compilation of {}", path_str);
                    
                    let bfo_file = path_str.replace(".hbf", ".bfo");
                    let bf_file = path_str.replace(".hbf", ".bf");
                    
                    // Verify it compiles to BFO
                    compile_to_bfo(path_str, &bfo_file);
                    assert!(fs::metadata(&bfo_file).is_ok(), "BFO file should be generated for {}", path_str);
                    
                    // Verify it builds to BF
                    build_bf(&bfo_file, &bf_file);
                    assert!(fs::metadata(&bf_file).is_ok(), "BF file should be generated for {}", path_str);
                }
            }
        }
    }
}
