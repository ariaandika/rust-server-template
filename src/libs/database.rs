
#[macro_export]
macro_rules! enum_encode {
    ($id:ident) => {
        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $id {
            fn decode(value: <sqlx::Postgres as sqlx::database::HasValueRef<'r>>::ValueRef) -> std::result::Result<Self, sqlx::error::BoxDynError> {
                Ok(Self::from_str(value.as_str()?)?)
            }
        }
        impl sqlx::Type<sqlx::Postgres> for $id {
            fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("TEXT")
            }
        }
    }
}

pub async fn setup() -> sqlx::PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .connect(&dotenvy::var("DATABASE_URL").expect("DATABASE_URL env is required"))
        .await
        .expect("unable to connect to database")
}

