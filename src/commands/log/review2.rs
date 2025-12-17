use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use clap::Args as ArgsTrait;
use http_body_util::Full;
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Bytes, Incoming},
    header::CONTENT_TYPE,
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::{
    config::Config, error::Error, inspect::log::resolve_log_dir, inspect2::log::list_logs,
    profile::apply_profile, result::Result,
};

#[derive(ArgsTrait, Debug)]
pub struct Args {
    /// Log directory
    #[arg(long)]
    log_dir: Option<PathBuf>,
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(main_async(args, config))
}

async fn main_async(args: Args, config: &Config) -> Result<()> {
    // TODO bind addr and port from args

    apply_profile(config)?;
    let log_dir = resolve_log_dir(args.log_dir.as_ref());
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    eprintln!("Listening on 127.0.0.1:3000");

    let http = http1::Builder::new();
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();
    let mut signal = std::pin::pin!(shutdown_signal());
    let log_dir = Arc::new(log_dir);

    loop {
        tokio::select! {
            Ok((stream, _addr)) = listener.accept() => {
                let io = TokioIo::new(stream);
                let log_dir = log_dir.clone();
                let service = service_fn(move |req| {
                    let log_dir = log_dir.clone();
                    async move {
                        let resp = handle_req(req, log_dir).await;
                        Ok::<_, Error>(resp)
                    }
                });
                let conn = http.serve_connection(io, service);
                let watched_conn = graceful.watch(conn);
                tokio::spawn(async move {
                    if let Err(e) = watched_conn.await {
                        eprintln!("Error serving connection: {:?}", e);
                    }
                });
            },

            _ = &mut signal => {
                drop(listener);
                break;
            }
        }
    }

    tokio::select! {
        _ = graceful.shutdown() => {
            Ok(())
        },
        _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
            Err(Error::custom("timed out wait for all connections to close"))
        }
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.unwrap();
}

trait ResponseExt {
    fn not_found() -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("Not found")))
            .unwrap()
    }

    fn bad_request() -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::from("Bad request")))
            .unwrap()
    }

    fn server_error() -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Full::new(Bytes::from("Server error")))
            .unwrap()
    }

    // fn plain_text<B: Into<Bytes>>(body: B) -> Response<Full<Bytes>> {
    //     Response::new(Full::new(body.into()))
    // }

    fn json<B: Into<Bytes>>(body: B) -> Response<Full<Bytes>> {
        Response::builder()
            .header(CONTENT_TYPE, "application/json")
            .status(StatusCode::OK)
            .body(Full::new(body.into()))
            .unwrap()
    }
}

impl ResponseExt for Response<Full<Bytes>> {}

async fn handle_req(req: Request<Incoming>, log_dir: Arc<PathBuf>) -> Response<Full<Bytes>> {
    match req.method() {
        &Method::GET => match req.uri().path() {
            "/api/logs" => get_logs(&log_dir),
            _ => Response::not_found(),
        },
        _ => Response::bad_request(),
    }
}

fn get_logs(log_dir: &Path) -> Response<Full<Bytes>> {
    let logs = match list_logs(log_dir) {
        Ok(val) => val,
        Err(e) => {
            log::error!("Error reading logs: {e}");
            return Response::server_error();
        }
    };
    match serde_json::to_string(&logs) {
        Ok(encoded) => Response::json(encoded),
        Err(e) => {
            log::error!("Error encoding logs: {e}");
            Response::server_error()
        }
    }
}
