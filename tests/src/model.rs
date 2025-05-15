use diesel::prelude::*;
#[cfg(feature = "sync")]
use diesel_versioning::Versioned;
#[cfg(feature = "async")]
use diesel_versioning::VersionedAsync;

use crate::schema::{self};

#[cfg(feature = "sync")]
#[derive(Queryable, Selectable, AsChangeset, Debug, PartialEq, Identifiable, Clone, Versioned)]
#[diesel(table_name = schema::simple)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite, diesel::pg::Pg))]
pub struct SimpleEntry {
    pub id: i32,
    #[version]
    pub version: i32,
    pub body: String,
}

#[cfg(feature = "async")]
#[derive(Queryable, Selectable, AsChangeset, Debug, PartialEq, Identifiable, Clone, VersionedAsync)]
#[diesel(table_name = schema::simple)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite, diesel::pg::Pg))]
pub struct SimpleEntry {
    pub id: i32,
    #[version]
    pub version: i32,
    pub body: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = schema::simple)]
pub struct NewSimpleEntry {
    pub body: String,
}
