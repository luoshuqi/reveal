reveal
======

This library provides a macro to automatically add context to error.

## Example

```rust
use std::fs;
use std::path::Path;

use reveal::error;

fn main() {
    let e = save_user_data(1, "test").unwrap_err();
    println!("{}", e);
}


#[error]
fn save_user_data(uid: usize, data: &str) -> reveal::Result<()> {
    file_put_contents(Path::new(&format!("/root/{}", uid)), data.as_bytes())?;
    Ok(())
}

// `content = "_"` excludes argument `content` from context
#[error(content = "_")]
fn file_put_contents(path: &Path, content: &[u8]) -> reveal::Result<()> {
    fs::write(path, content)?;
    Ok(())
}
```

```console
Permission denied (os error 13)
0: fs :: write(path, content)
		in file_put_contents(path = "/root/1")
		at src/main.rs:21
1: file_put_contents(Path :: new(& format! ("/root/{}", uid)), data.as_bytes())
		in save_user_data(uid = 1, data = "test")
		at src/main.rs:14
```