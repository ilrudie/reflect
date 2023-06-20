use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::{body::Bytes, service::service_fn, Request, Response};
use hyper::{Method, StatusCode};
use log::{error, info};
use std::collections::HashMap;
use std::time::Duration;
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;
use tokio::time;

const BACKLOG: u8 = 128;
// const SVC_NAME: &str = "reflect";

// TODO: path routing?
// TODO: sleep/delay path
// TODO: headers path
// TODO: status path
// TODO: metrics on out of band server
// TODO: health on out of band server
// TODO: inspect peer to try to determine if it's an IPv4 upgrade connection
// TODO: should this have logging? env var for...
// TODO: env var for port override
// TODO: env var for port override out of band
// TODO: env var to bind local-only
// TODO: env var for ipv4-only
// TODO: env var for ipv6-only
// TODO: handle SIGTERM, is this needed?

async fn handle(
    req: Request<hyper::body::Incoming>,
    local: SocketAddr,
    peer: SocketAddr,
) -> Result<Response<Full<Bytes>>, Infallible> {
    info!("handling connection from {}", peer);

    let method = req.method();
    let params: HashMap<String, String> = req
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);

    let after_duration = params.get("after");

    match method {
        &Method::GET => {
            if let Some(dur) = after_duration {
                let dur: Result<u64, _> = dur.parse();
                if let Ok(dur) = dur {
                    info!("requested repsonse after {}s", dur);
                    time::sleep(Duration::new(dur, 0)).await;
                }
            }

            if let Ok(resp) = Response::builder().body(Full::new(Bytes::from(format!(
                "You connected on {} port {}\nHello World!\n",
                local.ip().to_string(),
                local.port().to_string()
            )))) {
                Ok(resp)
            } else {
                Ok(Response::new(Full::new(Bytes::from("Hello World!\n"))))
            }
        }
        _ => serve_error(StatusCode::METHOD_NOT_ALLOWED),
    }
}

async fn build_listener<A>(addr: A) -> TcpListener
where
    A: std::net::ToSocketAddrs,
{
    let addr = std::net::ToSocketAddrs::to_socket_addrs(&addr)
        .unwrap()
        .last()
        .unwrap();

    let l = if addr.is_ipv6() {
        info!("setting up IPv6 listener");
        let builder = net2::TcpBuilder::new_v6().unwrap();
        builder.only_v6(true).unwrap();
        builder.bind(addr).unwrap();
        builder.listen(BACKLOG.into()).unwrap()
    } else {
        info!("setting up IPv4 listener");
        let v4 = net2::TcpBuilder::new_v4().unwrap();
        v4.bind(addr).unwrap().listen(BACKLOG.into()).unwrap()
    };

    l.set_nonblocking(true).unwrap();
    TcpListener::from_std(l).unwrap()
}

async fn listen_and_serve<A>(
    addr: A,
    _name: &str, // likely required for metrics but stubbed out
                 // metrics_cx: Context
) where
    A: std::net::ToSocketAddrs,
{
    // let meter = global::meter(SVC_NAME);
    // let counter = meter.u64_observable_counter(name).init();

    let l = build_listener(addr).await;

    loop {
        let (stream, local) = l.accept().await.unwrap();
        info!("accepted connection on {}", local);
        let peer = stream.peer_addr().unwrap();
        info!("connection is from {}", peer);

        // counter.observe(
        //     &metrics_cx,
        //     1,
        //     &[
        //         KeyValue::new("client_socket", peer.to_string()),
        //         KeyValue::new("local_socket", local.to_string()),
        //     ],
        // );

        tokio::spawn(async move {
            info!("spawning a new task");
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(move |r| self::handle(r, local, peer)))
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
            info!("task complete");
        });
    }

    // if let Ok(listener) = TcpListener::bind(addr).await {
    //     loop {
    //         let (stream, local) = listener.accept().await.unwrap();
    //         let peer = stream.peer_addr().unwrap();

    //         tokio::task::spawn(async move {
    //             if let Err(err) = http1::Builder::new()
    //                 .serve_connection(stream, service_fn(move |r| self::handle(r, local, peer)))
    //                 .await
    //             {
    //                 println!("Error serving connection: {:?}", err);
    //             }
    //         });
    //     }
    // } else {
    //     // this is an error
    // }
}

fn serve_metrics(// metrics: Vec<prometheus::proto::MetricFamily>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // let encoder = TextEncoder::new();
    // let mut result = Vec::new();
    // if let Err(e) = encoder.encode(&metrics, &mut result) {
    //     error!("admin server encountered error decoding metrics {}", e);
    //     return serve_error(StatusCode::INTERNAL_SERVER_ERROR);
    // }
    // let body = Full::from(result);
    let body = Full::from("metrics aren't implemented yet :(\n");
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(body)
        .unwrap())
}

fn serve_error(code: StatusCode) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::builder()
        .status(code)
        .body(Full::default())
        .unwrap())
}

async fn handle_admin(
    r: Request<hyper::body::Incoming>,
    // m: Vec<prometheus::proto::MetricFamily>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let method = r.method().to_owned();
    let path = r.uri().path();
    match (method, path) {
        (Method::GET, "/metrics") => {
            // handle metrics here
            serve_metrics()
        }
        _ => {
            // return error
            serve_error(StatusCode::NOT_FOUND)
        }
    }
}

async fn start_admin_server<A>(addr: A)
where
    A: tokio::net::ToSocketAddrs,
{
    // let exporter = init_prom_exporter();
    let l = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("admin server failed to bind with error: {}", e);
            return; // bail out early with no admin server
        }
    };

    info!("admin server started");

    loop {
        let (stream, _) = l.accept().await.unwrap();
        // let metrics = exporter.registry().gather();
        tokio::spawn(async move {
            info!("spawning a new admin server task");
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(move |r| self::handle_admin(r)))
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
            info!("admin server task complete");
        });
    }
}

// fn init_meter_provider() {
//     let exporter = init_prom_exporter();

//     let gp = global::GlobalMeterProvider::new(exporter.meter_provider().unwrap());

//     // let reader = PeriodicReader::builder(exporter, runtime::Tokio).build();
//     // let p = MeterProvider::builder()
//     //     .with_reader(reader)
//     //     // .with_resource(Resource::new(vec![KeyValue::new(
//     //     //     "service.name",
//     //     //     "metrics-basic-example",
//     //     // )]))
//     //     .build();

//     global::set_meter_provider(gp);
// }

// fn init_prom_exporter() -> PrometheusExporter {
//     let controller = controllers::basic(processors::factory(
//         selectors::simple::inexpensive(), // maybe use histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]) instead if we need hist
//         aggregation::cumulative_temporality_selector(),
//     ))
//     .build();

//     opentelemetry_prometheus::exporter(controller).init()
// }

#[tokio::main]
async fn main() {
    env_logger::init();
    // let metrics_cx = Context::current();
    // init_meter_provider();

    // tokio::spawn(listen_and_serve("0.0.0.0:8080"));
    // tokio::spawn(listen_and_serve("[::]:8080"));

    // loop {}

    let tasks = vec![
        tokio::spawn(listen_and_serve("0.0.0.0:8080", "IPv4")),
        tokio::spawn(listen_and_serve("[::]:8080", "IPv6")),
        tokio::spawn(start_admin_server("localhost:9080")),
    ];

    futures::future::join_all(tasks).await;

    // make sure to bind IPv4 first, on mac at least there is a tcp46 which takes over the tcp4 if bound first
    // let listenerv4 = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    // let socketv4 = TcpSocket::new_v4().unwrap();
    // socketv4.set_reuseaddr(true).unwrap();
    // socketv4.bind("0.0.0.0:8080".parse().unwrap()).unwrap();
    // let listenerv4 = socketv4.listen(BACKLOG.into()).unwrap();

    // let listenerv6 = build_listener("[::]:8080").await;
    // let listenerv4 = build_listener("0.0.0.0:8080").await;

    // let listenerv6 = TcpListener::bind("[::]:8080").await.unwrap();
    // let socketv6 = TcpSocket::new_v6().unwrap();
    // socketv6.set_reuseaddr(true).unwrap();
    // error!("reuse address set to {}", socketv6.reuseaddr().unwrap());
    // socketv6.bind("[::]:8080".parse().unwrap()).unwrap();
    // let listenerv6 = socketv6.listen(BACKLOG.into()).unwrap();

    // loop {
    //     info!("waiting for incoming connections...");
    //     let (stream, local) = tokio::select! {
    //         Ok((stream, addr)) = listenerv6.accept() => {
    //             info!("IPv6 accepted connection");
    //             (stream, addr)
    //         },
    //         Ok((stream, addr)) = listenerv4.accept() => {
    //             info!("ipv4 accepted connection");
    //             (stream, addr)
    //         }
    //     };

    //     let peer = stream.peer_addr().unwrap();

    //     info!("ready to spawn task for {}", local);

    //     tokio::task::spawn(async move {
    //         if let Err(err) = http1::Builder::new()
    //             .serve_connection(stream, service_fn(move |r| self::handle(r, local, peer)))
    //             .await
    //         {
    //             error!("Error serving connection: {:?}", err);
    //         }
    //         info!("finished service connection");
    //     });

    //     info! {"handler spawned; bottom of loop"}
    // }
}
