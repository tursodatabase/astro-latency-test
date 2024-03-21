#![deny(warnings)]

use libsql::{Builder, Database};
use std::env;
use std::sync::Arc;
use std::time::SystemTime;
use warp::Filter;

async fn handler(db: Arc<Database>) -> Result<String, warp::Rejection> {
    let before = SystemTime::now();
    db.sync().await.or(Err(warp::reject::reject()))?;
    let time = SystemTime::now()
        .duration_since(before)
        .or(Err(warp::reject::reject()))?
        .as_millis();
    let conn = db.connect().or(Err(warp::reject::reject()))?;
    let mut result = conn
        .query("SELECT COUNT(*) FROM auth", ())
        .await
        .or(Err(warp::reject::reject()))?;
    let count: i64 = *result
        .next()
        .await
        .or(Err(warp::reject::reject()))?
        .unwrap()
        .get_value(0)
        .or(Err(warp::reject::reject()))?
        .as_integer()
        .unwrap();
    Ok(format!(
        "Sync took {} ms. Current number of rows is {}",
        time, count
    ))
}

#[tokio::main]
async fn main() {
    let url = env::var("LIBSQL_URL").expect("LIBSQL_URL must be set");
    let token = env::var("LIBSQL_AUTH_TOKEN").unwrap_or_default();

    println!("Connecting to: {}", url);
    println!("Using empty token: {}", token.is_empty());

    let db = Arc::new(
        Builder::new_remote_replica("local.db", url, token)
            .build()
            .await
            .expect("failed to connect to db"),
    );
    let db_map = warp::any().map(move || db.clone());
    // Match any request and return hello world!
    let routes = warp::any()
        .and(warp::post())
        .and(warp::path!("sync"))
        .and(db_map)
        .and_then(handler);

    warp::serve(routes)
        // ipv6 + ipv6 any addr
        .run(([0, 0, 0, 0, 0, 0, 0, 0], 8080))
        .await;
}
