//! Manipulate [DJB's Constant Database](http://cr.yp.to/cdb.html) files. These
//! are 2 level disk-based hash tables that efficiently handle thousands of
//! keys, while remaining space-efficient.
//!
//! Constant databases have the desirable property of requiring low overhead to
//! open.
//!
//! ># What is it?
//! >
//! >cdb is a fast, reliable, simple package for creating and reading constant
//! >databases. Its database structure provides several features:
//! >
//! >* Fast lookups: A successful lookup in a large database normally takes
//! >just two disk accesses. An unsuccessful lookup takes only one.
//! >* Low overhead: A database uses 2048 bytes, plus 24 bytes per record, plus
//! >the space for keys and data.
//! >* No random limits: cdb can handle any database up to 4 gigabytes. There
//! >are no other restrictions; records don't even have to fit into memory.
//! >Databases are stored in a machine-independent format.
//! >* Fast atomic database replacement: cdbmake can rewrite an entire database
//! >two orders of magnitude faster than other hashing packages.
//! >* Fast database dumps: cdbdump prints the contents of a database in
//! >cdbmake-compatible format.
//! >
//! > cdb is designed to be used in mission-critical applications like e-mail.
//! > Database replacement is safe against system crashes. Readers don't have
//! > to pause during a rewrite.
//!
//!
//! > # A structure for constant databases
//! > Copyright 1996  
//! > [D. J. Bernstein](mailto:djb@pobox.com)
//! >
//! > A cdb is an associative array: it maps strings (`keys`) to strings
//! > (`data`).
//! >
//! > A cdb contains 256 pointers to linearly probed open hash tables. The
//! > hash tables contain pointers to `(key, data)` pairs. A cdb is stored in
//! > a single file on disk:
//! >
//! > ```text
//! > +----------------+---------+-------+-------+-----+---------+
//! > | p0 p1 ... p255 | records | hash0 | hash1 | ... | hash255 |
//! > +----------------+---------+-------+-------+-----+---------+
//! > ```
//! >
//! > Each of the 256 initial pointers states a position and a length. The
//! > position is the starting byte position of the hash table. The length
//! > is the number of slots in the hash table.
//! >
//! > Records are stored sequentially, without special alignment. A record
//! > states a key length, a data length, the key, and the data.
//! >
//! > Each hash table slot states a hash value and a byte position. If the
//! > byte position is 0, the slot is empty. Otherwise, the slot points to
//! > a record whose key has that hash value.
//! >
//! > Positions, lengths, and hash values are 32-bit quantities, stored in
//! > little-endian form in 4 bytes. Thus a cdb must fit into 4 gigabytes.
//! >
//! > A record is located as follows. Compute the hash value of the key in
//! > the record. The hash value modulo 256 is the number of a hash table.
//! > The hash value divided by 256, modulo the length of that table, is a
//! > slot number. Probe that slot, the next higher slot, and so on, until
//! > you find the record or run into an empty slot.
//! >
//! > The cdb hash function is `h = ((h << 5) + h) ^ c`, with a starting
//! > hash of `5381`.
//!
//! #Example
//!
//! To write to a new CDB:
//!
//! ```
//! # use galvanize::Result;
//! use galvanize::Writer;
//! use std::fs::File;
//!
//! # // Doing this to get around the fact that you can't have `try!` in `main`.
//! # fn main() {
//! #     let _ = do_try();
//! # }
//! #
//! # fn do_try() -> Result<()> {
//! # let filename = "lib_writer_example.cdb";
//! #
//! let key = "key".as_bytes();
//! let value = "value".as_bytes();
//!
//! let mut f = File::create(filename)?;
//! let mut cdb_writer = Writer::new(&mut f)?;
//! cdb_writer.put(key, value)?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! To read from an existing CDB:
//!
//! ```
//! # use galvanize::Result;
//! # use std::fs::File;
//! use galvanize::Reader;
//!
//! # // Doing this to get around the fact that you can't have `try!` in `main`.
//! # fn main() {
//! #     let _ = do_try();
//! # }
//! #
//! # fn do_try() -> Result<()> {
//! # let filename = "lib_writer_example.cdb";
//! #
//! let mut f = File::open(filename)?;
//! let mut cdb_reader = Reader::new(&mut f)?;
//! #
//! # let key = "key".as_bytes();
//! # let value = "value".as_bytes();
//!
//! let stored_vals = cdb_reader.get(key);
//! assert_eq!(stored_vals.len(), 1);
//! assert_eq!(&stored_vals[0][..], &value[..]);  // "value".as_bytes()
//!
//! // The CDB contains only one entry:
//! assert_eq!(cdb_reader.len(), 1);
//!
//! let non_existing_key = "non_existing_key".as_bytes();
//! let empty = cdb_reader.get(non_existing_key);
//! assert_eq!(empty.len(), 0);
//!
//! // Accessing a key that isn't in the CDB:
//! let non_existing_key = "non_existing_key".as_bytes();
//! let empty = cdb_reader.get(non_existing_key);
//! assert_eq!(empty.len(), 0);
//!
//! assert!(cdb_reader.get_first(non_existing_key).is_err());
//! #
//! #     Ok(())
//! # }
//! ```

pub mod helpers;
pub mod reader;
pub mod types;
pub mod writer;

pub use reader::Reader;
pub use types::{Error, Result};
pub use writer::Writer;
