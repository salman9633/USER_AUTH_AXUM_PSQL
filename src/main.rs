use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use tokio_postgres::{Client, NoTls};

#[tokio::main]
async fn main() {
    let client=db().await;
    let app=app(client);
    let listner=tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server Listening to {}",listner.local_addr().unwrap());
    axum::serve(listner,app).await.unwrap()

}

fn app(client: Client)->Router{
    Router::new().route("/",get(||async { "Hello World" }))
        .with_state(Arc::new(client))
}

async  fn db()->Client{
    let conn_str="host=localhost port=5432 user=postgres password=postgres dbname=axum_user_managment";
    let (client,connection)=tokio_postgres::connect(&conn_str,NoTls).await.unwrap();

    tokio::spawn(async move{
        if let Err(e)=connection.await{
            eprintln!("Connection Err: {}",e)
        }
    });

    client
}
