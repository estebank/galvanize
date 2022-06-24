extern crate galvanize;

use galvanize::Reader;
use galvanize::Writer;
use galvanize::helpers::hash;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Path;


#[test]
fn known_good_djb_hash() {
  assert_eq!(hash("dave".as_bytes()), 2087378131);
}

#[test]
fn djb_correct_wrapping() {
  assert_eq!(hash("davedavedavedavedave".as_bytes()), 3529598163);
}

fn make_writer<'a, F: Write + Read + Seek>(file: &'a mut F, items: &[(&[u8], &[u8])]) -> Writer<'a, F> {
    // This is how you write into a CDB.
    let mut cdb_writer = Writer::new(file).ok().unwrap();

    for item in items.iter() {
        // Inserting returns a success or error.
        let _ = cdb_writer.put(item.0, item.1);
    }
    cdb_writer
}

#[test]
fn create_file() {
    let filename = "new_file.cdb";
    let items = [("key".as_bytes(),
                  "this is a value that is sligthly longer that the others".as_bytes()),
                 ("another key".as_bytes(), "value field".as_bytes()),
                 ("hi".as_bytes(), "asdf".as_bytes())];
    {
        let mut f = File::create(filename).unwrap();
        let mut cdb_writer = make_writer(&mut f, &items);
        for i in 0..128 {
            let _ = cdb_writer.put(&[i], &[i]);
        }
        for i in 0..128 {
            let _ = cdb_writer.put(&[i], &[128 - i]);
        }
        let k = "25".as_bytes();
        let _ = cdb_writer.put(k, "a".as_bytes());
        let _ = cdb_writer.put(k, "b".as_bytes());
        // The CDB file get's automatically flushed to disk on scope end.
    }

    {
        // This is how you read from a CDB.
        let mut f = File::open(filename).unwrap();
        let mut cdb_reader = Reader::new(&mut f).ok().unwrap();

        for item in items.iter() {
            // Fetch first value for a given key.
            let (k, v) = *item;
            match cdb_reader.get_first(k) {
                Ok(val) => assert_eq!(v, &val[..]),
                Err(e) => panic!("{:?} {:?} {:?}", k, v, e),
            }
        }

        for i in 0..128 {
            // Fetch value for a specific position under a given key.
            match cdb_reader.get_from_pos(&[i], 0) {
                Ok(v) => assert_eq!(&[i], &v[..]),
                Err(e) => panic!("Error reading first value from key {:?}: {:?}", i, e),
            }
            match cdb_reader.get_from_pos(&[i], 1) {
                Ok(v) => assert_eq!(&[128 -i], &v[..]),
                Err(e) => panic!("Error reading second value from key {:?}: {:?}", i, e),
            }
        }

        assert_eq!(cdb_reader.get("25".as_bytes()), vec!["a".as_bytes(), "b".as_bytes()]);
        assert_eq!(cdb_reader.len(), 261);
    }
}

#[test]
fn read_from_top_250_passwords_file() {
    // This is how you read from a CDB.
    let filename = "tests/testdata/top250pws.cdb";
    let mut f = File::open(filename).unwrap();

    let mut cdb_reader = Reader::new(&mut f).ok().unwrap();

    assert_eq!(cdb_reader.get("letmein".as_bytes()), vec!["10".as_bytes()]);
    assert_eq!(cdb_reader.len(), 250);
    assert_eq!(cdb_reader.len(), cdb_reader.into_iter().count());
}

#[test]
fn read_from_passwords_dump_file() {
    // This is how you read from a CDB.
    let filename = "tests/testdata/pwdump.cdb";
    let mut f = File::open(filename).unwrap();

    let mut cdb_reader = Reader::new(&mut f).ok().unwrap();

    assert_eq!(cdb_reader.get("f7396427246008f9d580c9a666000976".as_bytes()),
               vec!["defton".as_bytes(),
                    "deftones".as_bytes(),
                    "DEFTONES".as_bytes(),
                    ]);
    assert_eq!(cdb_reader.len(), 3000);
}

#[test]
fn keys() {
    // Use of (key, value) iterator on a CDB Reader.
    let filename = "tests/testdata/top250pws.cdb";
    let mut f = File::open(filename).unwrap();

    let mut cdb_reader = Reader::new(&mut f).ok().unwrap();
    let len = cdb_reader.len();

    assert_eq!(len, cdb_reader.keys().len());
    // Do it again to make sure the iterator doesn't consume and lifetimes work
    // as expected.
    assert_eq!(len, cdb_reader.keys().len());
}

#[test]
fn turn_writer_into_reader() {
    let filename = "writer_into_reader.cdb";
    let items = [("key".as_bytes(),
                  "this is a value that is sligthly longer that the others".as_bytes()),
                 ("another key".as_bytes(), "value field".as_bytes()),
                 ("hi".as_bytes(), "asdf".as_bytes())];
    let path = Path::new(filename);
    {
        let _ = File::create(path);
    }
    let mut options = OpenOptions::new();
    options.write(true).read(true);

    let mut f = options.open(path).unwrap();
    let cdb_writer = make_writer(&mut f, &items);
    let mut cdb_reader = cdb_writer.as_reader().unwrap();
    for item in items.iter() {
        // Fetch first value for a given key.
        let (k, v) = *item;
        match cdb_reader.get_first(k) {
            Ok(val) => assert_eq!(v, &val[..]),
            Err(e) => panic!("{:?} {:?} {:?}", k, v, e),
        }
    }
}

#[test]
fn turn_reader_into_writer() {
    let filename = "reader_into_writer.cdb";
    let items = [("key".as_bytes(), "this is a value that is sligthly longer that the others".as_bytes()),
                 ("another key".as_bytes(), "value field".as_bytes()),
                 ("hi".as_bytes(), "asdf".as_bytes())];
    let k1 = "key".as_bytes();
    let k2 = "new key".as_bytes();
    let v1 = "different value".as_bytes();
    let v2 = "a value".as_bytes();

    let path = Path::new(filename);
    {
        let mut f = File::create(filename).unwrap();
        let _ = make_writer(&mut f, &items);
    }
    {
        let mut options = OpenOptions::new();
        options.write(true).read(true);

        let mut f = options.open(path).unwrap();
        let mut cdb_reader = Reader::new(&mut f).unwrap();
        // Turn into iteartor to confirm that the reader works as expected.
        for item in items.iter() {
            // Fetch first value for a given key.
            let (k, v) = *item;
            match cdb_reader.get_first(k) {
                Ok(val) => assert_eq!(v, &val[..]),
                Err(e) => panic!("{:?} {:?} {:?}", k, v, e),
            }
        }

        // Turn into writer
        let mut cdb_writer = cdb_reader.as_writer().unwrap();
        // Add a new value under an existing key
        let _ = cdb_writer.put(k1, v1);
        // Add a new value under a new key
        let _ = cdb_writer.put(k2, v2);
    }
    {
        // Read from the CDB to confirm that both the original data and the
        // data that was appended are there.
        let mut f = File::open(filename).unwrap();
        let mut cdb_reader = Reader::new(&mut f).ok().unwrap();
        for item in items.iter() {
            // Fetch first value for a given key.
            let (k, v) = *item;
            match cdb_reader.get_first(k) {
                Ok(val) => assert_eq!(v, &val[..]),
                Err(e) => panic!("{:?} {:?} {:?}", k, v, e),
            }
        }
        assert_eq!(v1, &cdb_reader.get_from_pos(k1, 1).unwrap()[..]);
        assert_eq!(v2, &cdb_reader.get_from_pos(k2, 0).unwrap()[..]);
    }
}
