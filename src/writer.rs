//! This module allows you to write to a CDB.
use helpers::{hash, pack};
use reader::Reader;
use std::io::{Read, Seek, SeekFrom, Write};
use types::Result;


/// Allows you to create a (or append to) CDB.
///
/// #Example
///
/// ```
/// # use galvanize::Result;
/// use galvanize::Writer;
/// use std::fs::File;
///
/// # // Doing this to get around the fact that you can't have `try!` in `main`.
/// # fn main() {
/// #     let _ = do_try();
/// # }
/// #
/// # fn do_try() -> Result<()> {
/// # let filename = "writer_example.cdb";
/// #
/// let mut f = File::create(filename)?;
/// let mut cdb_writer = Writer::new(&mut f)?;
/// let key = "key".as_bytes();
/// let value = "value".as_bytes();
/// cdb_writer.put(key, value)?;
///
/// // Write out the hash table from the `Writer` and transform into a `Reader`
/// let mut cdb_reader = cdb_writer.as_reader()?;
/// let stored_vals = cdb_reader.get(key);
/// assert_eq!(stored_vals.len(), 1);
/// assert_eq!(&stored_vals[0][..], &value[..]);  // "value".as_bytes()
/// #
/// #     Ok(())
/// # }
/// ```
pub struct Writer<'a, F: Write + Read + Seek + 'a> {
    /// Opened file to write values into.
    file: &'a mut F,
    /// Working hash table for the contents of the CDB.
    index: Vec<Vec<(u32, u32)>>,
}

impl<'a, F: Write + Read + Seek + 'a> Writer<'a, F> {
    /// Creates a new `Reader` consuming the provided `file`.
    ///
    /// The `file` must allow writes to be performed.
    pub fn new(file: &'a mut F) -> Result<Writer<'a, F>> {
        file.seek(SeekFrom::Start(0))?;
        file.write(&[0; 2048])?;

        Self::new_with_index(file, vec![Vec::new(); 256])
    }

    /// Used by `Reader::as_writer` method, to prepopulate the index from the
    /// underlying `file`.
    pub fn new_with_index(file: &'a mut F, index: Vec<Vec<(u32, u32)>>) -> Result<Writer<'a, F>> {
        Ok(Writer {
            file: file,
            index: index,
        })
    }

    /// Write `value` for `key` into this CDB.
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let pos = self.file.seek(SeekFrom::Current(0))? as u32;
        self.file.write(&pack(key.len() as u32))?;
        self.file.write(&pack(value.len() as u32))?;

        self.file.write(key)?;
        self.file.write(value)?;

        let h = hash(key) & 0xffffffff;
        self.index[(h & 0xff) as usize].push((h, pos));
        Ok(())
    }

    /// Write out the hash table to the `file` footer.
    fn finalize(&mut self) {
        let mut index: Vec<(u32, u32)> = Vec::new();

        &self.file.seek(SeekFrom::End(0));
        for tbl in &self.index {
            let length = (tbl.len() << 1) as u32;
            let mut ordered: Vec<(u32, u32)> = vec!((0, 0); length as usize);
            for &pair in tbl {
                let where_ = (pair.0 >> 8) % length;
                for i in (where_..length).chain(0..where_) {
                    if ordered[i as usize].0 == 0 {
                        ordered[i as usize] = pair;
                        break;
                    }
                }
            }
            index.push((self.file.seek(SeekFrom::End(0)).unwrap() as u32, length));
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
    }

    /// Transform this `Writer` into a `Reader` using the same underlying
    /// `file`.
    ///
    /// The `Writer` will flush the hash table to the underlying `file`.
    pub fn as_reader(mut self) -> Result<Reader<'a, F>> {
        {
            let s = &mut self;
            s.finalize();
        }
        Reader::new(self.file)
    }
}

impl<'a, F: Write + Read + Seek + 'a> Drop for Writer<'a, F> {
    /// Write out the hash table footer for this CDB.
    fn drop(&mut self) {
        self.finalize();
    }
}
