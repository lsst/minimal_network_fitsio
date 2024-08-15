# Minimal Network FITSIO

This is a very prototyping-stage Rust crate for performing low-level compression and decompression according to the FITS standard, without assuming anything about where those files are stored.
It is expected to be primarily used via a (forthcoming) Python interface along with Python code that takes care of the actual I/O and at least most of the FITS header parsing.
