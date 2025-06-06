mod common;
mod mongo_raw;
mod server;

use common::{models::World, random_id};
use mongo_raw::database::{
    DatabaseConnection, find_world_by_id, find_worlds, update_worlds,
};

use common::{
    get_env,
    utils::{Params, parse_params},
};
use rama::http::{
    StatusCode,
    service::web::{Router, extract::Query, response::IntoResponse},
};
use std::time::Duration;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(feature = "simd-json")]
use common::simd_json::Json;
#[cfg(not(feature = "simd-json"))]
use rama::http::service::web::response::Json;

use dotenv::dotenv;
use mongodb::{
    Client,
    options::{ClientOptions, Compressor},
};
use rand::{SeedableRng, rng, rngs::SmallRng};

async fn db(DatabaseConnection(db): DatabaseConnection) -> impl IntoResponse {
    let random_id = random_id(&mut rng());

    let world = find_world_by_id(db, random_id)
        .await
        .expect("world could not be found");

    (StatusCode::OK, Json(world))
}

async fn queries(
    DatabaseConnection(db): DatabaseConnection,
    Query(params): Query<Params>,
) -> impl IntoResponse {
    let q = parse_params(params);

    let mut rng = SmallRng::from_rng(&mut rng());
    let worlds = find_worlds(db, &mut rng, q).await;
    let results = worlds.expect("worlds could not be retrieved");

    (StatusCode::OK, Json(results))
}

async fn updates(
    DatabaseConnection(db): DatabaseConnection,
    Query(params): Query<Params>,
) -> impl IntoResponse {
    let q = parse_params(params);

    let mut rng = SmallRng::from_rng(&mut rng());

    let worlds = find_worlds(db.clone(), &mut rng, q)
        .await
        .expect("worlds could not be retrieved");
    let mut updated_worlds: Vec<World> = Vec::with_capacity(q);

    for mut world in worlds {
        world.random_number = random_id(&mut rng);
        updated_worlds.push(world);
    }

    update_worlds(db.clone(), updated_worlds.clone())
        .await
        .expect("could not update worlds");

    (StatusCode::OK, Json(updated_worlds.clone()))
}

fn main() {
    dotenv().ok();
    server::start_tokio(serve_app)
}

async fn serve_app() {
    let database_url: String = get_env("MONGODB_URL");
    let max_pool_size: u32 = get_env("MONGODB_MAX_POOL_SIZE");
    let min_pool_size: u32 = get_env("MONGODB_MIN_POOL_SIZE");

    let mut client_options = ClientOptions::parse(database_url).await.unwrap();

    // setup connection pool
    client_options.max_pool_size = Some(max_pool_size);
    client_options.min_pool_size = Some(min_pool_size);
    client_options.connect_timeout = Some(Duration::from_millis(200));

    // the server will select the algorithm it supports from the list provided by the driver
    client_options.compressors = Some(vec![
        Compressor::Snappy,
        Compressor::Zlib {
            level: Default::default(),
        },
        Compressor::Zstd {
            level: Default::default(),
        },
    ]);

    let client = Client::with_options(client_options).unwrap();
    let database = client.database("hello_world");

    let app = Router::new()
        .get("/db", db)
        .get("/queries", queries)
        .get("/updates", updates);

    server::serve(database, app, Some(8000)).await;
}
