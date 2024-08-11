use crate::errors::AppError;
use argon2::PasswordVerifier;
use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};
use password_hash::{PasswordHasher, SaltString};

pub struct ServerPwdHasher;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait PwdHasher {
    async fn hash_password(&self, password : &str) -> Result<String, AppError>;
    async fn verify_password(&self, password : &str, hashed_password : &str) -> Result<(), AppError>;
}

#[async_trait]
impl PwdHasher for ServerPwdHasher {
    async fn hash_password(&self, password : &str) -> Result<String, AppError> {
        let pwd = password.to_string();
        tokio::task::spawn_blocking(move || {
            hash(&pwd)
        })
        .await
        .map_err(|_| AppError::UnexpectedError("failed spawn.".into()))?
    }

    async fn verify_password(&self, password : &str, hashed_password : &str) -> Result<(), AppError> {
        let hashed_pwd = hashed_password.to_string();
        let pwd = password.to_string();
        tokio::task::spawn_blocking(move || {
            verify(&pwd, &hashed_pwd)
        })
        .await
        .map_err(|_| AppError::UnexpectedError("failed spawn.".into()))?
    }
}

fn hash(password : &str) -> Result<String, AppError> {
    let params = argon2::Params::new(15000, 2, 1, None).unwrap();
    let salt = SaltString::generate(rand::thread_rng());
    let hasher = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    
    let result = hasher.hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::UnexpectedError(e.to_string()))?;

    Ok(result.to_string())
}

fn verify(password : &str, hashed_password : &str) -> Result<(), AppError> {
    let phc_string = argon2::PasswordHash::new(hashed_password)
        .map_err(|e| AppError::UnauthorizedError(e.to_string()))?;

    argon2::Argon2::default()
        .verify_password(password.as_bytes(), &phc_string)
        .map_err(|e| AppError::UnauthorizedError(e.to_string()))?;

    Ok(())
}
