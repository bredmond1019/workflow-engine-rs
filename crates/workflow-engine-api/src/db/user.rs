use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::users;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub salt: String,
    pub is_active: bool,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub salt: String,
    pub is_active: bool,
    pub role: String,
}

impl User {
    /// Verify password against stored hash
    pub fn verify_password(&self, password: &str) -> bool {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(&self.salt);
        let computed_hash = format!("{:x}", hasher.finalize());
        
        computed_hash == self.password_hash
    }
    
    /// Check if user account is active and can login
    pub fn can_login(&self) -> bool {
        self.is_active
    }
}

impl NewUser {
    /// Create a new user with hashed password
    pub fn new(username: String, email: String, password: &str, role: String) -> Self {
        use sha2::{Sha256, Digest};
        use rand::{thread_rng, Rng};
        
        // Generate random salt
        let salt: String = (0..32)
            .map(|_| thread_rng().gen_range(0x20..0x7F) as u8 as char)
            .collect();
        
        // Hash password with salt
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(&salt);
        let password_hash = format!("{:x}", hasher.finalize());
        
        Self {
            username,
            email,
            password_hash,
            salt,
            is_active: true,
            role,
        }
    }
}

/// User repository for database operations
pub struct UserRepository<'a> {
    connection: &'a mut diesel::PgConnection,
}

impl<'a> UserRepository<'a> {
    pub fn new(connection: &'a mut diesel::PgConnection) -> Self {
        Self { connection }
    }
    
    /// Find user by username
    pub fn find_by_username(&mut self, username: &str) -> Result<Option<User>, diesel::result::Error> {
        use crate::db::schema::users::dsl;
        
        users::table
            .filter(dsl::username.eq(username))
            .first::<User>(self.connection)
            .optional()
    }
    
    /// Find user by email
    pub fn find_by_email(&mut self, email: &str) -> Result<Option<User>, diesel::result::Error> {
        use crate::db::schema::users::dsl;
        
        users::table
            .filter(dsl::email.eq(email))
            .first::<User>(self.connection)
            .optional()
    }
    
    /// Create a new user
    pub fn create(&mut self, new_user: NewUser) -> Result<User, diesel::result::Error> {
        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(self.connection)
    }
    
    /// Update user's last login timestamp
    pub fn update_last_login(&mut self, user_id: Uuid) -> Result<(), diesel::result::Error> {
        use crate::db::schema::users::dsl;
        
        diesel::update(users::table.filter(dsl::id.eq(user_id)))
            .set(dsl::last_login.eq(Some(Utc::now())))
            .execute(self.connection)?;
        
        Ok(())
    }
    
    /// Validate user credentials
    pub fn validate_credentials(&mut self, username: &str, password: &str) -> Result<Option<User>, diesel::result::Error> {
        if let Some(user) = self.find_by_username(username)? {
            if user.can_login() && user.verify_password(password) {
                // Update last login timestamp
                if let Err(e) = self.update_last_login(user.id) {
                    log::warn!("Failed to update last login for user {}: {}", username, e);
                }
                return Ok(Some(user));
            }
        }
        Ok(None)
    }
}