# konfig-rust

A simple, configuration management library for Rust. `konfig-rust` lets you focus on app logic and design instead of config management.

This library is a Rust implementation of my [konfig-go](https://github.com/kociumba/konfig-go) library.

## Features

- Allows for unrelated configuration to live in the same config file
- Automatically manages configuration loading and saving
- Supports JSON, YAML, and TOML formats
- Uses derive macros for easy section definition
- Supports callbacks for custom logic on load and save
- Allows for complete custom flow, since you can decide when the data is saved or loaded

## Planned features

- Support more file formats (e.g., environment variables, command-line arguments)
- Allow the user to supply their own format handler
- *Have a feature suggestion? Open an issue!*

## Installation

```bash
cargo add konfig-rust
```

If you want to use the `#[derive(KonfigSection)]` macro, you also need the `konfig-rust-derive` crate

```bash
cargo add konfig-rust-derive
```

### Quick Start

This is a very simple example, more complex ones are planned as examples in the github repo.

```rust
use serde::{Deserialize, Serialize};
use konfig_rust::*;
use konfig_rust::format::*;

use konfig_rust_derive::KonfigSection;

#[derive(Serialize, Deserialize, KonfigSection)] // Aside from KonfigSection, you also have to use the Serialize and Deserialize macros
struct Config {
    name: String,
    age: u32,
}

fn main() {
    let mut c = Config { name: "Bob".to_string(), age: 32 };

    let mut manager = KonfigManager::new(KonfigOptions {
        format: Format::JSON.create_handler(), // since version v0.1.2 you can implement and provide your own formats
        auto_save: false,
        use_callbacks: true,
        config_path: "config.json".to_string(),
    });

    manager.register_section(&mut c).unwrap();

    manager.load().unwrap();

    println!("Name: {}, Age: {}", c.name, c.age); // Notice how you just access the struct like normal in memory state storage

    c.age = c.age + 1;
    
    manager.save().unwrap();
}
```

### Detailed Usage

#### Configuration Manager Options

The KonfigOptions struct provides several options to customize the behavior of the configuration manager:

```rust
/// Options for the KonfigManager
pub struct KonfigOptions {
    /// Format of the configuration file
    pub format: Format,
    /// Path to the configuration file
    pub config_path: String,
    /// Currently noop due to lifetime issues, meant to register callbacks for panic, SIGINT and SIGTERM to save the data
    pub auto_save: bool,
    /// Whether to use callbacks (on_load, validate)
    pub use_callbacks: bool,
}
```

#### Creating Configuration Sections

Define your configuration struct and derive KonfigSection, Serialize, and Deserialize:

```rust
use serde::{Deserialize, Serialize};
use konfig_rust_derive::KonfigSection;

#[derive(Serialize, Deserialize, KonfigSection)]
#[section_name = "my_section"] // Optional: Define section name, defaults to struct name in snake_case
struct MyConfig {
    setting1: String,
    setting2: i32,
    timeout_ms: u64,
}
```
if you want to provide custom `validate` and `on_load` callbacks, you just have to implement
the `KonfigSection` trait manually instead of using the `KonfigSection` derive macro

#### File Formats

konfig-rust supports three configuration file formats, mirroring the Go version:

1. JSON (`Format::JSON`):
    ```json
    {
    "my_section": {
      "setting1": "value",
      "setting2": 42,
      "timeout_ms": 30000
      }
    }
    ```
2. YAML (`Format::YAML`):
    ```yaml
    my_section:
      setting1: value
      setting2: 42
      timeout_ms: 30000
    ```
3. TOML (`Format::TOML`):
    ```toml
    [my_section]
    setting1 = "value"
    setting2 = 42
    timeout_ms = 30000
    ```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License

## Author

Created by [@kociumba](https://github.com/kociumba)