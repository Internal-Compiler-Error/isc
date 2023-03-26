# isc
Intelligently Selective Copy (`isc`) is a ⚡️amazingly fast⚡️(sorry the meme had to be done) cli tool that copies all 
the files from source to destination directory, but only those files that are not present in the destination directory. 
The equality of files is determined by their sha256 checksums. The tool computes the checksums of the files in parallel!

## Note
Currently the source directory can only contain files and not directories. Haven't decided what is the semantics of 
copying directories involving symbolic links yet.

# Usage
```bash
ics <source> <destination>
```
or if the destination is the current directory
```bash
ics <source>
```

# TODO
- [ ] Add support for copying directories
- [ ] Add support for specifying the number of parallel workers
- [ ] Use `tokio` to make copying parallel