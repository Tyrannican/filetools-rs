# Filetools

Simple crate for perfoming some small `Path` operations in Rust.

Offers the user the ability to:

* Create directories (single / multiple at a time)
* Check given filepaths match a pattern
* List all files / directories in a path
    * This can be just the files / directories inside the path root
    * This can also include files / directories in **ALL** subdirectories contained in the path
* List files / directories as above but filter the results based on a Filter Pattern
* Some general naming functions for creating `PathBuf` names

More will be added in the future but this should suffice for small path operations.

## Usage

Add to your `Cargo.toml`

```toml
[dependencies]
filetools = "0.2.0"
```
