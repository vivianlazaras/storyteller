use crate::error::Error;
use wasmtime::{Engine, Module, Linker, Store};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde_derive::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {}

pub struct Instantiation {
    linker: Linker<Data>,
    store: Store<Data>,
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
        let engine = Engine::default();
        let instantiations = HashMap::new();
        Ok(Self {
            root,
            engine,
            instantiations
        })
    } 
    
}