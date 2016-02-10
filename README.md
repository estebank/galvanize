## Galvanize: Pure Rust CDB library [![Build Status](https://travis-ci.org/estebank/galvanize.svg?branch=master)](https://travis-ci.org/estebank/galvanize)

Manipulate [DJB's Constant Database](http://cr.yp.to/cdb.html) files. These are
2 level disk-based hash tables that efficiently handle thousands of keys, while
remaining space-efficient.

Constant databases have the desirable property of requiring low overhead to
open.

>#What is it?
>
>cdb is a fast, reliable, simple package for creating and reading constant
>databases. Its database structure provides several features:
>
>* Fast lookups: A successful lookup in a large database normally takes just
>two disk accesses. An unsuccessful lookup takes only one.
>* Low overhead: A database uses 2048 bytes, plus 24 bytes per record, plus
>the space for keys and data.
>* No random limits: cdb can handle any database up to 4 gigabytes. There are
>no other restrictions; records don't even have to fit into memory.
>Databases are stored in a machine-independent format.
>* Fast atomic database replacement: cdbmake can rewrite an entire database
>two orders of magnitude faster than other hashing packages.
>* Fast database dumps: cdbdump prints the contents of a database in
>cdbmake-compatible format.
>
> cdb is designed to be used in mission-critical applications like e-mail.
> Database replacement is safe against system crashes. Readers don't have to
> pause during a rewrite.


---

Originally based on [dw/python-pure-cdb (Pure Python CDB
Library)](https://github.com/dw/python-pure-cdb).
