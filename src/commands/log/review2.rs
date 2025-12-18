use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use clap::Args as ArgsTrait;
use http_body_util::Full;
use hyper::{
    Method, Request, Response, StatusCode, Uri,
    body::{Bytes, Frame, Incoming, SizeHint},
    header,
    server::conn::http1,
    service::service_fn,
};
use hyper_staticfile::Static;
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

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

pub fn main(args: Args, config: &Config) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(main_async(args, config))
}

async fn main_async(args: Args, config: &Config) -> Result<()> {
    apply_profile(config)?;
    let log_dir = Arc::new(resolve_log_dir(args.log_dir.as_ref()));

    // Listen addr
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = TcpListener::bind(addr).await?;
    eprintln!("Listening on 127.0.0.1:{}", args.port);

    // Handler for static content
    // TODO look relative to program bin for default location
    let client_home =
        std::env::var("GAGE_REVIEW_CLIENT").unwrap_or("../gage-review/dist/client".into());
    let static_ = Static::new(Path::new(&client_home));

    // Graceful shutdown state
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();
    let mut signal = std::pin::pin!(shutdown_signal());

    // HTTP connection builder
    let http = http1::Builder::new();

    loop {
        tokio::select! {
            Ok((stream, _addr)) = listener.accept() => {
                let io = TokioIo::new(stream);
                let log_dir = log_dir.clone();
                let static_ = static_.clone();
                let service = service_fn(move |req| {
                    let log_dir = log_dir.clone();
                    let static_ = static_.clone();
                    async move {
                        let resp = handle_req(req, log_dir, static_).await;
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

enum Body {
    Full(Full<Bytes>),
    File(hyper_staticfile::Body),
}

impl<F: Into<Full<Bytes>>> From<F> for Body {
    fn from(value: F) -> Self {
        Self::Full(value.into())
    }
}

impl Body {
    fn from_staticfile(value: hyper_staticfile::Body) -> Self {
        Self::File(value)
    }
}

impl hyper::body::Body for Body {
    type Data = Bytes;
    type Error = std::io::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<std::result::Result<Frame<Self::Data>, Self::Error>>> {
        match &mut *self.get_mut() {
            Self::Full(inner) => match Pin::new(inner).poll_frame(cx) {
                Poll::Ready(val) => Poll::Ready(val.map(|val| Ok(val.unwrap()))),
                Poll::Pending => Poll::Pending,
            },
            Self::File(inner) => Pin::new(inner).poll_frame(cx),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Self::Full(full) => full.is_end_stream(),
            Self::File(file) => file.is_end_stream(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Full(inner) => inner.size_hint(),
            Self::File(inner) => inner.size_hint(),
        }
    }
}

trait ResponseExt {
    fn bad_request() -> Response<Body> {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Bad reqeust"))
            .unwrap()
    }

    fn server_error() -> Response<Body> {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Server error"))
            .unwrap()
    }

    fn json(encoded: String) -> Response<Body> {
        Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .status(StatusCode::OK)
            .body(Body::from(encoded))
            .unwrap()
    }
}

impl ResponseExt for Response<Body> {}

async fn handle_req(
    req: Request<Incoming>,
    log_dir: Arc<PathBuf>,
    static_: Static,
) -> Response<Body> {
    if req.method() != &Method::GET {
        return Response::bad_request();
    }
    match req.uri().path() {
        "/api/logs" => get_logs(&log_dir),
        _ => handle_static(req, static_).await,
    }
}

const STATIC_ROUTES: [&str; 3] = ["/_shell.html", "/favicon.ico", "/assets/"];

async fn handle_static(req: Request<Incoming>, static_: Static) -> Response<Body> {
    // Serve `/_shell.html` for any unknown routes
    let path = req.uri().path();
    let known_route = STATIC_ROUTES.iter().any(|route| path.starts_with(route));
    let req = if known_route { req } else { to_shell_req(req) };
    match static_.serve(req).await {
        Ok(resp) => {
            let (parts, body) = resp.into_parts();
            Response::from_parts(parts, Body::from_staticfile(body))
        }
        Err(e) => {
            log::error!("Error reading static file: {}", e);
            Response::server_error()
        }
    }
}

fn to_shell_req<T>(req: Request<T>) -> Request<T> {
    // Tear down req to change path
    let (mut parts, body) = req.into_parts();
    let mut uri = parts.uri.into_parts();

    // Set path to `/_shell.html` while preserving query
    uri.path_and_query = Some(
        uri.path_and_query
            .and_then(|p| p.query().map(|q| format!("/_shell.html?{q}")))
            .unwrap_or_else(|| "/_shell.html".to_string())
            .try_into()
            .unwrap(),
    );

    // Rebuild req
    parts.uri = Uri::from_parts(uri).unwrap();
    Request::from_parts(parts, body)
}

fn get_logs(log_dir: &Path) -> Response<Body> {
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
