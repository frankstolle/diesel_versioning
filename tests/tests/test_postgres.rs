use std::error::Error;

#[cfg(feature = "sync")]
use diesel::RunQueryDsl;
use diesel::{
    ExpressionMethods, SelectableHelper, query_dsl::methods::FilterDsl,
    result::DatabaseErrorKind,
};
#[cfg(feature = "async")]
use diesel_async::RunQueryDsl;
#[cfg(feature = "sync")]
use diesel_versioning::Versioned;
#[cfg(feature = "async")]
use diesel_versioning::VersionedAsync;
use diesel_versioning_async_example::{
    model::{NewSimpleEntry, SimpleEntry},
    schema::{self},
};
use fixtures::{TestDatabase, logger, postgres};
use rstest::rstest;

mod fixtures;

#[cfg(feature = "sync")]
#[rstest]
#[test]
fn it_should_update_an_entry(
    _logger: (),
    mut postgres: TestDatabase,
) -> Result<(), Box<dyn Error>> {
    let conn = &mut postgres.conn;
    let entry = NewSimpleEntry {
        body: "initial text".to_owned(),
    };
    let mut entry: SimpleEntry = diesel::insert_into(schema::simple::table)
        .values((&entry, schema::simple::version.eq(1)))
        .returning(SimpleEntry::as_returning())
        .get_result(conn)?;

    entry.body = "updated text".to_owned();
    entry.update_versioned(conn)?;

    let entry = schema::simple::table
        .filter(schema::simple::id.eq(entry.id))
        .first::<SimpleEntry>(conn)?;
    assert_eq!("updated text", &entry.body);
    assert_eq!(2, entry.version);
    Ok(())
}

#[cfg(feature = "async")]
#[rstest]
#[tokio::test]
async fn it_should_update_an_entry(
    _logger: (),
    #[future] postgres: TestDatabase,
) -> Result<(), Box<dyn Error>> {
    let conn = &mut postgres.await.conn;
    let entry = NewSimpleEntry {
        body: "initial text".to_owned(),
    };
    let mut entry: SimpleEntry = diesel::insert_into(schema::simple::table)
        .values((&entry, schema::simple::version.eq(1)))
        .returning(SimpleEntry::as_returning())
        .get_result(conn)
        .await?;

    entry.body = "updated text".to_owned();
    entry.update_versioned(conn).await?;

    let entry = schema::simple::table
        .filter(schema::simple::id.eq(entry.id))
        .first::<SimpleEntry>(conn)
        .await?;
    assert_eq!("updated text", &entry.body);
    assert_eq!(2, entry.version);
    Ok(())
}

#[cfg(feature = "sync")]
#[rstest]
#[test]
fn it_should_fail_if_update_an_entry_twice(
    _logger: (),
    mut postgres: TestDatabase,
) -> Result<(), Box<dyn Error>> {
    let conn = &mut postgres.conn;
    let entry = NewSimpleEntry {
        body: "initial text".to_owned(),
    };
    let mut first_entry: SimpleEntry = diesel::insert_into(schema::simple::table)
        .values((&entry, schema::simple::version.eq(1)))
        .returning(SimpleEntry::as_returning())
        .get_result(conn)?;
    let mut second_entry = first_entry.clone();
    first_entry.body = "updated text on first".to_owned();
    first_entry.update_versioned(conn)?;

    second_entry.body = "updated text on second".to_owned();
    let result = second_entry.update_versioned(conn);

    match result {
        Ok(_) => panic!("expected error on update"),
        Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::CheckViolation, info)) => {
            assert!(
                info.message().starts_with("optimistic locking:"),
                "expected CheckViolation Error message start with 'optimistic locking:', but found: {}",
                info.message()
            );
        }
        Err(err) => {
            panic!("got unexpected error: {:?}", err);
        }
    };
    let entry = schema::simple::table
        .filter(schema::simple::id.eq(first_entry.id))
        .first::<SimpleEntry>(conn)?;
    assert_eq!("updated text on first", &entry.body);
    assert_eq!(2, entry.version);
    Ok(())
}

#[cfg(feature = "async")]
#[rstest]
#[tokio::test]
async fn it_should_fail_if_update_an_entry_twice(
    _logger: (),
    #[future] postgres: TestDatabase,
) -> Result<(), Box<dyn Error>> {
    let conn = &mut postgres.await.conn;
    let entry = NewSimpleEntry {
        body: "initial text".to_owned(),
    };
    let mut first_entry: SimpleEntry = diesel::insert_into(schema::simple::table)
        .values((&entry, schema::simple::version.eq(1)))
        .returning(SimpleEntry::as_returning())
        .get_result(conn)
        .await?;
    let mut second_entry = first_entry.clone();
    first_entry.body = "updated text on first".to_owned();
    first_entry.update_versioned(conn).await?;

    second_entry.body = "updated text on second".to_owned();
    let result = second_entry.update_versioned(conn).await;

    match result {
        Ok(_) => panic!("expected error on update"),
        Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::CheckViolation, info)) => {
            assert!(
                info.message().starts_with("optimistic locking:"),
                "expected CheckViolation Error message start with 'optimistic locking:', but found: {}",
                info.message()
            );
        }
        Err(err) => {
            panic!("got unexpected error: {:?}", err);
        }
    };
    let entry = schema::simple::table
        .filter(schema::simple::id.eq(first_entry.id))
        .first::<SimpleEntry>(conn)
        .await?;
    assert_eq!("updated text on first", &entry.body);
    assert_eq!(2, entry.version);
    Ok(())
}

#[cfg(feature = "sync")]
#[rstest]
#[test]
fn it_should_delete_an_entry(
    _logger: (),
    mut postgres: TestDatabase,
) -> Result<(), Box<dyn Error>> {
    use diesel::OptionalExtension;

    let conn = &mut postgres.conn;
    let entry = NewSimpleEntry {
        body: "initial text".to_owned(),
    };
    let mut entry: SimpleEntry = diesel::insert_into(schema::simple::table)
        .values((&entry, schema::simple::version.eq(1)))
        .returning(SimpleEntry::as_returning())
        .get_result(conn)?;

    entry.delete_versioned(conn)?;

    let entry = schema::simple::table
        .filter(schema::simple::id.eq(entry.id))
        .first::<SimpleEntry>(conn)
        .optional()?;
    if let Some(entry) = entry {
        panic!("expected none, but got: {:?}", entry);
    }
    Ok(())
}

#[cfg(feature = "async")]
#[rstest]
#[tokio::test]
async fn it_should_delete_an_entry(
    _logger: (),
    #[future] postgres: TestDatabase,
) -> Result<(), Box<dyn Error>> {
    use diesel::OptionalExtension;

    let conn = &mut postgres.await.conn;
    let entry = NewSimpleEntry {
        body: "initial text".to_owned(),
    };
    let mut entry: SimpleEntry = diesel::insert_into(schema::simple::table)
        .values((&entry, schema::simple::version.eq(1)))
        .returning(SimpleEntry::as_returning())
        .get_result(conn).await?;

    entry.delete_versioned(conn).await?;

    let entry = schema::simple::table
        .filter(schema::simple::id.eq(entry.id))
        .first::<SimpleEntry>(conn).await
        .optional()?;
    if let Some(entry) = entry {
        panic!("expected none, but got: {:?}", entry);
    }
    Ok(())
}

#[cfg(feature = "sync")]
#[rstest]
#[test]
fn it_should_fail_if_delete_a_updated_entry(
    _logger: (),
    mut postgres: TestDatabase,
) -> Result<(), Box<dyn Error>> {
    let conn = &mut postgres.conn;
    let entry = NewSimpleEntry {
        body: "initial text".to_owned(),
    };
    let mut first_entry: SimpleEntry = diesel::insert_into(schema::simple::table)
        .values((&entry, schema::simple::version.eq(1)))
        .returning(SimpleEntry::as_returning())
        .get_result(conn)?;
    let mut second_entry = first_entry.clone();
    first_entry.body = "updated text on first".to_owned();
    first_entry.update_versioned(conn)?;

    let result = second_entry.delete_versioned(conn);

    match result {
        Ok(_) => panic!("expected error on delete"),
        Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::CheckViolation, info)) => {
            assert!(
                info.message().starts_with("optimistic locking:"),
                "expected CheckViolation Error message start with 'optimistic locking:', but found: {}",
                info.message()
            );
        }
        Err(err) => {
            panic!("got unexpected error: {:?}", err);
        }
    };
    let entry = schema::simple::table
        .filter(schema::simple::id.eq(first_entry.id))
        .first::<SimpleEntry>(conn)?;
    assert_eq!("updated text on first", &entry.body);
    assert_eq!(2, entry.version);
    Ok(())
}

#[cfg(feature = "async")]
#[rstest]
#[tokio::test]
async fn it_should_fail_if_delete_a_updated_entry(
    _logger: (),
    #[future] postgres: TestDatabase,
) -> Result<(), Box<dyn Error>> {
    let conn = &mut postgres.await.conn;
    let entry = NewSimpleEntry {
        body: "initial text".to_owned(),
    };
    let mut first_entry: SimpleEntry = diesel::insert_into(schema::simple::table)
        .values((&entry, schema::simple::version.eq(1)))
        .returning(SimpleEntry::as_returning())
        .get_result(conn).await?;
    let mut second_entry = first_entry.clone();
    first_entry.body = "updated text on first".to_owned();
    first_entry.update_versioned(conn).await?;

    let result = second_entry.delete_versioned(conn).await;

    match result {
        Ok(_) => panic!("expected error on delete"),
        Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::CheckViolation, info)) => {
            assert!(
                info.message().starts_with("optimistic locking:"),
                "expected CheckViolation Error message start with 'optimistic locking:', but found: {}",
                info.message()
            );
        }
        Err(err) => {
            panic!("got unexpected error: {:?}", err);
        }
    };
    let entry = schema::simple::table
        .filter(schema::simple::id.eq(first_entry.id))
        .first::<SimpleEntry>(conn).await?;
    assert_eq!("updated text on first", &entry.body);
    assert_eq!(2, entry.version);
    Ok(())
}
