mod libs;
mod routes;

#[tokio::main]
async fn main() {

    let db_layer = axum::Extension(std::sync::Arc::new(libs::database::setup().await));

    let router = routes::routes().layer(db_layer);

    let tcp = tokio::net::TcpListener::bind("127.0.0.1:3000").await.expect("unable to bind port 3000");

    println!("Listening in http://127.0.0.1:3000...");
    axum::serve(tcp, router).await.expect("unable to start server");
}

