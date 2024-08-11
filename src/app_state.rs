use crate::{configurations::JwtSettings, db::DbPool};
use crate::utils::password_hasher::ServerPwdHasher;

pub struct AppState {
    pub pool : DbPool,
    pub jwt_settings : JwtSettings,
    pub pwd_hasher : ServerPwdHasher
}