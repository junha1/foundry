use crate::ipc::{servo_channel::ServoChannel, Ipc, IpcRecv, IpcSend, Terminate, RecvError, intra::Intra};
use codechain_module::link::{Linkable, Port, Receiver};
use codechain_module::sandbox::{Error, Result, Sandbox, Sandboxer};
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use std::thread;

struct IpcSendWrapper<T: IpcSend> {
    send: T
}

impl<T: IpcSend> Receiver for IpcSendWrapper<T> {
    fn receive(&mut self, message: Box<dyn AsRef<[u8]>>) {
        self.send.send((*message).as_ref())
    }
}

/// ProcPort is a statemachine.
/// 1. State INIT.
/// 2.
///
pub struct ProcPort<R: IpcRecv + 'static, S: IpcSend + 'static> {
    recv: Option<Mutex<R>>,
    send: Option<Mutex<S>>,
    receiver: Option<thread::JoinHandle<()>>
}

impl<R: IpcRecv+ 'static, S: IpcSend + 'static> Port for ProcPort<R, S> {
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

    fn receiver(&mut self) -> Box<dyn Receiver> {
        let v = self.send.take().expect("You already linked this port").into_inner();
        Box::new(IpcSendWrapper{send: v})
    }

    fn link(&mut self, mut receiver: Box<dyn Receiver>) {
        let recv = self.recv.take().unwrap().into_inner();
        assert!(self.receiver.replace(thread::spawn(move || {
            loop {
                let data = match recv.recv(None) {
                    Err(RecvError::TimeOut) => panic!(),
                    Err(RecvError::Termination) => return,
                    Ok(x) => x,
                };  
                receiver.receive(Box::new(data))
            }
        })).is_none())
    }
}

pub struct ProcSandbox {
    /// All ports that this sandbox creates will use this IPC scheme.
    ipc_type: String
}

impl ProcSandbox {
    pub fn new(path: &dyn AsRef<Path>,id_map: &[(&str, &[&str])],
    init: &[u8],
    exports: &[(&str, &[u8])]) -> Self {
        unimplemented!()
    }

    fn create_new_ipc(&self) -> (){
        unimplemented!()
    }

    fn get_exported(&self) -> Vec<Vec<u8>> {
        unimplemented!()
    }
}

impl Linkable for ProcSandbox {
    fn supported_linkers(&self) -> &'static [&'static str] {
        &["Base", "DomainSocket", "Intra"]
    }

    fn new_port(&mut self) -> Arc<dyn Port> {
        match &self.ipc_type[..] {
            "Intra" => {
                unimplemented!()
            },
            _ => panic!()
        }
    }
}

impl Sandbox for ProcSandbox {
    fn sandboxer(&self) -> Arc<dyn Sandboxer> {
        unimplemented!()
    }
}