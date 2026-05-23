mod config;
mod routes;
mod middleware;
mod proxy;

use config::{load_config, validate};
use routes::{RouteStore, find_matching};
use middleware::{Logger, RateLimiter};
use proxy::PoolManager;

use http::Request;
use http::Response;
use http::StatusCode;
use http_body_util::Empty;
use bytes::Bytes;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use std::error::Error;
use std::convert::Infallible;
use std::sync::Arc;
use std::env;
use std::process;

const DEFAULT_CONFIG: &str = "config.yaml";
const PID_FILE: &str = "gateway.pid";

type RespBody = Empty<Bytes>;

enum CliAction { Reload, Stop }

fn handle_cli_args() -> Option<CliAction> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { return None; }

    let mut config_file = DEFAULT_CONFIG.to_string();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-t" => {
                if let Err(e) = validate(&config_file) {
                    eprintln!("test failed: {}", e);
                    process::exit(1);
                }
                println!("test is successful");
                process::exit(0);
            }
            "-h" | "--help" => {
                println!("Usage: gateway [OPTIONS]");
                println!("  -t            test config (nginx -t)");
                println!("  -c <file>     config file (default: config.yaml)");
                println!("  -s reload     reload config (nginx -s reload)");
                println!("  -s stop       stop gateway (nginx -s stop)");
                println!("  -h, --help    show this help");
                process::exit(0);
            }
            "-c" => { i += 1; if i < args.len() { config_file = args[i].clone(); } }
            "-s" => {
                i += 1;
                if i >= args.len() { process::exit(1); }
                match args[i].as_str() {
                    "reload" => return Some(CliAction::Reload),
                    "stop" => return Some(CliAction::Stop),
                    _ => process::exit(1),
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn send_signal_to_running(action: CliAction) {
    let pid = std::fs::read_to_string(PID_FILE)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok());

    if let Some(pid) = pid {
        let sig = match action { CliAction::Reload => libc::SIGUSR1, CliAction::Stop => libc::SIGTERM };
        unsafe { libc::kill(pid as i32, sig); }
        println!("signal sent to {}", pid);
    } else {
        eprintln!("gateway not running (no PID file)");
        process::exit(1);
    }
}

async fn handle_connection(
    stream: TokioIo<tokio::net::TcpStream>,
    routes: Arc<RouteStore>,
    pool_manager: Arc<PoolManager>,
    logger: Arc<Logger>,
    rate_limiter: Arc<RateLimiter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let svc = hyper::service::service_fn(move |req: Request<Incoming>| {
        let routes = routes.clone();
        let pm = pool_manager.clone();
        let logger = logger.clone();
        let rl = rate_limiter.clone();
        async move {
            serve_request(req, routes, pm, logger, rl).await
        }
    });

    http1::Builder::new()
        .serve_connection(stream, svc)
        .with_upgrades()
        .await?;
    Ok(())
}

async fn serve_request(
    req: Request<Incoming>,
    routes: Arc<RouteStore>,
    pool_manager: Arc<PoolManager>,
    logger: Arc<Logger>,
    rate_limiter: Arc<RateLimiter>,
) -> Result<Response<RespBody>, Infallible> {
    let path = req.uri().path().to_string();
    let method = req.method().as_str().to_string();
    logger.log_request(&method, &path);

    if !rate_limiter.check("default") {
        return Ok(resp(StatusCode::TOO_MANY_REQUESTS));
    }

    let route = find_matching(routes.all(), &path);
    match route {
        Some(r) => {
            match pool_manager.forward(&r.backend, &path, &r.headers) {
                Ok(_) => { logger.log_response(200, &path); Ok(resp(StatusCode::OK)) }
                Err(e) => { eprintln!("Proxy error: {}", e); Ok(resp(StatusCode::BAD_GATEWAY)) }
            }
        }
        None => Ok(resp(StatusCode::NOT_FOUND)),
    }
}

fn resp(status: StatusCode) -> Response<RespBody> {
    Response::builder().status(status).body(Empty::new()).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Handle CLI args
    if let Some(action) = handle_cli_args() {
        send_signal_to_running(action);
        process::exit(0);
    }

    let cfg = load_config(DEFAULT_CONFIG)?;
    println!("Loaded {} routes", cfg.routes.len());

    std::fs::write(PID_FILE, process::id().to_string()).ok();

    let mut store = RouteStore::new();
    for r in cfg.routes { store.add(r); }
    let store = Arc::new(store);

    let pool_manager = Arc::new(PoolManager::new());
    let logger = Arc::new(Logger::new());
    let rate_limiter = Arc::new(RateLimiter::new(100));

    let addr = format!("{}:{}", cfg.host, cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Listening on {}:{}", cfg.host, cfg.port);
    println!("PID: {}", process::id());

    loop {
        let (stream, remote) = listener.accept().await?;
        println!("From {}", remote);
        let io = TokioIo::new(stream);
        let routes = store.clone();
        let pm = pool_manager.clone();
        let l = logger.clone();
        let rl = rate_limiter.clone();
        tokio::spawn(handle_connection(io, routes, pm, l, rl));
    }
}