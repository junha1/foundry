use crate::link::{Linkable, Port, Receiver};
use crate::sandbox::{Error, Result, Sandbox, Sandboxer};
use cbsb::execution::executor::{self, Executor};
use cbsb::ipc::{intra::Intra, servo_channel::ServoChannel, Ipc, IpcRecv, IpcSend, RecvError, Terminate};
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use std::thread;

pub struct ProcSandbox<I: Ipc + 'static, E: Executor> {
    ctx: Option<Mutex<executor::Context<I, E>>>,
}

impl<I: Ipc, E: Executor> ProcSandbox<I, E> {
    pub fn new(path: &dyn AsRef<Path>, id_map: &[(&str, &[&str])], init: &[u8], exports: &[(&str, &[u8])]) -> Self {
        let ctx: executor::Context<I, E> = executor::execute(path.as_ref().to_str().unwrap()).unwrap();
        ctx.ipc.send(&serde_cbor::to_vec(&id_map).unwrap());
        ctx.ipc.send(&serde_cbor::to_vec(&init).unwrap());
        ctx.ipc.send(&serde_cbor::to_vec(&exports).unwrap());
        ProcSandbox {
            ctx: Some(Mutex::new(ctx)),
        }
    }
}

impl<I: Ipc, E: Executor> Linkable for ProcSandbox<I, E> {
    fn supported_linkers(&self) -> &'static [&'static str] {
        &["Base", "DomainSocket", "Intra"]
    }

    fn new_port(&mut self) -> Arc<dyn Port> {
        let (meta1, meta2) = I::arguments_for_both_ends();
        let (common1, common2) = I::arguments_for_both_ends();

        self.ctx.as_ref().unwrap().lock().ipc.send(&serde_cbor::to_vec(&"create_port").unwrap());
        self.ctx.as_ref().unwrap().lock().ipc.send(&serde_cbor::to_vec(&meta2).unwrap());
        self.ctx.as_ref().unwrap().lock().ipc.send(&serde_cbor::to_vec(&common2).unwrap());
        Arc::new(super::port::ProcPort::<I>::new(meta1, common1))
    }
}

impl<I: Ipc, E: Executor> Sandbox for ProcSandbox<I, E> {
    fn sandboxer(&self) -> Arc<dyn Sandboxer> {
        unimplemented!()
    }
}
