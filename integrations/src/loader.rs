use crate::error::Error;
use wasmtime::{Engine, Module, Linker, Store};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde_derive::*;
use wasmtime_wasi::p2::WasiCtxBuilder;
use anyhow::Result;
use wasmtime::Instance;
use std::pin::Pin;
use wasmtime_wasi::p2::WasiCtx;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {}

pub struct Instantiation {
    linker: Linker<WasiCtx>,
    store: Store<WasiCtx>,
    instance: Instance,
}

impl Instantiation {
    pub fn new(linker: Linker<WasiCtx>, store: Store<WasiCtx>, instance: Instance) -> Instantiation {
        Self {
            linker,
            store,
            instance
        }
    }
}

pub struct Loader {
    root: PathBuf,
    engine: Engine,
    instantiations: HashMap<String, Instantiation>,
    env: HashMap<String, String>,
    args: Vec<String>,
}

impl Loader {
    /// pass directory to where wasm modules are stored.
    /// as well as a map of env vars.
    pub fn new<P: AsRef<Path>>(dir: P, vars: HashMap<String, String>, args: Vec<String>) -> Result<Self, std::io::Error> {
        
        let root = dir.as_ref().to_path_buf();
        
        let engine = Engine::default();
        let instantiations = HashMap::new();
        Ok(Self {
            root,
            engine,
            instantiations,
            env: vars,
            args
        })
    } 
    
    pub fn load<P: AsRef<Path>>(&mut self, name: &str, module_path: P) -> Result<()> {
        let mut wasi = WasiCtxBuilder::new();
        
        //let module_path = self.root.join(name);
        let module_str = format!("{}", module_path.as_ref().display());
        wasi.arg(&module_str);
        let module = Module::from_file(&self.engine, &module_path)?;

        for arg in self.args.iter() {
            wasi.arg(arg);
        }

        for (k, v) in self.env.iter() {
            wasi.env(k, v);
        }
        
        wasi.socket_addr_check(|addr, socket_use| {
            Box::pin(async move {
                true
            })
        });

        wasi.allow_tcp(true).allow_udp(true);

        let mut store = Store::new(&self.engine, wasi.build());
        let mut linker = Linker::new(&self.engine);

        let instance = linker.instantiate(&mut store, &module)?;

        self.instantiations.insert(name.to_string(), Instantiation::new(linker, store, instance));

        Ok(())
    }
}