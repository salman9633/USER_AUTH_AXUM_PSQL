use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::sync::Arc;
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

#[derive(Debug, Deserialize)]
struct UserRequest {
    email: String,
    username: String,
    password: String,
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
