use helpers::hash;
use helpers::pack;
use helpers::unpack;
use helpers::vec2str;
use reader::Reader;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use types::Error;
use types::Result;


/// CDB Writer struct
pub struct Writer<'a, F: Write + Read + Seek + 'a> {
    // Opened file to write values into.
    file: &'a mut F,
    // Working index for the contents of the CDB.
    index: Vec<Vec<(u32, u32)>>,
}

impl<'a, F: Write + Read + Seek + 'a> Writer<'a, F> {
    pub fn new(file: &'a mut F) -> Result<Writer<'a, F>> {
        try!(file.seek(SeekFrom::Start(0)));
        try!(file.write(&[0; 2048]));

        Self::new_with_index(file, vec![Vec::new(); 256])
    }

    pub fn new_with_index(file: &'a mut F, index: Vec<Vec<(u32, u32)>>) -> Result<Writer<'a, F>> {
        Ok(Writer {
            file: file,
            index: index,
        })
    }

    /// Write `value` for `key` into this CDB.
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let pos = try!(self.file.seek(SeekFrom::Current(0))) as u32;
        try!(self.file.write(&pack(key.len() as u32)));
        try!(self.file.write(&pack(value.len() as u32)));

        try!(self.file.write(key));
        try!(self.file.write(value));

        let h = hash(key) & 0xffffffff;
        self.index[(h & 0xff) as usize].push((h, pos));
        Ok(())
    }

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

    pub fn as_reader(mut self) -> Result<Reader<'a, F>> {
        {
            let s = &mut self;
            s.finalize();
        }
        Reader::new(self.file)
    }
}

impl<'a, F: Write + Read + Seek + 'a> Drop for Writer<'a, F> {
    /// Write out the index for this CDB.
    fn drop(&mut self) {
        self.finalize();
    }
}
