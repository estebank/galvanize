//! This module allows you to read from a CDB.
use helpers::hash;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use types::{Error, Result};
use writer::Writer;

/// Allows you to read from CDB.
///
/// #Example
///
/// Given a file stored at `filename` with the following contents:
///
/// ```text
/// {
///     "key": "value",
/// }
/// ```
///
/// this is how you can read the stored value:
///
/// ```
/// # use galvanize::Result;
/// # use galvanize::Writer;
/// use galvanize::Reader;
/// use std::fs::File;
///
/// # // Doing this to get around the fact that you can't have `try!` in `main`.
/// # fn main() {
/// #     let _ = do_try();
/// # }
/// #
/// # fn do_try() -> Result<()> {
/// # let filename = "reader_example.cdb";
/// let key = "key".as_bytes();
/// # {
/// #     let mut f = File::create(filename)?;
/// #     let mut cdb_writer = Writer::new(&mut f)?;
/// #     cdb_writer.put(key, "value".as_bytes());
/// # }
///
/// let mut f = File::open(filename)?;
/// let mut cdb_reader = Reader::new(&mut f)?;
/// let stored_vals = cdb_reader.get(key);
/// assert_eq!(stored_vals.len(), 1);
/// assert_eq!(&stored_vals[0][..], &"value".as_bytes()[..]);
///
/// // The CDB contains only one entry:
/// assert_eq!(cdb_reader.len(), 1);
///
/// // Accessing a key that isn't in the CDB:
/// let non_existing_key = "non_existing_key".as_bytes();
/// let empty = cdb_reader.get(non_existing_key);
/// assert_eq!(empty.len(), 0);
///
/// assert!(cdb_reader.get_first(non_existing_key).is_err());
/// #
/// #     Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Reader<'a, F: Read + Seek + 'a> {
    /// Opened file to read values from.
    file: &'a mut F,
    /// Index for the contents of the CDB.
    index: Vec<(u32, u32)>,
    /// Position in the file where the hash table starts.
    table_start: usize,
    /// How many elements are there in the CDB.
    length: usize,
}

/// Iterator struct for Key, Values in a CDB.
pub struct ItemIterator<'a, 'file: 'a, F: Read + Seek + 'file> {
    reader: &'a mut Reader<'file, F>,
}

/// Iterate over (Key, Values) in a CDB until the end of file.
impl<'a, 'file: 'a, F: Read + Seek + 'file> Iterator for ItemIterator<'a, 'file, F> {
    /// A single `key`, `value` pair.
    type Item = (Vec<u8>, Vec<u8>);

    /// Fetch the next (`key`, `value`) pair, if any.
    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.file.seek(SeekFrom::Current(0)) {
            Ok(pos) => {
                if pos >= self.reader.table_start as u64 {
                    return None;
                }
            }
            Err(_) => return None,
        }
        // We're in the Footer/Hash Table of the file, no more items.
        let mut buf: [u8; 8] = [0; 8];
        {
            let mut chunk = self.reader.file.take(8);
            let _ = chunk.read(&mut buf);
        }
        let k = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]); // Key length
        let v = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]); // Value length

        let mut key: Vec<u8> = vec![];
        {
            let mut chunk = self.reader.file.take(k as u64);
            let _ = chunk.read_to_end(&mut key);
        }

        let mut val: Vec<u8> = vec![];
        {
            let mut chunk = self.reader.file.take(v as u64);
            let _ = chunk.read_to_end(&mut val);
        }

        Some((key, val))
    }
}

/// Convert a [`Reader`]() CDB into an `Iterator`.
///
/// One use of this, is using Rust's `for` loop syntax.
/// #Example
/// ```
/// # use galvanize::Reader;
/// # use std::fs::File;
/// # let filename = "tests/testdata/top250pws.cdb";
/// let mut f = File::open(filename).unwrap();
///
/// let mut cdb_reader = Reader::new(&mut f).ok().unwrap();
/// let len = cdb_reader.len();
///
/// # let mut i = 0;
/// for (k, v) in cdb_reader.into_iter() {
///     // Consume the (k, v) pair.
/// #    let _ = k;
/// #    i += 1;
/// #    let s = &i.to_string();
/// #    let val = s.as_bytes();
/// #    assert_eq!(&v[..], &val[..]);
/// }
/// # assert_eq!(len, i);
/// #
/// # // Do it again to make sure the iterator doesn't consume and lifetimes
/// # // work as expected.
/// # i = 0;
/// # for (_, v) in cdb_reader.into_iter() {
/// #     i += 1;
/// #     let s = &i.to_string();
/// #     let val = s.as_bytes();
/// #     assert_eq!(&v[..], &val[..]);
/// # }
/// # assert_eq!(len, i);
/// ```
impl<'a, 'file: 'a, F: Read + Seek + 'file> IntoIterator for &'a mut Reader<'file, F> {
    /// A single `key`, `value` pair.
    type Item = (Vec<u8>, Vec<u8>);

    /// The [`ItemIterator`](struct.ItemIterator.html) type this will convert
    /// into.
    type IntoIter = ItemIterator<'a, 'file, F>;

    fn into_iter(self) -> Self::IntoIter {
        let _ = self.file.seek(SeekFrom::Start(2048));
        ItemIterator { reader: self }
    }
}

impl<'a, F: Read + Seek + 'a> Reader<'a, F> {
    /// Creates a new `Reader` consuming the provided `file`.
    pub fn new(file: &'a mut F) -> Result<Reader<'a, F>> {
        match file.seek(SeekFrom::End(0)) {
            Err(e) => return Err(Error::IOError(e)),
            Ok(n) => {
                if n < 2048 {
                    return Err(Error::CDBTooSmall);
                }
            }
        };

        // Using u32 instead of usize as standard CDBs can only be 4GB in size.
        let mut index: Vec<(u32, u32)> = vec![];
        let mut sum: u32 = 0;

        let mut buf: Vec<u8> = vec![];
        {
            file.seek(SeekFrom::Start(0))?;
            let mut chunk = file.take(2048);
            chunk.read_to_end(&mut buf)?;
        }

        for ix in 0..2048 / 8 {
            let i = ix * 8;
            let k = u32::from_le_bytes([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]);
            let v = u32::from_le_bytes([buf[i + 4], buf[i + 5], buf[i + 6], buf[i + 7]]);
            sum += v >> 1;
            index.push((k, v));
        }
        let table_start = index.iter().map(|item| item.0).min().unwrap();

        Ok(Reader {
            file,
            index,
            table_start: table_start as usize,
            length: sum as usize,
        })
    }

    /// How many `(key, value)` pairs are there in this Read Only CDB.
    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return a `Vec` of all the values under the given `key`.
    pub fn get(&mut self, key: &[u8]) -> Vec<Vec<u8>> {
        let mut i = 0;
        let mut values: Vec<Vec<u8>> = vec![];
        while let Ok(v) = self.get_from_pos(key, i) {
            values.push(v);
            i += 1;
        }
        values
    }

    /// Return a `Vec` of all the keys in this Read Only CDB.
    ///
    /// Keep in mind that if there're duplicated keys, they will appear
    /// multiple times in the resulting `Vec`.
    pub fn keys(&mut self) -> Vec<Vec<u8>> {
        let mut keys: Vec<Vec<u8>> = vec![];
        for item in self.into_iter() {
            keys.push(item.0);
        }
        keys
    }

    /// Pull the `value` bytes for the first occurence of the given `key` in
    /// this CDB.
    pub fn get_first(&mut self, key: &[u8]) -> Result<Vec<u8>> {
        self.get_from_pos(key, 0)
    }

    /// Pull the `value` bytes for the `index`st occurence of the given `key`
    /// in this CDB.
    pub fn get_from_pos(&mut self, key: &[u8], index: u32) -> Result<Vec<u8>> {
        let h = hash(key);
        let (start, nslots) = self.index[(h & 0xff) as usize];

        if nslots > index {
            // Bucket has keys.
            let end = start + (nslots << 3);
            let slot_off = start + (((h >> 8) % nslots) << 3);

            let mut counter = 0;
            // Every 8 bytes from the slot offset to the end, and then from the
            // end to the slot_offset.
            for pos in (slot_off..end)
                .chain(start..slot_off)
                .enumerate()
                .filter(|item| item.0 % 8 == 0)
                .map(|item| item.1)
            {
                let mut buf: [u8; 8] = [0; 8];
                {
                    self.file.seek(SeekFrom::Start(pos as u64))?;
                    let mut chunk = self.file.take(8);
                    chunk.read_exact(&mut buf)?;
                }
                let rec_h = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let rec_pos = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);

                if rec_h == 0 {
                    // Key not in file.
                    return Err(Error::KeyNotInCDB);
                } else if rec_h == h {
                    // Hash of key found in file.
                    {
                        self.file.seek(SeekFrom::Start(rec_pos as u64))?;
                        let mut chunk = self.file.take(8);
                        chunk.read_exact(&mut buf)?;
                    }
                    let klen = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                    let dlen = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);

                    let mut buf: Vec<u8> = vec![];
                    {
                        let mut chunk = self.file.take(klen as u64);
                        chunk.read_to_end(&mut buf)?;
                    }
                    {
                        if buf == key {
                            // Found key in file
                            buf.clear();

                            let mut chunk = self.file.take(dlen as u64);
                            chunk.read_to_end(&mut buf)?;

                            if counter == index {
                                return Ok(buf);
                            }
                            counter += 1;
                        }
                    }
                }
            }
        }
        Err(Error::KeyNotInCDB)
    }
}

// Needs to be a file to `truncate` at the end.
impl<'a> Reader<'a, File> {
    /// Transform this `Reader` into a `Writer` using the same underlying
    /// `file`.
    ///
    /// The underlying file will have its hash table `truncate`d. This will be
    /// regenerated on `Writer` drop.
    pub fn as_writer(self) -> Result<Writer<'a, File>> {
        match self.file.seek(SeekFrom::Start(self.table_start as u64)) {
            Ok(_) => {
                let mut index: Vec<Vec<(u32, u32)>> = vec![Vec::new(); 256];

                let buf = &mut [0_u8; 8];
                // Read hash table until end of file to recreate Writer index.
                while let Ok(s) = self.file.read(buf) {
                    if s == 0 {
                        // EOF
                        break;
                    }
                    let h = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                    let pos = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
                    index[(h & 0xff) as usize].push((h, pos));
                }

                // Clear the hash table at the end of the file. It'll be
                // recreated on `Drop` of the `Writer`.
                match self.file.set_len(self.table_start as u64) {
                    Ok(_) => (),
                    Err(e) => return Err(Error::IOError(e)),
                }
                Writer::new_with_index(self.file, index)
            }
            Err(e) => Err(Error::IOError(e)),
        }
    }
}
