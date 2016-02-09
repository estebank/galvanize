extern crate docopt;
extern crate galvanize;
extern crate rustc_serialize;

use docopt::Docopt;
use galvanize::Reader;
use galvanize::vec2str;
use std::fs::File;
use std::str::from_utf8;


const USAGE: &'static str = "
Galvanize

Usage:
  galvanize FILE (top|tail)
  galvanize FILE (top|tail) COUNT
  galvanize FILE count
  galvanize FILE <keys>
  galvanize FILE <keys> COUNT
  galvanize FILE all --yes-i-am-sure
  galvanize (-h | --help)
  galvanize --version

Options:
  -h --help  Show this screen.
  --version  Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_FILE: String,
    arg_keys: Vec<String>,
    cmd_top: bool,
    cmd_tail: bool,
    arg_COUNT: u32,
    cmd_count: bool,
    cmd_all: bool,
    flag_yes_i_am_sure: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                         .and_then(|d| d.decode())
                         .unwrap_or_else(|e| e.exit());

    let filename = args.arg_FILE;
    let mut f = File::open(filename.clone()).unwrap();
    let mut cdb_reader = Reader::new(&mut f).ok().unwrap();
    let count: usize = if args.arg_COUNT == 0 {
        10
    } else {
        args.arg_COUNT as usize
    };
    if args.cmd_top {
        for item in cdb_reader.into_iter().take(count) {
            println!("{:?}: {:?}", vec2str(&item.0), vec2str(&item.1));
        }
    } else if args.cmd_tail {
        let len = cdb_reader.len();
        for item in cdb_reader.into_iter().skip(len - count) {
            println!("{:?}: {:?}", vec2str(&item.0), vec2str(&item.1));
        }
    } else if args.cmd_count {
        println!("There're {} items in the CDB at {:?}",
                 cdb_reader.len(),
                 filename);
    } else if args.cmd_all {
        for item in cdb_reader.into_iter() {
            println!("{:?}: {:?}", vec2str(&item.0), vec2str(&item.1));
        }
    } else if args.arg_keys.len() > 0 {
        for key in args.arg_keys {
            println!("Values under key {:?}", key);
            for val in cdb_reader.get(&key.into_bytes()) {
                println!("    {:?}", vec2str(&val));
            }
        }
    }
}
