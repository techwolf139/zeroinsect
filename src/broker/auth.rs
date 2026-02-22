use crate::storage::kv_store::KvStore;
use crate::storage::schema::UserProfile;
use crate::storage::schema::UserStatus;
use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::{SaltString, PasswordHash}};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

pub enum AuthResult {
    Success(String),
    InvalidCredentials,
    TokenExpired,
    UserNotFound,
}

pub struct Authenticator {
    store: KvStore,
    jwt_secret: Vec<u8>,
}

impl Authenticator {
    pub fn new(store: KvStore) -> Self {
        Self {
            store,
            jwt_secret: std::env::var("RMC_JWT_SECRET")
                .unwrap_or_else(|_| "rmc_secret_key_change_in_production".to_string())
                .into_bytes(),
        }
    }

    pub async fn register_user(&self, username: &str, password: &str) -> Result<String> {
        if let Some(_existing) = self.store.get_user(username)? {
            return Err(anyhow::anyhow!("User already exists"));
        }
        
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?
            .to_string();
        
        let user_id = Uuid::new_v4().to_string();
        let user = UserProfile {
            user_id: user_id.clone(),
            username: username.to_string(),
            password_hash,
            created_at: chrono::Utc::now().timestamp(),
            status: UserStatus::Offline,
        };
        
        self.store.save_user(&user)?;
        Ok(user_id)
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> AuthResult {
        match self.store.get_user(username) {
            Ok(Some(user)) => {
                let argon2 = Argon2::default();
                let parsed_hash = match PasswordHash::new(&user.password_hash) {
                    Ok(hash) => hash,
                    Err(_) => return AuthResult::InvalidCredentials,
                };
                
                match argon2.verify_password(password.as_bytes(), &parsed_hash) {
                    Ok(_) => AuthResult::Success(user.user_id),
                    Err(_) => AuthResult::InvalidCredentials,
                }
            }
            Ok(None) => AuthResult::UserNotFound,
            Err(_) => AuthResult::InvalidCredentials,
        }
    }

    pub async fn authenticate_with_token(&self, token: &str) -> AuthResult {
        let validation = Validation::default();
        let decoding_key = DecodingKey::from_secret(&self.jwt_secret);
        
        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                let user_id = token_data.claims.sub;
                
                match self.store.get_user(&user_id) {
                    Ok(Some(_)) => AuthResult::Success(user_id),
                    Ok(None) => AuthResult::UserNotFound,
                    Err(_) => AuthResult::InvalidCredentials,
                }
            }
            Err(_) => AuthResult::TokenExpired,
        }
    }

    pub async fn generate_token(&self, user_id: &str) -> Result<String> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::days(7);
        
        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };
        
        let header = Header::default();
        let encoding_key = EncodingKey::from_secret(&self.jwt_secret);
        
        Ok(encode(&header, &claims, &encoding_key)?)
    }
}
