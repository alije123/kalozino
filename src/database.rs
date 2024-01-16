use ormlite::{postgres::PgConnection, Connection};

#[tracing::instrument]
pub async fn get_connection() -> PgConnection {
    PgConnection::connect(&std::env::var("DATABASE_URL").expect("missing DATABASE_URL"))
        .await
        .expect("failed to connect to database")
}
