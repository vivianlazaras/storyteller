use crate::errors::Error;
use wasmtime::{Engine, Module, Linker, Store};

pub struct Instantiation {
    linker: Linker,
    store: Store,
}

pub struct Loader {
    root: PathBuf,
    engine: Engine,
    instantiations: HashMap<String, Instantiation>,
}

impl Loader {
    /// pass directory to where wasm modules are stored.
    pub fn new<P: AsRef<Path>>(dir: P) -> Result<Self, std::io::Error> {
        let root = dir.as_ref().to_path_buf();
    } 
    pub fn load(integrations: Vec<Integration>) -> Result<Self, crate::error::Error> {

    }
}