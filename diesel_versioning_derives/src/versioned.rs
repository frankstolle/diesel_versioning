use proc_macro_error::abort_call_site;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

use crate::model::Model;

pub fn derive(item: DeriveInput, impl_async: bool) -> Result<TokenStream> {
    let model = Model::from_item(&item)?;

    let struct_name = &item.ident;
    let version_fieldname = match model.version_fieldname() {
        Some(fieldname) => fieldname,
        None => {
            abort_call_site!("no field is attributed with #[version]");
        }
    };
    let table_name = &model.table_names()[0];
    let backends = model.backends();

    let code: Vec<TokenStream> = backends
        .iter()
        .map(|backend| {
            if !impl_async {
                quote! {
                    #[automatically_derived]
                    impl<CONN> Versioned<CONN, #backend> for #struct_name
                    where
                        CONN: diesel::Connection<Backend = #backend>,
                        i32: diesel::serialize::ToSql<diesel::sql_types::Integer, #backend>,
                    {
                        fn update_versioned(&mut self, conn: &mut CONN) -> std::result::Result<(), diesel::result::Error>
                        {
                            let expected_version = self.#version_fieldname;
                            //FIXME: Frank: increment anpassen
                            self.#version_fieldname += 1;
                            let q = diesel::update(&*self)
                                .set(&*self)
                                .filter(#table_name::#version_fieldname.eq(expected_version));
                            let updated_rows = q.execute(conn)?;
                            if updated_rows != 1 {
                                return Err(diesel::result::Error::DatabaseError(
                                    diesel::result::DatabaseErrorKind::CheckViolation,
                                    Box::new(format!(
                                        "optimistic locking: updated {} rows. expected 1",
                                        updated_rows
                                    )),
                                ));
                            }
                            Ok(())
                        }

                        fn delete_versioned(&mut self, conn: &mut CONN) -> std::result::Result<(), diesel::result::Error>
                        {
                            let expected_version = self.#version_fieldname;
                            let q = diesel::delete(&*self).filter(#table_name::#version_fieldname.eq(expected_version));
                            let updated_rows = q.execute(conn)?;
                            if updated_rows != 1 {
                                return Err(diesel::result::Error::DatabaseError(
                                    diesel::result::DatabaseErrorKind::CheckViolation,
                                    Box::new(format!(
                                        "optimistic locking: deleted {} rows. expected 1",
                                        updated_rows
                                    )),
                                ));
                            }
                            Ok(())
                        }
                    }
                }
            }
            else {
            #[cfg(not(feature = "async"))]{
abort_call_site!("missing async feature to use VersionedAsync");
                }

            #[cfg(feature = "async")]{
                quote! {
                    #[automatically_derived]
                    impl<CONN> VersionedAsync<CONN, #backend> for #struct_name
                    where
                        CONN: diesel_async::AsyncConnection<Backend = #backend> + Send,
                        i32: diesel::serialize::ToSql<diesel::sql_types::Integer, #backend>,
                    {
                        async fn update_versioned(&mut self, conn: &mut CONN) -> std::result::Result<(), diesel::result::Error>
                        {
                            let expected_version = self.#version_fieldname;
                            //FIXME: Frank: increment anpassen
                            self.#version_fieldname += 1;
                            let q = diesel::update(&*self)
                                .set(&*self)
                                .filter(#table_name::#version_fieldname.eq(expected_version));
                            let updated_rows = diesel_async::RunQueryDsl::execute(q, conn).await?;
                            if updated_rows != 1 {
                                return Err(diesel::result::Error::DatabaseError(
                                    diesel::result::DatabaseErrorKind::CheckViolation,
                                    Box::new(format!(
                                        "optimistic locking: updated {} rows. expected 1",
                                        updated_rows
                                    )),
                                ));
                            }
                            Ok(())
                        }

                        async fn delete_versioned(&mut self, conn: &mut CONN) -> std::result::Result<(), diesel::result::Error>
                        {
                            let expected_version = self.#version_fieldname;
                            let q = diesel::delete(&*self).filter(#table_name::#version_fieldname.eq(expected_version));
                            let updated_rows = diesel_async::RunQueryDsl::execute(q, conn).await?;
                            if updated_rows != 1 {
                                return Err(diesel::result::Error::DatabaseError(
                                    diesel::result::DatabaseErrorKind::CheckViolation,
                                    Box::new(format!(
                                        "optimistic locking: deleted {} rows. expected 1",
                                        updated_rows
                                    )),
                                ));
                            }
                            Ok(())
                        }

                    }
                } }
            } 
        })
        .collect();

    Ok(quote! {
        #(#code)*
    })
}
