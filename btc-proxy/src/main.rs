use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;

// Function to parse the QuickNode URL from command-line arguments
fn get_quicknode_url() -> String {
    // We skip the first argument, which is the program name
    let args: Vec<String> = env::args().skip(1).collect();

    // Check if the QuickNode URL is provided
    if args.len() < 1 {
        panic!("QuickNode URL not provided. Usage: <program> <quicknode_url>");
    }

    // Return the first argument as the QuickNode URL
    args[0].clone()
}

async fn forward_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // QuickNode URL from the command-line argument
    let quicknode_url = get_quicknode_url();

    let client = reqwest::Client::new();
    let forwarded_req = client.request(req.method().clone(), &quicknode_url).body(
        hyper::body::to_bytes(req.into_body())
            .await
            .unwrap()
            .to_vec(),
    );

    // Forward the request and get the response
    match forwarded_req.send().await {
        Ok(res) => {
            let body = res.text().await.unwrap();
            Ok(Response::new(Body::from(body)))
        }
        Err(_) => {
            let mut response = Response::new(Body::from("Internal Server Error"));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() {
    // The local address to bind to
    let addr = SocketAddr::from(([127, 0, 0, 1], 8332));

    let quicknode_url = get_quicknode_url();
    println!("QuickNode URL: {}", quicknode_url);

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(forward_request)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
