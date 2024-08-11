use serde::Deserialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginFormData {
    pub username : String,
    pub password : String
}

#[derive(Deserialize)]
pub struct RegisterFormData {
    pub username : String,
    pub password : String
}


#[derive(FromRow, Deserialize)]
pub struct UserData{
    pub id : Uuid,
}

#[derive(FromRow, Deserialize)]
pub struct UserTokenData {
    pub user_id : Uuid
}