use helpers::hash;
use helpers::unpack;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use types::Error;
use types::Result;
use writer::Writer;


/// CDB Reader struct
#[derive(Debug)]
pub struct Reader<'a, F: Read + Seek + 'a> {
    // Opened file to read values from.
    file: &'a mut F,
    // Index for the contents of the CDB.
    index: Vec<(u32, u32)>,
    // Position in the file where the index table starts.
    table_start: usize,
    // How many elements are there in the CDB.
    length: usize,
}

/// Iterator struct for Key, Values in a CDB.
pub struct ItemIterator<'a, 'file: 'a, F: Read + Seek + 'file> {
    reader: &'a mut Reader<'file, F>,
}

/// Iterate over (Key, Values) in a CDB until the end of file.
impl<'a, 'file: 'a, F: Read + Seek + 'file> Iterator for ItemIterator<'a, 'file, F> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.file.seek(SeekFrom::Current(0)) {
            Ok(pos) => {
                if pos >= self.reader.table_start as u64 {
                    return None;
                }
            }
            Err(_) => return None,
        }
        // We're in the Footer of the file, no more items.
        let mut buf: [u8; 8] = [0; 8];
        {
            let mut chunk = self.reader.file.take(8);
            let _ = chunk.read(&mut buf);
        }
        let k = unpack([buf[0], buf[1], buf[2], buf[3]]);  // Key length
        let v = unpack([buf[4], buf[5], buf[6], buf[7]]);  // Value length

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

impl<'a, 'file: 'a, F: Read + Seek + 'file> IntoIterator for &'a mut Reader<'file, F> {
    type Item = (Vec<u8>, Vec<u8>);
    type IntoIter = ItemIterator<'a, 'file, F>;

    fn into_iter(self) -> Self::IntoIter {
        let _ = self.file.seek(SeekFrom::Start(2048));
        ItemIterator { reader: self }
    }
}

impl<'a, F: Read + Seek + 'a> Reader<'a, F> {
    pub fn new(file: &'a mut F) -> Result<Reader<'a, F>> {
        match file.seek(SeekFrom::End(0)) {
            Err(e) => return Err(Error::IOError(e)),
            Ok(n) => {
                if n < 2048 {
                    return Err(Error::CDBTooSmall);
                }
            }
        };

        let mut index: Vec<(u32, u32)> = vec![];
        let mut sum: u32 = 0;

        let mut buf: Vec<u8> = vec![];
        {
            try!(file.seek(SeekFrom::Start(0)));
            let mut chunk = file.take(2048);
            try!(chunk.read_to_end(&mut buf));
        }

        for ix in 0..2048 / 8 {
            let i = ix * 8;
            let k = unpack([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]);
            let v = unpack([buf[i + 4], buf[i + 5], buf[i + 6], buf[i + 7]]);
            sum = sum + (v >> 1);
            index.push((k, v));
        }
        let table_start = index.iter().map(|item| item.0).min().unwrap();

        Ok(Reader {
            file: file,
            index: index,
            table_start: table_start as usize,
            length: sum as usize,
        })
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn get(&mut self, key: &[u8]) -> Vec<Vec<u8>> {
        let mut i = 0;
        let mut values: Vec<Vec<u8>> = vec![];
        loop {
            match self.get_from_pos(key, i) {
                Ok(v) => values.push(v),
                Err(_) => break,
            }
            i += 1;
        }
        values
    }

    pub fn keys(&mut self) -> Vec<Vec<u8>> {
        let mut keys: Vec<Vec<u8>> = vec![];
        for item in self.into_iter() {
            keys.push(item.0);
        }
        keys
    }

    /// Pull the `value` bytes for the first occurence of the given `key` in this CDB.
    pub fn get_first(&mut self, key: &[u8]) -> Result<Vec<u8>> {
        self.get_from_pos(key, 0)
    }

    /// Pull the `value` bytes for the `index`st occurence of the given `key` in this CDB.
    pub fn get_from_pos(&mut self, key: &[u8], index: u32) -> Result<Vec<u8>> {
        // Truncate to 32 bits and remove sign.
        let h = hash(key) & 0xffffffff;
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
                           .map(|item| item.1) {
                let mut buf: [u8; 8] = [0; 8];
                {
                    try!(self.file.seek(SeekFrom::Start(pos as u64)));
                    let mut chunk = self.file.take(8);
                    try!(chunk.read(&mut buf));
                }
                let rec_h = unpack([buf[0], buf[1], buf[2], buf[3]]);
                let rec_pos = unpack([buf[4], buf[5], buf[6], buf[7]]);

                if rec_h == 0 {
                    // Key not in file.
                    return Err(Error::KeyNotInCDB);
                } else if rec_h == h {
                    // Hash of key found in file.
                    {
                        try!(self.file.seek(SeekFrom::Start(rec_pos as u64)));
                        let mut chunk = self.file.take(8);
                        try!(chunk.read(&mut buf));
                    }
                    let klen = unpack([buf[0], buf[1], buf[2], buf[3]]);
                    let dlen = unpack([buf[4], buf[5], buf[6], buf[7]]);

                    let mut buf: Vec<u8> = vec![];
                    {
                        let mut chunk = self.file.take(klen as u64);
                        try!(chunk.read_to_end(&mut buf));
                    }
                    {
                        if buf == key {
                            // Found key in file
                            buf.clear();

                            let mut chunk = self.file.take(dlen as u64);
                            try!(chunk.read_to_end(&mut buf));

                            if counter == index {
                                return Ok(buf);
                            }
                            counter = counter + 1;
                        }
                    }
                }
            }
        }
        Err(Error::KeyNotInCDB)
    }
}

impl<'a> Reader<'a, File> {
    // Needs to be a file to `truncate` at the end.
    pub fn as_writer(mut self) -> Result<Writer<'a, File>> {
        match self.file.seek(SeekFrom::Start(self.table_start as u64)) {
            Ok(_) => {
                let mut index: Vec<Vec<(u32, u32)>> = vec![Vec::new(); 256];

                let mut buf = &mut [0 as u8; 8];
                // Read hash table until end of file to recreate Writer index.
                while let Ok(s) = self.file.read(buf) {
                    if s == 0 {
                        // EOF
                        break;
                    }
                    let h = unpack([buf[0], buf[1], buf[2], buf[3]]);
                    let pos = unpack([buf[4], buf[5], buf[6], buf[7]]);
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
