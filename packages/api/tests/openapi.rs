//! The committed OpenAPI snapshot is the contract the web app's client is generated from.
//! Regenerate it with `just contracts` (sets `UPDATE_CONTRACTS=1`). Needs no database.

use std::{fs, path::PathBuf, sync::Arc};

use object_store::aws::AmazonS3Builder;
use sqlx::postgres::PgPoolOptions;
use vitro_api::{
    config::{object_store::ObjectStore, pg_database::PgDatabase},
    router::root::app,
};

fn snapshot_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/openapi.json")
}

#[tokio::test]
async fn openapi_snapshot_matches_the_api() {
    // Both handles are lazy or offline: building the router opens no connection.
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://unused@127.0.0.1:1/unused")
        .unwrap();
    let client = AmazonS3Builder::new()
        .with_endpoint("http://127.0.0.1:1")
        .with_region("us-east-1")
        .with_access_key_id("unused")
        .with_secret_access_key("unused")
        .with_bucket_name("unused")
        .with_allow_http(true)
        .build()
        .unwrap();
    let (_, api) = app(
        Arc::new(PgDatabase::from_pool(pool)),
        Arc::new(ObjectStore::from_client(client)),
    );
    let current = format!("{}\n", api.to_pretty_json().unwrap());

    let path = snapshot_path();
    if std::env::var_os("UPDATE_CONTRACTS").is_some() {
        fs::write(&path, &current).unwrap();
        return;
    }
    let committed = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        committed == current,
        "packages/contracts/openapi.json is out of date with the API; run `just contracts`"
    );
}
