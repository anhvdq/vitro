//! Server-side session behavior. Needs `DATABASE_URL` to a Postgres that can create databases.

use std::sync::Arc;

use sqlx::PgPool;
use vitro_api::{
    config::pg_database::PgDatabase,
    repository::{
        session_repository::{SessionRepository, SessionRepositoryImpl},
        user_repository::{UserRepository, UserRepositoryImpl},
    },
    service::auth_service::{AuthServiceImpl, AuthServiceTrait},
    util::api_response::ServiceError,
};

const EMAIL: &str = "owner@example.com";
const PASSWORD: &str = "correct horse battery";

fn service(pool: &PgPool, session_ttl_secs: u64) -> AuthServiceImpl {
    let db = Arc::new(PgDatabase::from_pool(pool.clone()));
    let users: UserRepository = Arc::new(UserRepositoryImpl::new(Arc::clone(&db)));
    let sessions: SessionRepository = Arc::new(SessionRepositoryImpl::new(db));
    AuthServiceImpl::new(users, sessions, session_ttl_secs)
}

/// `ServiceError` has no `Debug`, so unwrap by hand.
fn ok<T>(result: Result<T, ServiceError>) -> T {
    match result {
        Ok(value) => value,
        Err(_) => panic!("expected the service call to succeed"),
    }
}

async fn session_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM sessions")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn a_login_token_identifies_its_user_and_an_unknown_token_does_not(pool: PgPool) {
    let auth = service(&pool, 3600);
    let signed_up = ok(auth.signup(EMAIL, PASSWORD, None).await);

    let info = ok(auth.extract_auth_info(&signed_up.access_token).await);

    assert_eq!(info.user_id, signed_up.user.id);
    assert!(auth.extract_auth_info("not-a-real-token").await.is_err());
}

#[sqlx::test(migrations = "./migrations")]
async fn logout_revokes_the_token_immediately(pool: PgPool) {
    // The reason for server-side sessions: a stolen or retired token must stop working
    // at once, not when a JWT happens to expire.
    let auth = service(&pool, 3600);
    ok(auth.signup(EMAIL, PASSWORD, None).await);
    let login = ok(auth.login(EMAIL, PASSWORD).await);
    let info = ok(auth.extract_auth_info(&login.access_token).await);

    ok(auth.logout(&info).await);

    assert!(auth.extract_auth_info(&login.access_token).await.is_err());
}

#[sqlx::test(migrations = "./migrations")]
async fn logging_out_one_device_leaves_other_devices_signed_in(pool: PgPool) {
    let auth = service(&pool, 3600);
    ok(auth.signup(EMAIL, PASSWORD, None).await);
    let laptop = ok(auth.login(EMAIL, PASSWORD).await);
    let phone = ok(auth.login(EMAIL, PASSWORD).await);
    assert_ne!(laptop.access_token, phone.access_token);

    let laptop_info = ok(auth.extract_auth_info(&laptop.access_token).await);
    ok(auth.logout(&laptop_info).await);

    assert!(auth.extract_auth_info(&laptop.access_token).await.is_err());
    assert!(auth.extract_auth_info(&phone.access_token).await.is_ok());
}

#[sqlx::test(migrations = "./migrations")]
async fn an_expired_session_is_rejected_and_removed(pool: PgPool) {
    let auth = service(&pool, 0);
    let signed_up = ok(auth.signup(EMAIL, PASSWORD, None).await);
    assert_eq!(session_count(&pool).await, 1);

    assert!(
        auth.extract_auth_info(&signed_up.access_token)
            .await
            .is_err()
    );

    assert_eq!(
        session_count(&pool).await,
        0,
        "dead sessions do not pile up"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn the_raw_token_is_never_stored(pool: PgPool) {
    // A leaked sessions table must not hand out working tokens.
    let auth = service(&pool, 3600);
    let signed_up = ok(auth.signup(EMAIL, PASSWORD, None).await);

    let stored: Vec<u8> = sqlx::query_scalar("SELECT token_hash FROM sessions")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_ne!(stored, signed_up.access_token.as_bytes());
    assert_eq!(stored.len(), 32, "a SHA-256 digest");
}

#[sqlx::test(migrations = "./migrations")]
async fn deleting_a_user_ends_their_sessions(pool: PgPool) {
    let auth = service(&pool, 3600);
    let signed_up = ok(auth.signup(EMAIL, PASSWORD, None).await);

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(signed_up.user.id)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(session_count(&pool).await, 0);
    assert!(
        auth.extract_auth_info(&signed_up.access_token)
            .await
            .is_err()
    );
}
