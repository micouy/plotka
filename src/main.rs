#![warn(missing_docs)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::new_without_default)]

use ::actix::*;
use ::actix_web::server::HttpServer;
use ::actix_web::*;
use ::clap::{App as Clapp, AppSettings as ClappSettings, Arg, SubCommand};
use ::log::info;
use ::color_backtrace;
use ::pretty_env_logger;

use std::{
    io::{stdin, Stdin},
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

pub mod compose;
pub mod parse;
pub mod prelude;
pub mod server;
pub mod storage;

use self::prelude::*;

fn run_server<P: Parser<Stdin> + Send>(
    parser: P,
    static_path: String,
    ip_addr: String,
) -> (
    thread::JoinHandle<()>,
    Addr<Server<Stdin, P>>,
    Sender<StopAppMessage>,
    Receiver<StopAppMessage>,
) {
    let (addr_tx, addr_rx) = channel();
    let (io_thread_tx, io_thread_rx) = channel();

    let server_handle = {
        let io_thread_tx = io_thread_tx.clone();

        thread::spawn(move || {
            let sys = actix::System::new("Plotka");
            let addr = Arbiter::start(|_| Server::new(io_thread_tx, parser));
            let static_path = static_path;

            {
                let addr = addr.clone();

                HttpServer::new(move || {
                    let state = WsSessionState::new(addr.clone());

                    App::with_state(state)
                        .resource("/", |r| {
                            r.method(http::Method::GET).f(|_| {
                                HttpResponse::Found()
                                    .header("LOCATION", "/static/index.html")
                                    .finish()
                            })
                        })
                        .resource("/ws/", |r| r.route().f(ws_handshake))
                        .handler(
                            "/static/",
                            fs::StaticFiles::new(static_path.clone()).unwrap(),
                        )
                })
                .bind(ip_addr)
                .unwrap()
                .start();
            }

            addr_tx.clone().send(addr).unwrap();

            let _ = sys.run();
        })
    };

    let addr = addr_rx.recv().unwrap();

    (server_handle, addr, io_thread_tx, io_thread_rx)
}

fn run_io<P: Parser<Stdin>>(
    settings: P::Settings,
    addr: Addr<Server<Stdin, P>>,
    from_server_rx: Receiver<StopAppMessage>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let reader_handle = thread::spawn(move || {
            let reader = P::wrap_reader(stdin(), settings);

            reader.for_each(|line| {
                addr.do_send(InputMessage(line.unwrap()));
            });
        });

        loop {
            if let Ok(_message) = from_server_rx.recv() {
                info!("Stopping the reader thread.");
                drop(reader_handle);

                break;
            }
        }
    })
}

fn run_app<P: Parser<Stdin> + Send>(
    parser: P,
    settings: P::Settings,
    static_path: String,
    ip_addr: String,
) {
    // run server and IO thread.
    let (server_handle, server_addr, to_io_tx, from_server_rx) =
        run_server(parser, static_path, ip_addr);
    let io_handle = run_io(settings, server_addr, from_server_rx);

    let _ = server_handle.join();
    // send a stop signal to the stdio thread in case the server hasn't done it.
    let _ = to_io_tx.send(StopAppMessage::new());
    let _ = io_handle.join();
}

fn main() {
    // <3
    ::color_backtrace::install();
    ::pretty_env_logger::init();

    // Parse args and choose the parser...
    let matches = Clapp::new("plotka")
        .version(env!("CARGO_PKG_VERSION"))
        .author("micouay povierjja <szpontaniczny@gmail.com>")
        .help_message("Print help info.")
        .version_message("Print version info.")
        .setting(ClappSettings::DisableHelpSubcommand)
        .setting(ClappSettings::ArgRequiredElseHelp)
        .setting(ClappSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("ip-address")
                .help("Set IP address used to bind the internal server.")
                .long("ip-address")
                .short("a")
                .value_name("ADDRESS"),
        )
        .arg(
            Arg::with_name("static-path")
                .help("Set IP address used to bind the internal server.")
                .long("static-path")
                .short("s")
                .value_name("DIR")
                .required(true),
        )
        .subcommand(
            SubCommand::with_name("json")
                .about("Parse incoming data as JSON.")
                .help_message("Print help info.")
                .version_message("Print version info."),
        )
        .subcommand(
            SubCommand::with_name("csv")
                .about("Parse incoming data as CSV.")
                .help_message("Print help info.")
                .version_message("Print version info.")
                .arg(
                    Arg::with_name("headers")
                        .help("Set CSV headers.")
                        .long("headers")
                        .short("h")
                        .value_name("HEADERS")
                        .multiple(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("delimiter")
                        .help("Set CSV delimiter.")
                        .long("delimiter")
                        .value_name("DELIMITER")
                        .short("d"),
                ),
        )
        .get_matches();
    let ip_addr = matches
        .value_of("ip-address")
        .unwrap_or("127.0.0.1:8080")
        .to_string();
    let static_path = matches.value_of("static-path").unwrap().to_string();

    if let Some(_matches) = matches.subcommand_matches("json") {
        let parser = JsonParser::new();

        run_app(parser, (), static_path, ip_addr);
    } else if let Some(matches) = matches.subcommand_matches("csv") {
        // create a parser.
        let headers = matches
            .values_of("headers")
            .unwrap()
            .map(|h| h.to_string())
            .collect::<Vec<_>>();
        let parser = CsvParser::new(headers.clone());

        // create a reader.
        let delim = matches.value_of("delimiter").map(|d| {
            let bytes = d.as_bytes();
            if bytes.len() == 1 {
                bytes[0]
            } else {
                panic!("invalid CSV delimiter");
            }
        });
        let reader_settings = (headers, delim);

        run_app(parser, reader_settings, static_path, ip_addr);
    }
}
