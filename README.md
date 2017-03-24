# uuid_v1

[![crates.io version](https://img.shields.io/crates/v/uuid_v1.svg)](https://crates.io/crates/uuid_v1)

**uuid_v1** provides Rust implementation of Universally Unique Identifier (UUID) Version 1.
Implementation inspired by [satori/go.uuid](https://github.com/satori/go.uuid), [rust-lang-nursely/uuid](https://github.com/rust-lang-nursery/uuid)

[Documentation](https://docs.rs/uuid_v1)

## Example

```rust
extern crate uuid_v1;

fn main() {
    let uuid = uuid_v1::new_v1();
    println!("{}", uuid.to_string());
}
```
