use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::{body::Bytes, service::service_fn, Request, Response};
use hyper::{Method, StatusCode};
use log::{error, info};
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;

const BACKLOG: u8 = 128;

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
    match req.method() {
        &Method::GET => {
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
        _ => Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Full::default())
            .unwrap()),
    }
}

// async fn build_listener<A>(addr: A) -> TcpListener
// where
//     A: std::net::ToSocketAddrs,
// {
//     let addr = std::net::ToSocketAddrs::to_socket_addrs(&addr)
//         .unwrap()
//         .last()
//         .unwrap();

//     let l = if addr.is_ipv6() {
//         info!("setting up IPv6 listener");
//         let v6 = net2::TcpBuilder::new_v6().unwrap();
//         v6.only_v6(true)
//             .unwrap()
//             .bind(addr)
//             .unwrap()
//             .listen(BACKLOG.into())
//             .unwrap()
//     } else {
//         info!("setting up IPv4 listener");
//         let v4 = net2::TcpBuilder::new_v4().unwrap();
//         v4.bind(addr).unwrap().listen(BACKLOG.into()).unwrap()
//     };

//     TcpListener::from_std(l).unwrap()
// }

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

async fn listen_and_serve<A>(addr: A)
where
    A: std::net::ToSocketAddrs,
{
    // let addr = std::net::ToSocketAddrs::to_socket_addrs(&addr)
    //     .unwrap()
    //     .last()
    //     .unwrap();

    let l = build_listener(addr).await;

    loop {
        let (stream, local) = l.accept().await.unwrap();
        info!("accepted connection on {}", local);
        let peer = stream.peer_addr().unwrap();
        info!("connection is from {}", peer);

        // let stream = TcpStream::from_std(stream).unwrap();

        // if let Err(err) = http1::Builder::new()
        //     .serve_connection(stream, service_fn(move |r| self::handle(r, local, peer)))
        //     .await
        // {
        //     error!("Error serving connection: {:?}", err);
        // }

        // runtime.spawn(async move {
        //     info!("spawning a new task");
        //     if let Err(err) = http1::Builder::new()
        //         .serve_connection(stream, service_fn(move |r| self::handle(r, local, peer)))
        //         .await
        //     {
        //         error!("Error serving connection: {:?}", err);
        //     }
        // });

        tokio::spawn(async move {
            info!("spawning a new task");
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(move |r| self::handle(r, local, peer)))
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
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

#[tokio::main]
async fn main() {
    env_logger::init();

    tokio::spawn(listen_and_serve("0.0.0.0:8080"));
    tokio::spawn(listen_and_serve("[::]:8080"));

    loop {}

    // let tasks = vec![
    //     tokio::spawn(listen_and_serve("0.0.0.0:8080")),
    //     tokio::spawn(listen_and_serve("[::]:8080")),
    // ];

    // futures::future::join_all(tasks).await;

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
