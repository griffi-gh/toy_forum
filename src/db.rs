use rocket::serde::Serialize;
use rocket_db_pools::{Database, Connection};
use sqlx::{self, PgPool, Row};
use argon2::{self, Config as ArgonConfig};
use rand::{Rng, thread_rng};
use crate::consts::{EMAIL_REGEX, PASSWORD_REGEX, USERNAME_REGEX};
use time::PrimitiveDateTime;

#[derive(Serialize, sqlx::Type, Default)]
#[serde(crate="rocket::serde", rename_all = "lowercase")]
#[sqlx(type_name = "role_type", rename_all = "lowercase")]
pub enum UserRole {
  Banned,
  Unverified,
  #[default]
  User,
  Moderator,
  Admin
}

#[derive(Serialize)]
#[serde(crate="rocket::serde")]
pub struct User {
  user_id: i32,
  username: String,
  email: String,
  password_hash: String,
  //These are not serialized until i find a workaround
  #[serde(skip_serializing)]
  created_on: PrimitiveDateTime,
  #[serde(skip_serializing)]
  last_activity: PrimitiveDateTime,
  //-------------------------------------------------
  user_role: UserRole,
  token: String,
}

#[derive(Database)]
#[database("main")]
pub struct MainDatabase(PgPool);
impl MainDatabase {
  /// Returns token
  pub async fn register(mut db: Connection<Self>, email: &str, username: &str, password: &str) -> Result<String, &'static str> {
    //Validate email, username and password
    if !EMAIL_REGEX.is_match(email) {
      return Err("Invalid email");
    }
    if !USERNAME_REGEX.is_match(username) {
      return Err("Invalid username");
    }
    if !PASSWORD_REGEX.is_match(password) {
      return Err("Invalid password");
    }
  
    //Check if username was used before
    //TODO this is inefficient
    let email_used: bool = sqlx::query("SELECT not COUNT(*) = 0 FROM users WHERE email = $1 LIMIT 1")
      .bind(&email)
      .fetch_one(&mut *db).await
      .unwrap().try_get(0).unwrap();
    if email_used {
      return Err("This email address is already in use");
    }
  
    //Register user
    let mut salt = [0u8; 16];
    thread_rng().fill(&mut salt);
    let password_hash = argon2::hash_encoded(password.as_bytes(), &salt[..], &ArgonConfig::default()).unwrap();
    let token = {
      let mut data = [0u8; 16];
      thread_rng().fill(&mut data);
      base64::encode_config(data, base64::URL_SAFE)
    };
    debug_assert!(token.len() == 24, "Invalid token length");
    sqlx::query("INSERT INTO users (username, email, password_hash, token) VALUES($1, $2, $3, $4);")
      .bind(&username)
      .bind(&email)
      .bind(&password_hash)
      .bind(&token)
      .execute(&mut *db).await
      .unwrap(); //handle error?
    Ok(token)
  }  

  /// Returns token
  pub async fn login(mut db: Connection<Self>, email: &str, password: &str) -> Result<String, &'static str> {
    //Verify stuff
    if !EMAIL_REGEX.is_match(email) {
      return Err("Invalid email");
    }
    if !PASSWORD_REGEX.is_match(password) {
      return Err("Invalid password");
    }
    //Perform query
    let row = sqlx::query("SELECT password_hash, token FROM users WHERE email = $1")
      .bind(&email)
      .fetch_optional(&mut *db).await
      .unwrap();
    //Check if user exists
    let row = match row {
      Some(row) => row,
      None => { return Err("User doesn't exist"); }
    };
    //Get info from the row
    let (hashed_password, token): (String, String) = (row.get(0), row.get(1));
    //Check hash (assuming it's is in valid format)
    match argon2::verify_encoded(&password, hashed_password.as_bytes()).unwrap() { 
      true => Ok(token),
      false => Err("Incorrect password")
    }
  }

  /// Returns user id
  pub async fn get_token_user(mut db: Connection<Self>, token: &str) -> Option<u32> {
    let result = sqlx::query("SELECT user_id FROM users WHERE token = $1")
      .bind(token)
      .fetch_optional(&mut *db).await
      .unwrap();
    result.map(|row| row.get(0))
  }

  pub async fn get_user(mut db: Connection<Self>, user_id: u32) -> Option<User> {
    let result = sqlx::query("SELECT user_id, username, email, password_hash, created_on, last_activity, user_role, token FROM users WHERE user_id = $1")
      .bind(user_id as i32)
      .fetch_optional(&mut *db).await
      .unwrap();
    result.map(|row| User {
      user_id: row.get(0),
      username: row.get(1),
      email: row.get(2),
      password_hash: row.get(3),
      created_on: row.get(4),
      last_activity: row.get(5),
      user_role: row.get(6),
      token: row.get(7),
    })
  }

  pub async fn get_user_by_token(mut db: Connection<Self>, token: &str) -> Option<User> {
    let result = sqlx::query("SELECT user_id, username, email, password_hash, created_on, last_activity, user_role, token FROM users WHERE token = $1")
      .bind(token)
      .fetch_optional(&mut *db).await
      .unwrap();
    result.map(|row| User {
      user_id: row.get(0),
      username: row.get(1),
      email: row.get(2),
      password_hash: row.get(3),
      created_on: row.get(4),
      last_activity: row.get(5),
      user_role: row.get(6),
      token: row.get(7),
    })
  }
}
