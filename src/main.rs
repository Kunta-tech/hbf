use std::env;
use hbf::{compile_to_bfo, build_bf};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let mut input_file = None;
    let mut output_file = None;
    let mut compile_only = false;
    let mut bfo_to_bf = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" => compile_only = true,
            "-s" => bfo_to_bf = true,
            "-o" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Error: -o requires an output file");
                    return;
                }
            }
            "-h" | "--help" => {
                print_usage();
                return;
            }
            arg if !arg.starts_with('-') => {
                if input_file.is_none() {
                    input_file = Some(arg.to_string());
                } else {
                    eprintln!("Error: Multiple input files not supported yet");
                    return;
                }
            }
            arg => {
                eprintln!("Error: Unknown option {}", arg);
                print_usage();
                return;
            }
        }
        i += 1;
    }

    let input_file = match input_file {
        Some(f) => f,
        None => {
            eprintln!("Error: No input file specified");
            print_usage();
            return;
        }
    };

    if input_file.ends_with(".hbf") {
        if compile_only {
            let bfo_out = output_file.unwrap_or_else(|| input_file.replace(".hbf", ".bfo"));
            compile_to_bfo(&input_file, &bfo_out);
        } else if bfo_to_bf {
            // This is a bit redundant but explicitly requested for the flag
            let bfo_out = input_file.replace(".hbf", ".bfo"); 
            let bf_out = output_file.unwrap_or_else(|| input_file.replace(".hbf", ".bf"));
            compile_to_bfo(&input_file, &bfo_out);
            build_bf(&bfo_out, &bf_out);
        } else {
            // Full pipeline (default)
            let bfo_out = input_file.replace(".hbf", ".bfo"); 
            let bf_out = output_file.unwrap_or_else(|| input_file.replace(".hbf", ".bf"));
            compile_to_bfo(&input_file, &bfo_out);
            build_bf(&bfo_out, &bf_out);
        }
    } else if input_file.ends_with(".bfo") {
        if compile_only {
            eprintln!("Warning: -c has no effect when input is already .bfo");
        }
        let bf_out = output_file.unwrap_or_else(|| input_file.replace(".bfo", ".bf"));
        build_bf(&input_file, &bf_out);
    } else {
        eprintln!("Error: Unsupported file extension. Use .hbf or .bfo");
    }
}

fn print_usage() {
    eprintln!("Usage: hbf [options] <file>");
    eprintln!("Options:");
    eprintln!("  -c              Compile HBF to BFO only");
    eprintln!("  -s              Compile BFO to BF only");
    eprintln!("  -o <file>       Specify output filename");
    eprintln!("  -h, --help      Display this help message");
}
