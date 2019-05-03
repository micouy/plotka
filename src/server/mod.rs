//! Server.

use ::actix::*;
use ::actix_web::*;
use ::log::info;
use ::rand::prelude::*;
use ::serde_json::Value;

use std::{collections::HashMap, io, marker::PhantomData, sync::mpsc::Sender};

use crate::{
    parse::{Parser, record::Record},
    storage::{Storage, StorageError},
    compose::{compose_init_message, compose_push_record_message},
};

mod session;

use self::session::*;

pub use self::session::WsSessionState;

#[derive(Message)]
#[rtype(usize)]
struct Connect {
    addr: Recipient<WsMessage>,
}

#[derive(Message)]
struct Disconnect {
    id: usize,
}

#[derive(Message, Clone)]
struct WsMessage(String);

/// A wrapper around the parser input.
#[derive(Message)]
pub struct InputMessage<I>(pub I)
where
    I: Send;

/// A message to stop other threads.
pub struct StopAppMessage;

/// Internal error.
#[derive(Debug)]
pub enum InternalError {
    Parse,
    Storage(StorageError),
}

impl StopAppMessage {
    pub fn new() -> Self {
        Self {}
    }
}

/// Internal server.
pub struct Server<R, P>
where
    R: 'static + io::Read,
    P: Parser<R>,
{
    sessions: HashMap<usize, Recipient<WsMessage>>,
    storage: Storage,
    rng: SmallRng,
    stop_tx: Sender<StopAppMessage>,
    reader: PhantomData<R>,
    parser: P,
}

impl<R, P> Server<R, P>
where
    R: io::Read,
    P: Parser<R>,
{
    /// Create a new server.
    pub fn new(stop_tx: Sender<StopAppMessage>, parser: P) -> Self {
        Server {
            sessions: HashMap::new(),
            rng: SmallRng::from_entropy(),
            storage: Storage::new(),
            stop_tx,
            reader: PhantomData,
            parser,
        }
    }

    fn broadcast_ws_message(&self, message: &Value) {
        info!("Sending a WS message.");

        let message = WsMessage(message.to_string());

        for session_addr in self.sessions.values() {
            let _ = session_addr.do_send(message.clone());
        }
    }

    fn handle_input<'a>(&'a mut self, input: &'a P::Input) -> Result<Record<'a>, InternalError> {
        let record = self
            .parser
            .parse(input)
            .map_err(|_| InternalError::Parse)?;

        self.storage
            .push_record(&record)
            .map_err(|e| InternalError::Storage(e))?;

        Ok(record)
    }
}

impl<R, P> Actor for Server<R, P>
where
    R: io::Read,
    P: Parser<R>,
{
    type Context = Context<Self>;

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        info!("Stopping...");

        // Close the IO thread.
        // TODO error handling?
        self.stop_tx.send(StopAppMessage::new()).unwrap();

        System::current().stop();

        Running::Stop
    }
}

impl<R, P> Handler<Connect> for Server<R, P>
where
    R: io::Read,
    P: Parser<R>,
{
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        info!("Someone has connected.");

        // Save sessions' address.
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr.clone());

	// Send init message.
	let message = compose_init_message(&self.storage);
	msg.addr.do_send(WsMessage(message.to_string()));

        id
    }
}

impl<R, P> Handler<Disconnect> for Server<R, P>
where
    R: io::Read,
    P: Parser<R>,
{
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        info!("Someone has disconnected.");

        let _ = self.sessions.remove(&msg.id);
    }
}

impl<R, P> Handler<InputMessage<P::Input>> for Server<R, P>
where
    R: io::Read,
    P: Parser<R>,
{
    type Result = ();

    fn handle(&mut self, msg: InputMessage<P::Input>, ctx: &mut Self::Context) {
        info!("Input received.");

        match self.handle_input(&msg.0) {
            Err(_) => {
                // TODO error message? error kind?
                info!("An error occured.");

                // TODO error handling?
                self.stop_tx.send(StopAppMessage {}).unwrap();
                ctx.stop();
            }
            Ok(record) => {
                // Send update message.
                let message = compose_push_record_message(&record);
                self.broadcast_ws_message(&message);
            }
        }
    }
}

pub fn ws_handshake<R, P>(
    req: &HttpRequest<WsSessionState<R, P>>,
) -> Result<HttpResponse, actix_web::Error>
where
    R: io::Read,
    P: Parser<R>,
{
    ws::start(
        req,
        WsSession {
            id: 0,
            parser: PhantomData,
        },
    )
}
