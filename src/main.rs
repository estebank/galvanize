extern crate galvanize;

use galvanize::Reader;
use galvanize::Writer;
use galvanize::vec2str;
use std::fs::File;
use std::str::from_utf8;


fn main() {
    let filename = "foo.cdb";
    let items = [("key".as_bytes(),
                  "this is a value that is sligthly longer that the others".as_bytes()),
                 ("another key".as_bytes(), "value field".as_bytes()),
                 ("hi".as_bytes(), "hello".as_bytes())];
    {
        // This is how you write into a CDB.
        let mut f = File::create(filename).unwrap();
        let mut cdb_writer = Writer::new(&mut f).ok().unwrap();
        println!("Opening a CDB writer on file {:?}", filename);

        for item in items.clone().iter() {
            let _ = cdb_writer.put(item.0, item.1);
        }
        for i in 0..128 {
            println!("Trying to insert ([{:?}], [{:?}]): Result {:?}",
                     i,
                     i,
                     cdb_writer.put(&[i], &[i]));
        }
        for i in 0..128 {
            let v = 128 - i;
            println!("Trying to insert ([{:?}], [{:?}]): Result {:?}",
                     i,
                     v,
                     cdb_writer.put(&[i], &[v]));
        }
        let k = "25".as_bytes();
        let v = "asdf".as_bytes();
        println!("Trying to insert ({:?}, {:?}): Result {:?}",
                 k,
                 v,
                 cdb_writer.put(k, v));
        let v = "a".as_bytes();
        println!("Trying to insert ({:?}, {:?}): Result {:?}",
                 k,
                 v,
                 cdb_writer.put(k, v));
        let v = "b".as_bytes();
        println!("Trying to insert ({:?}, {:?}): Result {:?}",
                 k,
                 v,
                 cdb_writer.put(k, v));
        println!("Closing writer CDB, writing indexes to disk.\n");
    }

    {
        // This is how you read from a CDB.
        let mut f = File::open(filename).unwrap();
        let mut cdb_reader = Reader::new(&mut f).ok().unwrap();
        println!("Opening a CDB reader on file {:?}", filename);

        println!("Fetching items");
        for item in items.iter() {
            let k = item.0;
            // Fetch first value for a given key.
            match cdb_reader.get_first(k) {
                Ok(v) => println!("{:?}: {:?}", from_utf8(k).unwrap(), vec2str(&v)),
                Err(e) => println!("Failed to get {:?}: {:?}", from_utf8(k).unwrap(), e),
            }
        }

        let k = "25".as_bytes();
        println!("Fetching values under key {:?} by position:",
                 from_utf8(k).unwrap());
        for i in 0..5 {
            // Fetch value for a specific position under a given key.
            match cdb_reader.get_from_pos(k, i) {
                Ok(v) => println!("    {}: {:?}", i, vec2str(&v)),
                Err(e) => println!("    {}: Error when fetching: {:?}", i, e),
            }
        }

        let k = "25".as_bytes();
        let vs: Vec<Vec<u8>> = cdb_reader.get(k);  // Fetch all the values under a key.

        println!("Values under {:?}:", from_utf8(k).unwrap());
        for v in vs {
            println!("   {:?}", vec2str(&v));
        }

        println!("{} items in the CDB at {}", cdb_reader.len(), filename);

        for item in cdb_reader.into_iter().take(10) {
            println!("  {:?} {:?}", item.0, item.1);
        }
        println!("  {:?}", cdb_reader.keys());
        let len = cdb_reader.len();
        for item in cdb_reader.into_iter().skip(len - 10) {
            println!("  {:?} {:?}", item.0, item.1);
        }
        println!("  {:?}", cdb_reader.keys());
    }
}
