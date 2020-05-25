use crate::link::{Linkable, Port, Receiver};
use crate::sandbox::{Error, Result, Sandbox, Sandboxer};
use cbsb::execution::executor::{self, Executor};
use cbsb::ipc::{intra::Intra, servo_channel::ServoChannel, Ipc, IpcRecv, IpcSend, RecvError, Terminate};
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use std::thread;

/// TODO: FIX THIS
type PortId = u32;

#[cfg(debug_assertions)]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1_000_000);
#[cfg(not(debug_assertions))]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(5000);

struct IpcSendWrapper<T: IpcSend> {
    send: T,
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
pub struct ProcPort<I: Ipc + 'static> {
    id: PortId,

    recv_meta: Option<Mutex<I::RecvHalf>>,
    send_meta: Option<Mutex<I::SendHalf>>,
    recv_common: Option<Mutex<I::RecvHalf>>,
    send_common: Option<Mutex<I::SendHalf>>,

    receiver: Option<thread::JoinHandle<()>>,
}

impl<I: Ipc + 'static> ProcPort<I> {
    pub fn new(ipc_meta_arg: Vec<u8>, ipc_common_arg: Vec<u8>,
        ) -> Self {

        let (send_meta, recv_meta) = I::new(ipc_meta_arg).split();
        let (send_common, recv_common) = I::new(ipc_common_arg).split();
        let id = serde_cbor::from_slice(&recv_meta.recv(Some(TIMEOUT)).unwrap()).unwrap();

        ProcPort {
            id,
            recv_meta: Some(Mutex::new(recv_meta)),
            send_meta: Some(Mutex::new(send_meta)),
            recv_common: Some(Mutex::new(recv_common)),
            send_common: Some(Mutex::new(send_common)),
            receiver: None
        }
    }
}

impl<I: Ipc + 'static>  Port for ProcPort<I> {
    fn export(&mut self, ids: &[usize]) -> &mut dyn Port {
        {
            let ipc_guard = self.send_meta.as_ref().expect("You already linked this port").lock();
            ipc_guard.send(&serde_cbor::to_vec(&"export").unwrap());
            ipc_guard.send(&serde_cbor::to_vec(&ids).unwrap());
        }
        self
    }

    fn import(&mut self, slots: &[&str]) -> &mut dyn Port {
        {
            let ipc_guard = self.send_meta.as_ref().expect("You already linked this port").lock();
            ipc_guard.send(&serde_cbor::to_vec(&"import").unwrap());
            ipc_guard.send(&serde_cbor::to_vec(&slots).unwrap());
        }
        self
    }

    fn receiver(&mut self) -> Box<dyn Receiver> {
        let v = self.send_common.take().expect("You already linked this port").into_inner();
        Box::new(IpcSendWrapper {
            send: v,
        })
    }

    fn link(&mut self, mut receiver: Box<dyn Receiver>) {
        let recv = self.recv_common.take().unwrap().into_inner();


        assert!(self
            .receiver
            .replace(thread::spawn(move || {
                loop {
                    let data = match recv.recv(None) {
                        Err(RecvError::TimeOut) => panic!(),
                        Err(RecvError::Termination) => return,
                        Ok(x) => x,
                    };
                    receiver.receive(Box::new(data))
                }
            }))
            .is_none())
    }
}
