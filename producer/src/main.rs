#![deny(warnings)]

use libsql::{Builder, Database};
use std::env;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::Filter;

async fn handler(count: u32, db: Arc<Database>) -> Result<String, warp::Rejection> {
    let conn = db.connect().or(Err(warp::reject::reject()))?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .or(Err(warp::reject::reject()))?
        .as_millis();
    for i in 0..count {
        conn.execute(
            &format!(
                "INSERT INTO auth (user, token) VALUES ('user-{}-{}', 'token-{}-{}')",
                ts, i, ts, i
            ),
            (),
        )
        .await
        .or(Err(warp::reject::reject()))?;
    }
    Ok(format!("Produced {} rows", count))
}

#[tokio::main]
async fn main() {
    let url = env::var("LIBSQL_URL").expect("LIBSQL_URL must be set");
    let token = env::var("LIBSQL_AUTH_TOKEN").unwrap_or_default();

    println!("Connecting to: {}", url);
    println!("Using empty token: {}", token.is_empty());

    let db = Arc::new(
        Builder::new_remote(url, token)
            .build()
            .await
            .expect("failed to connect to db"),
    );
    let db_map = warp::any().map(move || db.clone());
    // Match any request and return hello world!
    let routes = warp::any()
        .and(warp::post())
        .and(warp::path!("produce" / u32))
        .and(db_map)
        .and_then(handler);

    warp::serve(routes)
        // ipv6 + ipv6 any addr
        .run(([0, 0, 0, 0, 0, 0, 0, 0], 8080))
        .await;
}
