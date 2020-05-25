use super::sandbox::ProcSandbox;
use crate::link::{Linkable, Port};
use crate::sandbox::{Error, Result, Sandbox, Sandboxer};
use std::path::Path;
use std::sync::Arc;

/// ProcSandboxer is actually one of the useless case of Sandboxer.
/// We don't have to keep something extra to execute an Unix process.
pub struct ProcSandboxer {}

impl Sandboxer for ProcSandboxer {
    fn id(&self) -> &'static str {
        "ProcSandboxer"
    }

    fn supported_module_types(&self) -> &'static [&'static str] {
        &["PlainThread", "UnixProcess"]
    }

    fn load(
        &self,
        path: &dyn AsRef<Path>,
        id_map: &[(&str, &[&str])],
        init: &[u8],
        exports: &[(&str, &[u8])],
    ) -> Result<Arc<dyn Sandbox>> {
        Ok(Arc::new(ProcSandbox::<cbsb::ipc::servo_channel::ServoChannel, cbsb::execution::executor::Executable>::new(
            path, id_map, init, exports,
        )))
    }
}

#[cfg(test)]
mod test {
    use super::super::sandbox::*;
    use super::*;
}
