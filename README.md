# Diesel-Versioning

Diesel-Versioning implements [optimistic locking](https://en.wikipedia.org/wiki/Optimistic_concurrency_control) for [Diesel](https://diesel.rs).
This is achieved by an additional field on every entity, which should be support optimistic
locking.

## Getting started

The entity must have implemented `diesel::AsChangeset` and `diesel::Identifiable` to implement `Versioned`. You can
use the provided derive macro.

```rust
use diesel::AsChangeset;
use diesel::Identifiable;
use diesel_versioning::Versioned;

#[derive(AsChangeset, Identifiable, Versioned)]
#[diesel(table_name = schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
   pub id: i32,
   #[version]
   pub version: i32,
   pub body: String,
}
```

Currently only integer values are supported as version field.

If you use the feature-flag `async`, you have to use `VersionedAsync` instead of `Versioned`.

## License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
