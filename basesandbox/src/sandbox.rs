use codechain_module::link::{Linkable, Port};
use codechain_module::sandbox::{Sandboxer, Sandbox};
use std::path::Path;
use std::sync::Arc;
pub struct ProcSandboxer {
    
}

impl Sandboxer for ProcSandboxer {
    fn id(&self) -> &'static str {
        "ProcSandboxer"
    }

    fn supported_module_types(&self) -> &'static [&'static str] {
        &["PlainThread", "UnixProcess"]
    }

    fn load(&self, path: &dyn AsRef<Path>, id_map: &[(&str, &[&str])], init: &[u8], exports: &[(&str, &[u8])]) -> Result<Arc<dyn Sandbox>> {
        
    }
}