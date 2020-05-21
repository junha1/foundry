use crate::ipc::{servo_channel::ServoChannel, Ipc, IpcRecv, IpcSend};
use codechain_module::link::{Linkable, Port, Receiver};
use codechain_module::sandbox::{Error, Result, Sandbox, Sandboxer};
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;

impl Receiver for dyn IpcSend {
    fn receive(&mut self, message: Box<dyn AsRef<[u8]>>) {
        self.send((*message).as_ref())  
    }
}

pub struct ProcPort {
    recv: Option<Mutex<Box<dyn IpcRecv<Terminator = ()>>>>,
    send: Option<Mutex<Box<dyn IpcSend>>>
}

impl Port for ProcPort {
    fn export(&mut self, ids: &[usize]) -> &mut dyn Port {
        {
            let ipc_guard = self.send.as_ref().expect("You already linked this port").lock();
            ipc_guard.send(&serde_cbor::to_vec(&"export").unwrap());
            ipc_guard.send(&serde_cbor::to_vec(&ids).unwrap());
        }
        self
    }

    fn import(&mut self, slots: &[&str]) -> &mut dyn Port {
        {
            let ipc_guard = self.send.as_ref().expect("You already linked this port").lock();
            ipc_guard.send(&serde_cbor::to_vec(&"import").unwrap());
            ipc_guard.send(&serde_cbor::to_vec(&slots).unwrap());
        }
        self
    }

    fn receiver(&self) -> Arc<dyn Receiver> {
        panic!()
    }

    fn link(&mut self, receiver: Arc<dyn Receiver>) {}
}

pub struct ProcSandbox {}

impl ProcSandbox {
    fn get_exported(&self) -> Vec<Vec<u8>> {
        unimplemented!()
    }
}

impl Linkable for ProcSandbox {
    fn supported_linkers(&self) -> &'static [&'static str] {
        &["Base", "DomainSocket", "Intra"]
    }

    fn new_port(&mut self) -> Arc<dyn Port> {
        panic!()
    }
}
