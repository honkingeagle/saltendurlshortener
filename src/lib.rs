mod commands;

use axum::{
    routing::get, 
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Redirect
};
use sqlx::{Pool, Postgres, Row};
use std::sync::Arc;

type SharedState = Arc<AppState>;

pub struct AppState {
    pub pool: Pool<Postgres>,
    pub teloxide_token: String,
    pub prod_url: String,
}

impl AppState {
    pub fn new(pool: Pool<Postgres>, teloxide_token: String, prod_url: String) -> AppState {
        AppState {
            pool,
            teloxide_token,
            prod_url,
        }
    }
}

pub struct BackgroundServices {
    pub state: SharedState,
    pub router: Router,
}

impl BackgroundServices {
    pub fn new(state: SharedState, router: Router) -> BackgroundServices {
        BackgroundServices { state, router }
    }
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BackgroundServices {
    async fn bind(mut self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let router = axum::Server::bind(&addr).serve(self.router.into_make_service());

        tokio::select!(
            _ = router => (),
            _ =  commands::run(self.state) => ()
        );

        Ok(())
    }
}

async fn redirect(State(state): State<SharedState>, Path(nanoid): Path<String>) -> Result<Redirect, StatusCode> {
    let prod_url = &state.prod_url;
    let generated_url = format!("{prod_url}/{nanoid}");
    let query = sqlx::query("SELECT real_url FROM urls WHERE generated_url = $1")
    .bind(generated_url)
    .fetch_one(&state.pool)
    .await;
    match query {
        Ok(pg_row) => {
            let url: String = pg_row.get("real_url");
            Ok(Redirect::to(&url))
        },
        Err(_) => Err(StatusCode::NOT_FOUND)
    }
}

pub fn create_router(state: SharedState) -> Router {
    let router = Router::new()
        .route("/:nanoid", get(redirect))
        .with_state(state);

    router
}
