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
filetools = "0.3.0"
```

Then import into your project:

```rust
use filetools::{FtFilter, list_nested_files_with_filter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get all Lua files in the Neovim directory
    let root_path = "/home/user/.config/nvim";
    let filter = FtFilter::Raw("lua".to_string());
    let lua_files = list_nested_files_with_filter(&root_path, filter).await?;

    // Delete them all, we hate Lua
    for lua_file in lua_files.into_iter() {
        tokio::fs::remove_file(lua_file).await?;
    }
}
```
