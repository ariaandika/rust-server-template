# Axum Server Template

contain:

- database connection with `sqlx`
- role authentication with its extractor
- custom error struct

# Authentication

use `Auth` extractor to authenticate

in `crate::entity::users` there is:

- role enum
- role structs
- `create_role_data()`

To create new role:

1. Create `Role` enum variant
1. create a struct, can be unit struct, for type parameter in `Auth` extractor
1. Implement the `RoleTrait`, only Role enum variant is required,
1. derive serde, currently `Auth` will eagerly attempt to
parse role data when extracting, so the struct need to derive serde
1. optionally, give the struct a field, this struct only lives in
session token, not in database
1. if field is given, implement the creation in `create_role_data()`

# `crate::entity::users::Auth`

axum extractor for authentication

example:

```rust
async fn must_login_any_role(auth: Auth) {}
async fn must_login_as_admin(auth: Auth<Admin>) {}
```

# TODO

- create cutom hook for authorization

