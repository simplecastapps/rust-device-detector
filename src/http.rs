use anyhow::Result;

use hyper::http::StatusCode;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;

use crate::device_detector;
async fn serve_request(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/detect") => {
            // TODO prevent pulling entire body into memory in case of abuse
            let body = hyper::body::to_bytes(req.into_body()).await?;
            let body = String::from_utf8(body.to_vec())?;
            let detection = device_detector::parse(&body, None).unwrap();
            let response = serde_json::to_string(&detection.to_value())?;

            Ok(Response::new(Body::from(response)))
        }

        (&Method::GET, "/health") => Ok(Response::new("OK\n".into())),

        _route => {
            let err = "valid routes:\n  POST /detect with a body containing referer\n  GET  /health for heartbeat";
            eprintln!("{}", err);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(err))
        }
    }
    .map_err(|x| x.into())
}

pub async fn server(port: u16) {
    // TODO make ip configurable
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    eprintln!("listing on {}", addr);

    let make_svc = make_service_fn(|_conn| {
        let service = service_fn(move |req| serve_request(req));

        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    };
}
