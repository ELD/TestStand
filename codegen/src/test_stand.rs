use devise::{
    ext::SpanDiagnosticExt, DeriveGenerator, FromMeta, MapperBuild, Spanned, Support,
    ValidatorBuild,
};
use quote::{quote, quote_spanned};

const ONE_UNNAMED_FIELD: &str = "struct must have exactly one unnamed field";
const ONE_DATABASE_ATTR: &str = "struct must have exactly one `#[database(\"name\")]` attribute";
const ONE_MIGRATIONS_ATTR: &str = "struct must have exactly one `#[migration(\"path\")]` attribute";
const _ONE_CONNECTION_TYPE_ATTR: &str =
    "struct must have exactly one `#[connection_type(type)]` attribute";

#[derive(Debug, FromMeta)]
struct DatabaseAttribute {
    #[meta(naked)]
    name: String,
}

#[derive(Debug, FromMeta)]
struct MigrationPathAttribute {
    #[meta(naked)]
    path: String,
}

pub(crate) fn derive_test_stand(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    DeriveGenerator::build_for(input, quote!(impl test_stand::TestStand))
        .support(Support::TupleStruct)
        .validator(ValidatorBuild::new().struct_validate(|_, s| {
            if s.fields.len() == 1 {
                Ok(())
            } else {
                Err(s.span().error(ONE_UNNAMED_FIELD))
            }
        }))
        .outer_mapper(quote!(#[rocket::async_trait]))
        .inner_mapper(MapperBuild::new().try_struct_map(|_, s| {
            let db_name = DatabaseAttribute::one_from_attrs("database", &s.attrs)?
                .map(|attr| attr.name)
                .ok_or_else(|| s.span().error(ONE_DATABASE_ATTR))?;
            let migration_path =
                MigrationPathAttribute::one_from_attrs("migration_path", &s.attrs)?
                    .map(|attr| attr.path)
                    .ok_or_else(|| s.span().error(ONE_MIGRATIONS_ATTR))?;

            let fairing_name = format!("'{}' Test Stand", db_name);

            let pool_type = match &s.fields {
                syn::Fields::Unnamed(f) => &f.unnamed[0].ty,
                _ => unreachable!("Support::TupleStruct"),
            };

            Ok(quote_spanned! { pool_type.span() =>
                const NAME: &'static str = #db_name;
                const MIGRATION_PATH: &'static str = #migration_path;

                type TestStand = #pool_type;

                fn test_stand() -> test_stand::Initializer<Self> {
                    test_stand::Initializer::with_name(#fairing_name)
                }
            })
        }))
        .to_tokens()
}
