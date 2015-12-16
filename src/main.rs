extern crate galvanize;

use std::fs::File;
use galvanize::Writer;
use galvanize::Reader;
use galvanize::vec2str;


fn main() {
  let filename = "foo.cdb";
  { // This is how you write into a CDB.
    let mut f = File::create(filename).unwrap();
    let mut x = Writer::new(&mut f);

    for item in [
        ("key", "this is a value that is sligthly longer that the others"),
        ("another key", "value field"),
        ("hi", "asdf"),
      ].iter() {
      x.put(item.0, item.1);
    }
    for i in 0..128 {
      let v = &i.to_string()[..];
      x.put(v, v);
    }
    for i in 0..128 {
      let k = &i.to_string()[..];
      let v = &(128 - i).to_string()[..];
      x.put(k, v);
    }
    let k = &"25";
    x.put(k, &"asdf");
    x.put(k, &"a");
    x.put(k, &"b");

    x.finalize();
  }

  { // This is how you read from a CDB.
    let mut r = File::open(filename).unwrap();
    let mut y = Reader::new(&mut r);

    for k in ["key", "another key", "not", "long key not in the data"].iter() {
      let mut v: Vec<u8> = vec![];
      // Fetch first value for a given key.
      match y.get_first(&k, &mut v) {
        Ok(s) => println!("{:?}: {:?} (value byte length {})", k, vec2str(&v), s),
        Err(e) => println!("{:?}: {}", k, e)
      }
    }

    for i in 0..5 {
      let mut v: Vec<u8> = vec![];
      let k = "25";
      // Fetch value for a specific position under a given key.
      match y.get_from_pos(&k, &mut v, i) {
        Ok(s) => println!("{:?}: {:?} (value byte length {})", k, vec2str(&v), s),
        Err(e) => println!("{:?}: {}", k, e)
      }
    }

    println!("");

    let mut vs: Vec<Vec<u8>> = vec![];

    y.get_all(&"25", &mut vs);  // Fetch all the values under a key.

    for v in vs {
      println!("{:?}", vec2str(&v));
    }

    println!("{} items in the CDB at {}", y.len(), filename);
  }
}
