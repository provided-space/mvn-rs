#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv::dotenv().expect("Failed to load .env");
    let database_url = std::env::var("DATABASE_URL").expect("Missing environment variable DATABASE_URL");

    println!("Running migrations in build script.");
    let pool = sqlx::mysql::MySqlPoolOptions::new().connect(database_url.as_str()).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    return Ok(());
}
