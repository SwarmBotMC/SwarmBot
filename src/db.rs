use sqlx::{Error, PgPool};
use sqlx::postgres::PgPoolOptions;
use tokio::time::Duration;
use tokio_stream::StreamExt;
use crate::bootstrap::mojang::Mojang;
use crate::bootstrap::User;

pub struct Db {
    pool: PgPool,
}

#[derive(sqlx::FromRow, Debug)]
pub struct ValidCredentials {
    pub access_token: String,
    pub client_token: String,
    pub username: String,
    pub uuid: String,
    pub password: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct CachedUser {
    pub email: String,
    pub access_token: String,
    pub client_token: String,
    pub username: String,
    pub uuid: String,
    pub password: String,
}

pub enum Credentials {
    Invalid,
    Valid(ValidCredentials),
}

pub struct InvalidDbUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

pub struct ValidDbUser<'a> {
    pub email: &'a str,
    pub access_token: &'a str,
    pub client_token: &'a str,
    pub username: &'a str,
    pub uuid: &'a str,
    pub password: &'a str,
}

impl Db {
    pub async fn init() -> Db {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://andrew:password@127.0.0.1/mcbot").await.unwrap();

        sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS valid_users (
            email VARCHAR PRIMARY KEY NOT NULL,
            access_token VARCHAR NOT NULL,
            client_token VARCHAR NOT NULL,
            username VARCHAR NOT NULL,
            uuid VARCHAR NOT NULL,
            password VARCHAR NOT NULL
         )
        "#).execute(&pool).await.unwrap();

        sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS invalid_users (
            email VARCHAR PRIMARY KEY NOT NULL,
            password VARCHAR NOT NULL
         )
        "#).execute(&pool).await.unwrap();


        Db {
            pool
        }
    }

    pub async fn update_valid(&self, user: ValidDbUser<'_>) {
        // let conn = self.pool.acquire().await.unwrap();
        sqlx::query("INSERT INTO valid_users VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (email) DO UPDATE SET (access_token, client_token, username, uuid, password) = (EXCLUDED.access_token, EXCLUDED.client_token, EXCLUDED.username, EXCLUDED.uuid, EXCLUDED.password) ")
            .bind(user.email)
            .bind(user.access_token)
            .bind(user.client_token)
            .bind(user.username)
            .bind(user.uuid)
            .bind(user.password)
            .execute(&self.pool).await.unwrap();
    }

    pub async fn update_invalid(&self, user: InvalidDbUser<'_>) {
        // let conn = self.pool.acquire().await.unwrap();
        sqlx::query("INSERT INTO invalid_users VALUES ($1,$2) ON CONFLICT (email) DO UPDATE SET password = EXCLUDED.password")
            .bind(user.email)
            .bind(user.password)
            .execute(&self.pool).await.unwrap();
    }

    pub async fn get_credentials(&self, email: &str) -> Option<Credentials> {
        let invalid_res = sqlx::query("SELECT * FROM invalid_users where email = $1")
            .bind(email)
            .fetch_optional(&self.pool).await.unwrap();

        if invalid_res.is_some() {
            return Some(Credentials::Invalid);
        }

        let res = sqlx::query_as::<_, ValidCredentials>("SELECT access_token, client_token, username, uuid, password FROM valid_users where email = $1")
            .bind(email)
            .fetch_optional(&self.pool).await.unwrap();

        match res {
            None => None,
            Some(res) => Some(Credentials::Valid(res))
        }
    }

    pub async fn obtain_users(&self, amount: usize) -> Vec<CachedUser> {
        let invalid_res = sqlx::query_as::<_, CachedUser>("SELECT * FROM valid_users")
            .fetch(&self.pool);

        let vec: Result<Vec<_>, _> = invalid_res.take(amount).collect().await;
        vec.unwrap()
    }

    pub async fn update_users(&self, users: &[User], mojang: &Mojang) {
        for user in users {
            let creds = self.get_credentials(&user.email).await;
            if creds.is_none() {
                match mojang.authenticate(&user.email, &user.password).await {
                    Ok(res) => {
                        self.update_valid(ValidDbUser {
                            email: &user.email,
                            access_token: &res.access_token,
                            client_token: &res.client_token,
                            username: &res.username,
                            password: &user.password,
                            uuid: &res.uuid.to_string(),
                        }).await;

                        println!("updating {}", user.email);
                    }
                    Err(err) => {
                        println!("could not authenticate {} reason {:?}", user.email, err);
                        self.update_invalid(InvalidDbUser {
                            email: &user.email,
                            password: &user.password,
                        }).await;
                    }
                };
                tokio::time::sleep(Duration::from_secs(5)).await;
            } else {
                println!("have creds for {}", user.email);
            }
        }
    }
}
