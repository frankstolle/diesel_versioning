//! # Diesel-Versioning
//!
//! Diesel-Versioning implements [optimistic locking](https://en.wikipedia.org/wiki/Optimistic_concurrency_control) for [Diesel](https://diesel.rs).
//! This is achieved by an additional field on every entity, which should be support optimistic
//! locking.
//!
//! The entity must have implemented [`diesel::AsChangeset`] and [`diesel::Identifiable`] to implement [`Versioned`]. You can
//! use the provided derive macro.
//!
//! ```ignore
//! use diesel::AsChangeset;
//! use diesel::Identifiable;
//! use diesel_versioning::Versioned;
//!
//! #[derive(AsChangeset, Identifiable, Versioned)]
//! #[diesel(table_name = schema::users)]
//! #[diesel(check_for_backend(diesel::pg::Pg))]
//! pub struct User {
//!    pub id: i32,
//!    #[version]
//!    pub version: i32,
//!    pub body: String,
//! }
//! ```
//!
//! Currently only integer values are supported as version field.
//!
//! If you use the feature-flag `async`, you have to use [`VersionedAsync`] instead of
//! [`Versioned`].

use diesel::Connection;
use diesel::{AsChangeset, result::Error};

#[cfg(feature = "async")]
use diesel_async::AsyncConnection;

///
/// Trait that must be implemented by an entity, to support optimitsic locking. You would use the
/// provided derive macro.
///
/// If you want to use async connection, use [`VersionedAsync`] instead.
///
pub trait Versioned<CONN, DB>: AsChangeset
where
    CONN: Connection<Backend = DB>,
    DB: diesel::backend::Backend,
{
    ///
    /// Updates the entity using the provided connection. The version field will be checked and
    /// incremented.
    ///
    fn update_versioned(&mut self, conn: &mut CONN) -> Result<(), Error>;

    ///
    /// Deletes the entity using the provided connection. The version field will be checked.
    ///
    fn delete_versioned(&mut self, conn: &mut CONN) -> Result<(), Error>;
}

#[cfg(feature = "async")]
///
/// Trait that must be implemented by an entity, to support optimitsic locking. You would use the
/// provided derive macro.
///
/// This is the async version of [`Versioned`]
///
pub trait VersionedAsync<CONN, DB>: AsChangeset
where
    CONN: AsyncConnection<Backend = DB>,
    DB: diesel::backend::Backend,
{
    ///
    /// Updates the entity using the provided connection. The version field will be checked and
    /// incremented.
    ///
    fn update_versioned(&mut self, conn: &mut CONN) -> impl Future<Output = Result<(), Error>>;

    ///
    /// Deletes the entity using the provided connection. The version field will be checked.
    ///
    fn delete_versioned(&mut self, conn: &mut CONN) -> impl Future<Output = Result<(), Error>>;
}

pub use diesel_versioning_derives::Versioned;
#[cfg(feature = "async")]
pub use diesel_versioning_derives::VersionedAsync;
