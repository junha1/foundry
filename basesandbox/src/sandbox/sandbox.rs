use codechain_module::link::{Linkable, Port};
use codechain_module::sandbox::{Error, Result, Sandbox, Sandboxer};
use std::path::Path;
use std::sync::Arc;

pub struct ProcSandbox {}

impl Linkable for ProcSandbox {
    fn supported_linkers(&self) -> &'static [&'static str] {
        &["Base", "DomainSocket", "Intra"]
    }

    fn new_port(&mut self) -> Arc<dyn Port> {
        
    }
}