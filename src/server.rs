//! Server.

use ::actix::*;
use ::actix_web::*;
use ::log::info;
use ::rand::prelude::*;
use ::serde_json::Value;

use std::{collections::HashMap, fmt, marker::PhantomData, sync::mpsc::Sender};

use crate::{parse::Parser, storage::Storage, Error};

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

#[derive(Message)]
pub struct InputMessage<I>(pub I);

/// A message to stop other threads.
pub struct StopApp;

/// Internal server.
pub struct Server<P>
where
    P: Parser,
{
    sessions: HashMap<usize, Recipient<WsMessage>>,
    storage: Storage,
    rng: SmallRng,
    stop_tx: Sender<StopApp>,
    parser: P,
}

impl<P> Server<P>
where
    P: Parser,
{
    /// Create a new server.
    pub fn new(stop_tx: Sender<StopApp>, parser: P) -> Self {
        Server {
            sessions: HashMap::new(),
            rng: SmallRng::from_entropy(),
            storage: Storage::new(),
            stop_tx,
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

    fn handle_input(&mut self, input: P::Input) -> Result<(), Error> {
        let record = self.parser.parse(&input).map_err(|_| Error::Parse)?;

        self.storage
            .push_record(record)
            .map_err(|e| Error::Storage(e))
    }
}

impl<P> Actor for Server<P>
where
    P: Parser,
{
    type Context = Context<Self>;

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        info!("Stopping...");

        // Close the IO thread.
        self.stop_tx.send(StopApp {}).unwrap();

        System::current().stop();

        Running::Stop
    }
}

impl<P> Handler<Connect> for Server<P>
where
    P: Parser,
{
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        info!("Someone has connected.");

        // Save sessions' address.
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr.clone());

        // TODO send init message.

        id
    }
}

impl<P> Handler<Disconnect> for Server<P>
where
    P: Parser,
{
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        info!("Someone has disconnected.");

        let _ = self.sessions.remove(&msg.id);
    }
}

impl<P> Handler<InputMessage<P::Input>> for Server<P>
where
    P: Parser,
{
    type Result = ();

    fn handle(&mut self, msg: InputMessage<P::Input>, _: &mut Self::Context) {
        info!("Input received.");

        // TODO error handling.
        self.handle_input(msg.0).unwrap();
    }
}

pub fn ws_route<P>(
    req: &HttpRequest<WsSessionState<P>>,
) -> Result<HttpResponse, actix_web::Error>
where
    P: Parser,
{
    ws::start(
        req,
        WsSession {
            id: 0,
            phantom_type: PhantomData,
        },
    )
}

pub struct WsSessionState<P>
where
    P: Parser,
{
    pub addr: Addr<Server<P>>,
}

struct WsSession<P>
where
    P: Parser,
{
    id: usize,
    phantom_type: PhantomData<P>,
}

impl<P> StreamHandler<ws::Message, ws::ProtocolError> for WsSession<P>
where
    P: Parser,
{
    fn handle(&mut self, _msg: ws::Message, _ctx: &mut Self::Context) {}
}

impl<P> Actor for WsSession<P>
where
    P: Parser,
{
    type Context = ws::WebsocketContext<Self, WsSessionState<P>>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();

        ctx.state()
            .addr
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    _ => ctx.stop(),
                }

                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        ctx.state().addr.do_send(Disconnect { id: self.id });

        Running::Stop
    }
}

impl<P> Handler<WsMessage> for WsSession<P>
where
    P: Parser,
{
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
