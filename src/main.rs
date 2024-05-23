

#[tokio::main]
async fn main() {

    let router = routes();

    let tcp = tokio::net::TcpListener::bind("127.0.0.1:3000").await.expect("unable to bind port 3000");

    axum::serve(tcp, router).await.expect("unable to start server");
}

fn routes() -> axum::Router {
    axum::Router::new()
}

