use crate::db::{DbContext, TxContext};
use crate::errors::AppError;
use crate::features::auth::domain::Credentials;
use crate::utils::password_hasher::PwdHasher;
use serde::Deserialize;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use uuid::{NoContext, Timestamp, Uuid};

use super::models::{UserData, UserTokenData};

#[derive(Deserialize, FromRow)]
struct ValidationResult {
    id: Uuid,
    password: String,
}

#[tracing::instrument(
    name = "Validating User Credentials",
    skip(credentials, db, pwd_hasher)
)]
pub async fn validate_credentials(
    credentials: &Credentials,
    db: &impl DbContext,
    pwd_hasher: &impl PwdHasher,
) -> Result<Uuid, AppError> {
    let query = sqlx::query_as(
        r#"
            SELECT id, password FROM users WHERE username = $1
        "#,
    )
    .bind(credentials.username.to_string());

    let result = db.fetch_optional::<ValidationResult>(query).await?;

    let (id, password) = match result {
        Some(data) => (data.id, data.password),
        None => return Err(AppError::UnauthorizedError("Invalid Username".into())),
    };

    pwd_hasher.verify_password(&credentials.password, &password).await?;

    Ok(id)
}

#[tracing::instrument(name = "Creating User", skip(credentials, db, pwd_hasher))]
pub async fn create_user(
    credentials: &Credentials,
    db: &impl DbContext,
    pwd_hasher: &impl PwdHasher,
) -> Result<Uuid, AppError> {
    let id = Uuid::new_v7(Timestamp::now(NoContext));

    let password = pwd_hasher.hash_password(&credentials.password).await?;

    let query = sqlx::query!(
        r#"
            INSERT INTO users (id, username, password, created_at)
            VALUES 
            ($1, $2, $3, now())
        "#,
        id,
        credentials.username,
        password
    );

    db.execute_query(query).await?;

    Ok(id)
}

#[tracing::instrument(name = "Fetching User by Id", skip(user_id, db))]
pub async fn get_user_by_id(user_id: Uuid, db: &impl DbContext) -> Result<UserData, AppError> {
    let query = sqlx::query_as(
        r#"
            SELECT id FROM users WHERE id = $1
        "#,
    )
    .bind(user_id);

    let result = db.fetch_optional::<UserData>(query).await?;

    match result {
        Some(data) => Ok(data),
        None => Err(AppError::NotFoundError("User was not found".into())),
    }
}

#[tracing::instrument(name = "Fetching User by Id", skip(rt, db))]
pub async fn get_user_tokens_by_token(
    rt: &str,
    db: &impl DbContext,
) -> Result<Option<UserTokenData>, AppError> {
    let query = sqlx::query_as(
        r#"
            SELECT user_id FROM user_tokens WHERE refresh_token = $1
        "#,
    )
    .bind(rt.to_string());

    let result = db.fetch_optional::<UserTokenData>(query).await?;

    Ok(result)
}

#[tracing::instrument(name = "Adding Refresh token", skip(token, user_id, db))]
pub async fn add_refresh_token_by_user_id(
    token: &str,
    user_id: Uuid,
    db: &impl DbContext,
) -> Result<(), AppError> {
    let id = Uuid::new_v7(Timestamp::now(NoContext));

    let query = sqlx::query!(
        r#"
            INSERT INTO user_tokens (id, refresh_token, user_id, created_at)
            VALUES
            ($1, $2, $3, now())
        "#,
        id,
        token,
        user_id
    );

    db.execute_query(query).await?;

    Ok(())
}

#[tracing::instrument(name = "Deleting refresh token by token", skip(token, db))]
pub async fn delete_refresh_token_by_token(
    token: &str,
    db: &impl DbContext,
) -> Result<(), AppError> {
    let query = sqlx::query!(
        r#"
            DELETE FROM user_tokens WHERE refresh_token = $1
        "#,
        token
    );

    db.execute_query(query).await?;

    Ok(())
}

#[tracing::instrument(name = "Deleting all refresh token by user id", skip(user_id, db))]
pub async fn delete_all_refresh_token_by_user_id(
    user_id: Uuid,
    db: &impl DbContext,
) -> Result<(), AppError> {
    let query = sqlx::query!(
        r#"
            DELETE FROM user_tokens WHERE user_id = $1
        "#,
        user_id
    );

    db.execute_query(query).await?;

    Ok(())
}

#[tracing::instrument(
    name = "Verifying User by Id",
    skip(user_id, tx)
)]
pub async fn verify_user_by_id_tx(
    user_id: Uuid,
    tx : &mut impl TxContext
) -> Result<Option<PgRow>, AppError>{
    let query = sqlx::query(
        r#"
            SELECT id FROM users WHERE id = $1
        "#
    )
    .bind(user_id);

    let result = tx.fetch_optional(query).await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use fake::{faker::internet::en::Password, Fake};
    use uuid::Uuid;

    use crate::{
        db::MockDbContext,
        features::auth::{
            domain::Credentials,
            repository::{validate_credentials, ValidationResult},
        },
        utils::{password_hasher::MockPwdHasher, randomizer::generate_random_string},
    };

    fn generate_test_user() -> Credentials {
        Credentials {
            username: generate_random_string(12),
            password: generate_random_string(12),
        }
    }

    #[tokio::test]
    async fn a_valid_credentials_is_accepted() {
        let credentials = generate_test_user();

        let mut db_mock = MockDbContext::new();
        let mut pwd_mock = MockPwdHasher::new();

        db_mock.expect_fetch_optional().times(1).returning(|_| {
            Ok(Some(ValidationResult {
                id: Uuid::new_v4(),
                password: Password(1..12).fake(),
            }))
        });

        pwd_mock
            .expect_verify_password()
            .times(1)
            .returning(|_, _| Ok(()));

        let result = validate_credentials(&credentials, &db_mock, &pwd_mock).await;

        assert_ok!(result);
    }

    #[tokio::test]
    async fn an_invalid_username_is_rejected() {
        let credentials = generate_test_user();

        let mut db_mock = MockDbContext::new();
        let mut pwd_mock = MockPwdHasher::new();

        db_mock
            .expect_fetch_optional::<ValidationResult>()
            .times(1)
            .returning(|_| Ok(None));

        pwd_mock
            .expect_verify_password()
            .times(0)
            .returning(|_, _| Ok(()));

        let result = validate_credentials(&credentials, &db_mock, &pwd_mock).await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn an_invalid_password_is_rejected() {
        let credentials = generate_test_user();

        let mut db_mock = MockDbContext::new();
        let mut pwd_mock = MockPwdHasher::new();

        db_mock
            .expect_fetch_optional::<ValidationResult>()
            .times(1)
            .returning(|_| {
                Ok(Some(ValidationResult {
                    id: Uuid::new_v4(),
                    password: Password(1..12).fake(),
                }))
            });

        pwd_mock
            .expect_verify_password()
            .times(1)
            .returning(|x, y| {
                if x != y {
                    return Err(crate::errors::AppError::UnexpectedError(
                        "Invalid Password".into(),
                    ));
                }
                Ok(())
            });

        let result = validate_credentials(&credentials, &db_mock, &pwd_mock).await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn a_db_error_would_cancel_validating_credentials() {
        let credentials = generate_test_user();

        let mut db_mock = MockDbContext::new();
        let mut pwd_mock = MockPwdHasher::new();

        db_mock
            .expect_fetch_optional::<ValidationResult>()
            .times(1)
            .returning(|_| {
                return Err(crate::errors::AppError::DbError(
                    sqlx::error::Error::ColumnNotFound("Error".into()),
                ));
            });

        pwd_mock
            .expect_verify_password()
            .times(0)
            .returning(|x, y| {
                if x != y {
                    return Err(crate::errors::AppError::UnexpectedError(
                        "Invalid Password".into(),
                    ));
                }
                Ok(())
            });

        let result = validate_credentials(&credentials, &db_mock, &pwd_mock).await;

        assert_err!(result);
    }
}
