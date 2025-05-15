use diesel::{Connection, PgConnection, SqliteConnection};
#[cfg(feature = "async")]
use diesel_async::{AsyncConnection, AsyncPgConnection, sync_connection_wrapper::SyncConnectionWrapper};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use env_logger::Builder;
use rstest::fixture;
#[cfg(not(feature = "async"))]
use testcontainers::{Container, runners::SyncRunner};
#[cfg(feature = "async")]
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers::{GenericImage, ImageExt, core::WaitFor};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
pub const POSTGRES_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations-psql");

#[fixture]
pub fn logger() {
    let _ = Builder::new()
        .filter(None, log::LevelFilter::Info)
        .filter(Some("test_versioning"), log::LevelFilter::Debug)
        .write_style(env_logger::WriteStyle::Always)
        .is_test(true)
        .try_init();
}

#[cfg(not(feature = "async"))]
#[fixture]
pub fn sqlite() -> SqliteConnection {
    let mut connection = SqliteConnection::establish(":memory:").unwrap();
    log::info!("run migration scripts");
    connection.run_pending_migrations(MIGRATIONS).unwrap();
    connection
}


#[cfg(feature = "async")]
#[fixture]
pub async fn sqlite() -> SyncConnectionWrapper<SqliteConnection> {
    let mut connection = SqliteConnection::establish(":memory:").unwrap();
    let connection = tokio::task::spawn_blocking(move || {
        log::info!("run migration scripts");
        connection.run_pending_migrations(MIGRATIONS).unwrap();
        connection
    })
    .await
    .unwrap();
    SyncConnectionWrapper::new(connection)
}
#[cfg(not(feature = "async"))]
#[allow(dead_code)]
pub struct TestDatabase {
    pub container: Container<GenericImage>,
    pub conn: PgConnection,
}

#[cfg(feature = "async")]
#[allow(dead_code)]
pub struct TestDatabase {
    pub container: ContainerAsync<GenericImage>,
    pub conn: AsyncPgConnection,
}

#[cfg(not(feature = "async"))]
#[fixture]
pub fn postgres() -> TestDatabase {
    log::info!("starting testdatabase container");
    let container = GenericImage::new("postgres", "17.4")
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_exposed_port(testcontainers::core::ContainerPort::Tcp(5432))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "testdb")
        .with_env_var("POSTGRES_DB", "testdb")
        .start()
        .expect("failedto start postgresql");
    log::info!("importing database schema");
    let dburl = format!(
        "postgres://postgres:testdb@{}:{}/testdb",
        container.get_host().unwrap(),
        container.get_host_port_ipv4(5432).unwrap()
    );
    log::info!("connecting to database for migration update: {}", dburl);
    let mut connection = PgConnection::establish(&dburl).unwrap();
    log::info!("run migration scripts");
    connection
        .run_pending_migrations(POSTGRES_MIGRATIONS)
        .unwrap();
    log::info!("database successful started");
    TestDatabase {
        container,
        conn: connection,
    }
}

#[cfg(feature = "async")]
#[fixture]
pub async fn postgres() -> TestDatabase {
    log::info!("starting testdatabase container");
    let container = GenericImage::new("postgres", "17.4")
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_exposed_port(testcontainers::core::ContainerPort::Tcp(5432))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "testdb")
        .with_env_var("POSTGRES_DB", "testdb")
        .start()
        .await
        .expect("failedto start postgresql");
    log::info!("importing database schema");
    let dburl = format!(
        "postgres://postgres:testdb@{}:{}/testdb",
        container.get_host().await.unwrap(),
        container.get_host_port_ipv4(5432).await.unwrap()
    );
    log::info!("connecting to database for migration update: {}", dburl);
    let mut connection = PgConnection::establish(&dburl).unwrap();
    log::info!("run migration scripts");
    connection
        .run_pending_migrations(POSTGRES_MIGRATIONS)
        .unwrap();
    log::info!("database successful started");
    let connection = AsyncPgConnection::establish(&dburl).await.unwrap();
    TestDatabase {
        container,
        conn: connection,
    }
}
