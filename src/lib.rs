use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::num::Wrapping;


/// DJB hash function
pub fn hash(string: &str) -> u32 {
  let mut h: Wrapping<u32> = Wrapping(5381);
  for c in string.as_bytes().iter() {
    let x: Wrapping<u32> = Wrapping(c.to_owned() as u32);
    h = (((h << 5) + h) ^ x) & Wrapping(0xffffffff);
  }
  h.0
}


#[inline]
pub fn pack(v: u32) -> [u8; 4] {
  [
    v as u8,
    (v >> 8) as u8,
    (v >> 16) as u8,
    (v >> 24) as u8,
  ]
}


#[inline]
pub fn unpack(v: [u8; 4]) -> u32 {
  (
    (v[0] as u32) |
    ((v[1] as u32) << 8) |
    ((v[2] as u32) << 16) |
    ((v[3] as u32) << 24)
  )
}


/// CDB Reader struct
pub struct Reader<'a> {
  // Opened file to read values from.
  data: &'a mut File,
  // Index for the contents of the CDB.
  index: Vec<(u32, u32)>,
  // Position in the file where the index table starts.
  table_start: usize,
  // How many elements are there in the CDB.
  length: usize,
}

/// CDB Writer struct
pub struct Writer<'a> {
  // Opened file to write values into.
  file: &'a mut File,
  // Working index for the contents of the CDB.
  index: Vec<Vec<(u32, u32)>>,
}


impl<'a> Reader<'a> {
  pub fn new(file: &'a mut File) -> Reader<'a> {
    if file.seek(SeekFrom::End(0)).unwrap() < 2048 {
      panic!("CDB too small");
    }

    let mut index: Vec<(u32, u32)> = vec![];
    let mut table_start: u32 = 0;
    let mut sum: u32 = 0;
    {
      file.seek(SeekFrom::Start(0));
      let mut chunk = file.take(2048);
      let mut buf: Vec<u8> = vec![];
      chunk.read_to_end(&mut buf);

      for ix in 0..2048/8 {
        let i = ix * 8;
        let k = unpack([buf[i], buf[i+1], buf[i+2], buf[i+3]]);
        let v = unpack([buf[i+4], buf[i+5], buf[i+6], buf[i+7]]);
        sum = sum + (v >> 1);
        index.push((k, v));
      }
      table_start = index.iter().map(|item| item.0 ).min().unwrap();
    }

    Reader {
      data: file,
      index: index,
      table_start: table_start as usize,
      length: sum as usize,
    }
  }

  pub fn len(&self) -> usize {
    self.length
  }

  pub fn get_all(&mut self, key: &str, values: &mut Vec<Vec<u8>>) {
    let mut i = 0;
    loop {
      let mut value: Vec<u8> = vec![];
      match self.get_from_pos(key, &mut value, i) {
        Ok(v) => values.push(value),
        Err(e) => break,
      }
      i+=1;
    }
  }

  /// Pull the `value` bytes for the first occurence of the given `key` in this CDB.
  pub fn get_first(&mut self, key: &str, value: &mut Vec<u8>) -> Result<usize, &str> {
    self.get_from_pos(key, value, 0)
  }

  /// Pull the `value` bytes for the `index`st occurence of the given `key` in this CDB.
  pub fn get_from_pos(&mut self, key: &str, value: &mut Vec<u8>, index: u32) -> Result<usize, &str> {
    // Make sure the buffer is empty before writing.
    value.clear(); 
    let mut ret: Result<usize, &str> = Err::<usize, &str>("key not in CDB");

    // Truncate to 32 bits and remove sign.
    let h = hash(key) & 0xffffffff;
    let (start, nslots) = self.index[(h & 0xff) as usize];

    if nslots > index {  // Bucket has keys.
      let end = start + (nslots << 3);
      let slot_off = start + (((h >> 8) % nslots) << 3);

      let mut iterator = (slot_off..end).chain(start..slot_off);
      let mut counter = 0;
      loop {
        value.clear();
        let pos_option = iterator.next();
        if pos_option == None {
          ret = Err::<usize, &str>("key not in CDB");
          break;
        }
        let pos = pos_option.unwrap();

        let mut buf: [u8; 4] = [0; 4];
        {
          self.data.seek(SeekFrom::Start((pos) as u64));
          let mut chunk = self.data.take(4);
          chunk.read(&mut buf);
        }
        let rec_h = unpack(buf);

        {
          let mut chunk = self.data.take(4);
          chunk.read(&mut buf);
        }
        let mut rec_pos = unpack(buf);
        if rec_h == 0 {  // Key not in file.
          ret = Err::<usize, &str>("key not in CDB");
          break;
        } else if rec_h == h {  // Hash of key found in file.
          let mut buf: [u8; 4] = [0; 4];

          {
            self.data.seek(SeekFrom::Start((rec_pos) as u64));
            let mut chunk = self.data.take(4);
            chunk.read(&mut buf);
          }
          let klen = unpack(buf);  // Key length

          {
            let mut chunk = self.data.take(4);
            chunk.read(&mut buf);
          }
          let dlen = unpack(buf);  // Value length

          rec_pos = rec_pos + 8;  // Start of the key.

          let mut buf: Vec<u8> = vec![];
          {
            self.data.seek(SeekFrom::Start((rec_pos) as u64));
            let mut chunk = self.data.take(klen as u64);
            chunk.read_to_end(&mut buf);
          }
          {
            let k = std::str::from_utf8(&buf[..]).unwrap();
            if k == key {  // Found key in file
              rec_pos = rec_pos + klen;

              self.data.seek(SeekFrom::Start((rec_pos) as u64));
              let mut chunk = self.data.take(dlen as u64);
              chunk.read_to_end(value);

              if counter == index {
                ret = Ok(dlen as usize);
                break;
              }
              counter = counter + 1;
            }
          }
        }

        for _ in 0..7 {  // Jump to end of 8 bytes.
          iterator.next();
        }
      }
    }
    ret
  }
}


impl<'a> Writer<'a> {

  pub fn new(file: &'a mut File) -> Writer {
    file.seek(SeekFrom::Start(0));
    file.write(&[0; 2048]);

    Writer {
      file: file,
      index: vec!(Vec::new(); 256),
    }
  }

  /// Write `value` for `key` into this CDB.
  pub fn put(&mut self, key: &str, value: &str) {
    let pos = self.file.seek(SeekFrom::Current(0)).unwrap() as u32;

    self.file.write(&pack(key.len() as u32));
    self.file.write(&pack(value.len() as u32));

    self.file.write(key.as_bytes());
    self.file.write(value.as_bytes());

    let h = hash(key) & 0xffffffff;
    self.index[(h & 0xff) as usize].push((h, pos))
  }


  /// Write out the index for this CDB.
  pub fn finalize(&mut self) {
    let mut index: Vec<(u32, u32)> = Vec::new();

    for tbl in &self.index {
      let length = (tbl.len() << 1) as u32;
      let mut ordered: Vec<(u32, u32)> = vec!((0, 0); length as usize);
      for &pair in tbl {
        let where_ = (pair.0 >> 8) % length;
        for i in (where_..length).chain(0..where_) {
          if ordered[i as usize].0 == 0 {
            ordered[i as usize] = pair;
            //println!("{:?}", pair);
            break;
          }
        }
      }
      index.push((self.file.seek(SeekFrom::Current(0)).unwrap() as u32,
                  length
                  ));
      for pair in ordered {
        &self.file.write(&pack(pair.0));
        &self.file.write(&pack(pair.1));
      }
    }

    &self.file.seek(SeekFrom::Start(0));
    for pair in index {
        &self.file.write(&pack(pair.0));
        &self.file.write(&pack(pair.1));
    }

//        self.fp = None # prevent double finalize()
  }
}


pub fn vec2str<'a>(v: &'a Vec<u8>) -> &'a str {
  std::str::from_utf8(&v[..]).unwrap()
}


#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::File;

  #[test]
  fn known_good_djb_hash() {
    assert_eq!(hash(&"dave"), 2087378131);
  }

  #[test]
  fn djb_correct_wrapping() {
    assert_eq!(hash(&"davedavedavedavedave"), 3529598163);
  }

  #[test]
  fn full_roundtrip() {
    let filename = "foo.cdb";
    let kv = [
      ("key", "this is a value that is sligthly longer that the others"),
      ("another key", "value field"),
      ("hi", "asdf"),
    ];
    let repeat_k = "25";
    let repeat_values = ["a", "b", "c", "d"];

    { // This is how you write into a CDB.
      let mut f = File::create(filename).unwrap();
      let mut x = Writer::new(&mut f);

      for item in kv.iter() {
        x.put(item.0, item.1);
      }

      let k = &repeat_k;
      for v in repeat_values.iter() {
        x.put(k, &v);
      }

      x.finalize();
    }

    { // This is how you read from a CDB.
      let mut r = File::open(filename).unwrap();
      let mut y = Reader::new(&mut r);

      for item in kv.iter() {
        let mut v: Vec<u8> = vec![];
        let k = item.0;
        // Fetch first value for a given key.
        match y.get_first(&k, &mut v) {
          Ok(s) => assert_eq!(vec2str(&v).len(), s),
          Err(e) => assert!(true)
        }
        assert_eq!(item.1, vec2str(&v));
      }

      for i in 0..repeat_values.len() {
        let mut v: Vec<u8> = vec![];
        let k = repeat_k;
        // Fetch value for a specific position under a given key.
        match y.get_from_pos(&k, &mut v, i as u32) {
          Ok(s) => assert_eq!(repeat_values[i], vec2str(&v)),
          Err(e) => assert!(true)
        }
      }

      let mut vs: Vec<Vec<u8>> = vec![];
      y.get_all(&"25", &mut vs);  // Fetch all the values under a key.

      assert_eq!(repeat_values.len(), vs.len());
      for i in 0..vs.len() {
        let v = &vs[i];
        assert_eq!(repeat_values[i], vec2str(&v));
      }

      assert_eq!(7, y.len());

      { // Fetch non existing key.
        let mut v: Vec<u8> = vec![];
        let k = "non existing key";
        match y.get_first(&k, &mut v) {
          Ok(s) => assert!(true),
          Err(e) => assert_eq!("key not in CDB", e),
        }
      }
    }
  }
}
