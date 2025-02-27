pub mod format;

use crate::format::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{read, File};
use std::io::Write;
use std::path::Path;
use std::ptr::NonNull;
use std::str;
use thiserror::Error;

/// This is the error enum for konfig-rust, it contains all possible errors returned by konfig-rust
///
/// Always make sure to check for relevant error types
#[derive(Debug, Error)]
pub enum KonfigError {
    #[error("Validation callback error {0}")]
    ValidationError(String),
    #[error("OnLoad callback error {0}")]
    OnLoadError(String),
    #[error("Marshal error {0}")]
    MarshalError(String),
    #[error("Unmarshal error {0}")]
    UnmarshalError(String),
    #[error("Load error {0}")]
    LoadError(String),
    #[error("Save error {0}")]
    SaveError(String),
    #[error("Registration error {0}")]
    RegistrationError(String),
    #[error(transparent)]
    Other(#[from] Box<dyn Error>),
}

/// This is the trait allowing a struct to be registered as a section and managed by `KonfigManager`
///
/// You can use the shorthand `#[derive(KonfigSection)]` macro to implement automatically implement it using defaults
///
/// If you want to use custom validate or on_load callbacks, you have to implement this trait manually,
/// there are plans to allow for adding custom callbacks using the derive macro or with a functional interface
pub trait KonfigSection {
    fn name(&self) -> Cow<'_, str>;
    fn validate(&self) -> Result<(), KonfigError> {
        Ok(())
    }
    fn on_load(&self) -> Result<(), KonfigError> {
        Ok(())
    }
    fn to_bytes(&self, format: &FormatHandlerEnum) -> Result<Vec<u8>, KonfigError>;
    fn update_from_bytes(
        &mut self,
        bytes: &[u8],
        format: &FormatHandlerEnum,
    ) -> Result<(), KonfigError>;
}

// had to go into unsafe land to deliver dx 🤷
struct SectionPtr {
    ptr: NonNull<dyn KonfigSection>,
}

unsafe impl Send for SectionPtr {}
unsafe impl Sync for SectionPtr {}

impl SectionPtr {
    fn new<T: KonfigSection + 'static>(section: &mut T) -> Self {
        // SAFETY: We're creating a pointer to an object we know exists
        // The caller must ensure the object outlives all uses of this pointer
        let ptr = unsafe { NonNull::new_unchecked(section as *mut T as *mut dyn KonfigSection) };
        SectionPtr { ptr }
    }

    unsafe fn as_ref(&self) -> &dyn KonfigSection {
        unsafe { self.ptr.as_ref() }
    }

    unsafe fn as_mut(&mut self) -> &mut dyn KonfigSection {
        unsafe { self.ptr.as_mut() }
    }
}

/// Used for configuring `KonfigManager`
pub struct KonfigOptions {
    /// The format KonfigManager is supposed to use for the config file, possible options are in the `Format` enum
    pub format: Format,
    // If `true`, KonfigManager will try to load the config file when it is created
    // auto_load: bool,
    /// If `true` will try to save data to the config file on panic and SIGINT and SIGTERM (currently noop due to rust lifetime issues)
    pub auto_save: bool,
    /// Whether to call the validate and on_load callbacks when loading the data from file
    pub use_callbacks: bool,
    /// Path to the file used for configuration, if the file does not exist it will be created,
    /// the path can be absolute or relative
    pub config_path: String,
}

/// The main manager in konfig-rust, this is intended to be created near the start of your program, and destroyed by closing it
///
/// `KonfigManager` allows you to register sections, which then can be loaded and saved into a single file
///
/// ## Keep in mind this uses raw pointers to store data section, hence why it is supposed to be used, as a single "source of truth" in your app, likely as a global instance
///
/// example:
/// ```
/// use serde::{Deserialize, Serialize};
/// use konfig-rust::*;
/// use konfig-rust::format::*;
///
/// use konfig_derive::KonfigSection;
///
/// #[derive(Serialize, Deserialize, KonfigSection)] // Aside from KonfigSection, you also have to use the Serialize and Deserialize macros
/// struct Config {
///     name: String,
///     age: u32,
/// }
///
/// let mut c = Config { name: "Bob".to_string(), age: 32 };
///
/// let mut manager = KonfigManager::new(KonfigOptions {
///     format: Format::JSON,
///     auto_save: true,
///     use_callbacks: true,
///     config_path: "config.json".to_string(),
/// });
///
/// manager.register_section(&mut c).unwrap();
///
/// manager.load().unwrap();
///
/// println!("Name: {}, Age: {}", c.name, c.age); // Notice how you just access the struct like normal in memory state storage
///
/// manager.save().unwrap();
/// ```
pub struct KonfigManager {
    opts: KonfigOptions,
    format_handler: FormatHandlerEnum,
    path: Box<Path>,
    sections: HashMap<String, SectionPtr>,
}

// lazy_static! {
//         static ref SAVE_CLOSURE: Mutex<Vec<Box< FnOnce() + Send() + 'static>>> = Mutex::new(Vec::new());
// }

impl KonfigManager {
    /// Simply creates a new `KonfigManager`, with the passed in `KonfigOptions`
    pub fn new(opts: KonfigOptions) -> Self {
        let m = KonfigManager {
            format_handler: opts.format.create_handler(),
            path: Box::from(Path::new(&opts.config_path)),
            opts,
            sections: HashMap::new(),
        };

        // probably just gonna rawdog pointers here to, couse rust cries too much about it
        if m.opts.auto_save {
            // setup panic hook
            // let prev_hook = panic::take_hook();
            // panic::set_hook(Box::new(move |panic_info| {
            //     &m.save().unwrap();
            //     prev_hook(panic_info);
            // }));

            // TODO: setup fully later
            // setup SIGINT and SIGTERM
            // let mut signals = Signals::new(&[signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM]).unwrap();

        }

        m
    }

    /// Loads the found config data from the specified file into the registered sections
    ///
    /// Throws: `KonfigError::LoadError`
    pub fn load(&mut self) -> Result<(), KonfigError> {
        if File::open(&self.path).is_err() {
            File::create(&self.path).map_err(|err| KonfigError::LoadError(err.to_string()))?;
        }
        let data = read(&self.path).map_err(|err| KonfigError::LoadError(err.to_string()))?;

        if data.is_empty() {
            return Ok(());
        }

        let config: serde_json::Value = match &self.format_handler {
            FormatHandlerEnum::JSON(handler) => handler.unmarshal(data.as_slice())?,
            FormatHandlerEnum::YAML(handler) => handler.unmarshal(data.as_slice())?,
            FormatHandlerEnum::TOML(handler) => handler.unmarshal(data.as_slice())?,
        };

        let config_map = config
            .as_object()
            .ok_or_else(|| KonfigError::LoadError("Config root must be an object".to_string()))?;

        for (name, section_value) in config_map {
            if let Some(section_ptr) = self.sections.get_mut(name) {
                let bytes = self.format_handler.marshal(section_value)?;
                unsafe {
                    let section = section_ptr.as_mut();
                    section.update_from_bytes(&bytes, &self.format_handler)?;
                    if self.opts.use_callbacks {
                        section.validate()?;
                        section.on_load()?;
                    }
                }
            }
        }

        Ok(())
    }

    // fn internal_save(&self) {
    //     let mut closures = SAVE_CLOSURE.lock().unwrap();
    //     while let Some(closure) = closures.pop() {
    //         closure();
    //     }
    // }

    /// Saves the registered sections to the specified file
    ///
    /// Throws: `KonfigError::SaveError`
    pub fn save(&self) -> Result<(), KonfigError> {
        let mut map: HashMap<String, serde_json::Value> = HashMap::new();

        for (name, section_ptr) in &self.sections {
            let section = unsafe { section_ptr.as_ref() };
            let bytes = section.to_bytes(&self.format_handler)?;

            let value: serde_json::Value = match &self.format_handler {
                FormatHandlerEnum::JSON(_) => serde_json::from_slice(&bytes)
                    .map_err(|err| KonfigError::UnmarshalError(err.to_string()))?,
                FormatHandlerEnum::YAML(_) => serde_yaml::from_slice(&bytes)
                    .map_err(|err| KonfigError::UnmarshalError(err.to_string()))?,
                FormatHandlerEnum::TOML(_) => {
                    let s = str::from_utf8(&bytes)
                        .map_err(|err| KonfigError::UnmarshalError(err.to_string()))?;
                    toml::from_str(s).map_err(|err| KonfigError::UnmarshalError(err.to_string()))?
                }
            };

            map.insert(name.clone(), value);
        }

        let out = self.format_handler.marshal(&map)?;

        let mut f =
            File::create(&self.path).map_err(|err| KonfigError::SaveError(err.to_string()))?;

        f.write_all(out.as_slice())
            .map_err(|err| KonfigError::SaveError(err.to_string()))?;

        Ok(())
    }

    /// Registers a new section with the KonfigManager, the section must use the `Serialize` and `Deserialize` macros
    /// and implement the `KonfigSection` trait (or use the `#[derive(KonfigSection)]` macro to do it for you)
    ///
    /// Throws: `KonfigError::RegistrationError`
    pub fn register_section<T>(&mut self, section: &mut T) -> Result<(), KonfigError>
    where
        T: KonfigSection + 'static,
    {
        let name = section.name().to_string();

        if self.sections.contains_key(&name) {
            return Err(KonfigError::RegistrationError(format!(
                "Failed to register {}, section already registered",
                name
            )));
        }

        let section_ptr = SectionPtr::new(section);

        self.sections.insert(name, section_ptr);
        Ok(())
    }

    /// Quick helper that runs the validation callback on all registered sections,
    /// useful if the config is modified a lot in memory but rarely saved or loaded
    pub fn validate_all(&self) -> Vec<(String, Result<(), KonfigError>)> {
        self.sections
            .iter()
            .map(|(name, section_ptr)| {
                let result = unsafe { section_ptr.as_ref().validate() };
                (name.clone(), result)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use konfig_derive::KonfigSection;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_konfig() {
        #[derive(Serialize, Deserialize, KonfigSection)]
        struct TestData {
            a: i32,
            b: String,
        }

        #[derive(Serialize, Deserialize, KonfigSection)]
        struct TestData2 {
            port: String,
            host: String,
        }

        let mut t = TestData {
            a: 1,
            b: "test".to_string(),
        };

        let mut t2 = TestData2 {
            port: "8080".to_string(),
            host: "localhost".to_string(),
        };

        let mut mngr = KonfigManager::new(KonfigOptions {
            format: Format::JSON,
            auto_save: false,
            use_callbacks: true,
            config_path: "test.json".to_string(),
        });

        mngr.register_section(&mut t)
            .map_err(|err| println!("{}", err.to_string()))
            .unwrap();

        mngr.register_section(&mut t2)
            .map_err(|err| println!("{}", err.to_string()))
            .unwrap();

        mngr.load()
            .map_err(|err| println!("{}", err.to_string()))
            .unwrap();

        t.a = t.a + 1;

        mngr.save()
            .map_err(|err| println!("{}", err.to_string()))
            .unwrap();

        for (name, result) in mngr.validate_all() {
            println!("{}: {}", name, result.is_ok());
        }
        println!("TestData: {}, {}", t.a, t.b);
        println!("TestData2: {}, {}", t2.port, t2.host);
    }
}
