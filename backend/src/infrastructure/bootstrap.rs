use crate::infrastructure::database::DbPool;

/// If no admin account exists yet, creates one from the `ADMIN_EMAIL` /
/// `ADMIN_PASSWORD` env vars.
///
/// There is no public endpoint to create the first admin - every admin endpoint
/// requires an existing admin token - so without this, a fresh deployment would
/// have no way to log in at all.
pub async fn bootstrap_admin(pool: &DbPool) {
    let (has_admin,): (bool,) =
        sqlx::query_as("SELECT EXISTS (SELECT 1 FROM utilisateurs WHERE role = 'admin')")
            .fetch_one(pool)
            .await
            .expect("failed to check for an existing admin account");

    if has_admin {
        return;
    }

    let (Ok(email), Ok(password)) = (
        std::env::var("ADMIN_EMAIL"),
        std::env::var("ADMIN_PASSWORD"),
    ) else {
        tracing::warn!(
            "no admin account exists and ADMIN_EMAIL/ADMIN_PASSWORD are not set - \
             nobody will be able to log in until an admin is bootstrapped"
        );
        return;
    };

    let password_hash =
        bcrypt::hash(&password, bcrypt::DEFAULT_COST).expect("failed to hash admin password");

    sqlx::query(
        "INSERT INTO utilisateurs (exploitation_id, email, password_hash, role)
         VALUES (NULL, $1, $2, 'admin')",
    )
    .bind(&email)
    .bind(password_hash)
    .execute(pool)
    .await
    .expect("failed to create the bootstrap admin account");

    tracing::info!(%email, "created initial admin account from ADMIN_EMAIL/ADMIN_PASSWORD");
}
