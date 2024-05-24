use axum::Router;
use axum::routing::*;


pub fn routes() -> Router {
    Router::new()
        .nest("/auth/v1", users())
}

fn users() -> Router {
    use crate::entity::users::query;
    Router::new()
        .route("/login", post(query::login))
        .route("/login/cookie", post(query::login_cookie))
        .route("/session", get(query::session))
        .route("/session/admin", get(query::admin))
}

