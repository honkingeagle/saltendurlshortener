use saltendurlshortener::{AppState, BackgroundServices};
use std::{process, sync::Arc};
use sqlx::PgPool;
use shuttle_secrets::SecretStore;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_secrets::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] pool: PgPool
) -> Result<BackgroundServices, shuttle_runtime::Error> {

    sqlx::migrate!().run(&pool).await.unwrap_or_else(|err| {
        eprintln!("Unable to migrate sql files: {err}");
        process::exit(1);
    });

    let teloxide_token = secrets.get("TELOXIDE_TOKEN").unwrap();
    let prod_url = secrets.get("PROD_URL").unwrap();

    let state = Arc::new(AppState::new(pool, teloxide_token, prod_url));

    let router = saltendurlshortener::create_router(Arc::clone(&state));

    Ok(BackgroundServices::new(Arc::clone(&state), router))
}
