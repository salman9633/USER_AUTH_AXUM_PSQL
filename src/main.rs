use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_postgres::{Client, NoTls};

#[tokio::main]
async fn main() {
    let client = db().await;
    let app = app(client);
    let listner = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server Listening to {}", listner.local_addr().unwrap());
    axum::serve(listner, app).await.unwrap()
}

fn app(client: Client) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello World" }))
        .route("/user/signUp", post(signup))
        .route("/user/signIn", post(signin))
        .with_state(Arc::new(client))
}

async fn signup(
    State(client): State<Arc<Client>>,
    Json(user): Json<UserRequest>,
) -> impl IntoResponse {
    println!("{:?}", user);
    let hashed_password = bcrypt::hash(user.password, 10).unwrap();
    println!("{}", hashed_password);
    client
        .execute(
            "INSERT INTO users (username, password,email) VALUES ($1,$2,$3)",
            &[&user.username, &hashed_password, &user.email],
        )
        .await
        .unwrap();
    (StatusCode::OK, "Sign Up SuccessFull").into_response()
}

async fn signin(
    State(client): State<Arc<Client>>,
    Json(user): Json<UserRequest>,
) -> impl IntoResponse {
    let row = client
        .query(
            "SELECT * FROM users WHERE username = $1 OR email = $1",
            &[&user.username],
        )
        .await
        .unwrap();

    if row.is_empty() {
        return (StatusCode::UNAUTHORIZED, "User doesn't exist").into_response();
    }

    let hashed_password: String = row[0].get(2);

    let is_valid = bcrypt::verify(user.password, &hashed_password).unwrap();
    if is_valid {
        let username: String = row[0].get(1);
        let claim = Claim {
            sub: username,
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 60 * 60,
        };

        let token = encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret("SALMAN SECRET".as_bytes()),
        )
        .unwrap();
        (StatusCode::OK, {
            Json(LoginResponse {
                message: String::from("Success Fully Signed In"),
                token,
            })
        })
            .into_response()
    } else {
        (StatusCode::UNAUTHORIZED, "Incorrect Password").into_response()
    }
}

#[derive(Debug, Deserialize)]
struct UserRequest {
    email: Option<String>, //making email as optional for signin
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    message: String,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claim {
    sub: String,
    exp: u64,
}

async fn db() -> Client {
    let conn_str =
        "host=localhost port=5432 user=postgres password=postgres dbname=axum_user_managment";
    let (client, connection) = tokio_postgres::connect(&conn_str, NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection Err: {}", e)
        }
    });

    client
}
