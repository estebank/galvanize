#[cfg(feature = "cli")]
extern crate docopt;
extern crate galvanize;
#[cfg(feature = "cli")]
extern crate rustc_serialize;

#[cfg(feature = "cli")]
mod cli {
    use docopt::Docopt;
    use galvanize::helpers::vec2str;
    use galvanize::Reader;
    use std::cmp::min;
    use std::env;
    use std::fs::File;
    use std::process;

    const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

    #[derive(Debug, RustcDecodable)]
    #[allow(non_snake_case)]
    struct Args {
        arg_FILE: String,
        cmd_get: bool,
        arg_key: String,
        cmd_top: bool,
        cmd_tail: bool,
        arg_COUNT: u32,
        cmd_count: bool,
        cmd_all: bool,
        flag_version: bool,
    }

    fn display_items(item: (Vec<u8>, Vec<u8>)) {
        println!("{:?}: {:?}", vec2str(&item.0), vec2str(&item.1));
    }

    pub fn main() {
        let bin = match env::current_exe() {
            Ok(bin) => bin.file_name().map(|f| f.to_string_lossy().into_owned()),
            _ => None,
        }
        .unwrap_or_else(|| "cli".to_owned());

        let args: Args = Docopt::new(format!(
            "
             {0:}

             Usage:
               {0:} FILE (top|tail)
               {0:} FILE (top|tail) COUNT
               {0:} FILE count
               {0:} FILE get <key>
               {0:} FILE get -e <key>
               {0:} FILE all --yes-i-am-sure
               {0:} (-h | --help)
               {0:} --version

             Options:
               -h --help      Show this screen.
               --version      Show version.
               -e, --encoded  Treat the key as encoded.
             ",
            bin
        ))
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

        if args.flag_version {
            println!("galvanize {}", VERSION.unwrap_or("unknown"));
            process::exit(0);
        }

        let filename = args.arg_FILE;
        let mut f = match File::open(filename.clone()) {
            Ok(f) => f,
            Err(e) => {
                println!("Could not open file {:?}: {:?}", filename, e);
                process::exit(1);
            }
        };
        let mut cdb_reader = match Reader::new(&mut f) {
            Ok(f) => f,
            Err(e) => {
                println!("Could not use {:?} as a readonly CDB: {:?}", filename, e);
                process::exit(1);
            }
        };

        let count: usize = if args.arg_COUNT == 0 {
            10
        } else {
            args.arg_COUNT as usize
        };

        if args.cmd_all {
            // Show all (key, value) pairs.
            for item in cdb_reader.into_iter() {
                display_items(item);
            }
        } else if args.cmd_top {
            // Show COUNT first (key, value) pairs.
            for item in cdb_reader.into_iter().take(count) {
                display_items(item);
            }
        } else if args.cmd_tail {
            // Show COUNT last (key, value) pairs.
            let len = cdb_reader.len();
            for item in cdb_reader.into_iter().skip(len - min(len, count)) {
                display_items(item);
            }
        } else if args.cmd_count {
            // How many (key, value) are there in this file?
            println!(
                "There are {} items in the CDB at {:?}",
                cdb_reader.len(),
                filename
            );
        } else if args.cmd_get {
            // Get all values under a single key.
            let key = args.arg_key;
            let values = cdb_reader.get(&key.clone().into_bytes());
            if values.is_empty() {
                println!("There're no values under {:?}", key);
            } else if values.len() == 1 {
                println!("{:?}: {:?}", key, vec2str(&values[0]));
            } else {
                println!("Values under key {:?}", key);
                for val in values {
                    println!("    {:?}", vec2str(&val));
                }
            }
        }
    }
}

#[cfg(feature = "cli")]
fn main() {
    cli::main();
}

#[cfg(not(feature = "cli"))]
fn main() {
    println!("In order to run this tool, please enable `cli` feature:");
    println!("\n    cargo run --features=cli --example cli -- --help\n");
}
