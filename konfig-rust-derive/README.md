konfig-rust-derive is a derive macro helper for konfig-rust

It allows for quickly implementing the `KonfigSection` trait for structs with default behaviour.

Example:
```rust
#[derive(KonfigSection)]
struct Config {
    name: String,
    age: u32,
}
```