use ::actix::*;
use ::actix_web::*;

use std::io;

use super::*;

pub struct WsSessionState<R, P>
where
    R: 'static + io::Read,
    P: Parser<R>,
{
    addr: Addr<Server<R, P>>,
}

impl<R, P> WsSessionState<R, P>
where
    R: io::Read,
    P: Parser<R>,
{
    pub fn new(addr: Addr<Server<R, P>>) -> Self {
        Self { addr }
    }
}

pub struct WsSession<R, P>
where
    R: 'static + io::Read,
    P: Parser<R>,
{
    pub id: usize,
    pub parser: PhantomData<(R, P)>,
}

impl<R, P> StreamHandler<ws::Message, ws::ProtocolError> for WsSession<R, P>
where
    R: io::Read,
    P: Parser<R>,
{
    fn handle(&mut self, _msg: ws::Message, _ctx: &mut Self::Context) {}
}

impl<R, P> Actor for WsSession<R, P>
where
    R: io::Read,
    P: Parser<R>,
{
    type Context = ws::WebsocketContext<Self, WsSessionState<R, P>>;

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

impl<R, P> Handler<WsMessage> for WsSession<R, P>
where
    R: io::Read,
    P: Parser<R>,
{
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
