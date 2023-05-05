mod errors;
mod handlers;
mod mailer;
mod models;
mod validators;

use crate::handlers::user_handlers::{
    create_users, delete_users, get_users, get_users_by_id, update_users,
};
use actix_cors::Cors;
use actix_session::config::PersistentSession;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use std::env;

use crate::errors::APILayerError;
use crate::handlers::auth_handlers::{signin, signout, signup, signup_verify};
use crate::models::app_state::AppState;
use actix_web::cookie::time::Duration;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use opentelemetry::global;
use opentelemetry::runtime::TokioCurrentThread;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    init_telemetry();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");
    let app_state = AppState { pool };

    let key = actix_web::cookie::Key::generate();
    let session_store_url = env::var("SESSION_STORE_URL").expect("SESSION_STORE_URL must be set");
    let session_key = env::var("SESSION_KEY").expect("SESSION_KEY must be set");
    let redis_store = RedisSessionStore::new(&session_store_url)
        .await
        .expect("Failed to connect to Redis");

    let cors_allowed_origin = env::var("CLIENT_URL").expect("CLIENT_URL must be set");
    let cors_max_age = env::var("CORS_MAX_AGE")
        .expect("CORS_MAX_AGE must be set")
        .parse::<usize>()
        .expect("CORS_MAX_AGE must be a valid usize");

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(
                Cors::default()
                    .allowed_origin(&cors_allowed_origin)
                    .max_age(cors_max_age),
            )
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), key.clone())
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::minutes(5)),
                    )
                    .cookie_name(session_key.to_string())
                    .build(),
            )
            .app_data(
                web::JsonConfig::default()
                    .limit(4096)
                    .error_handler(|err, _| {
                        actix_web::Error::from(errors::AppError::Deserialization(
                            APILayerError::new(err.to_string()),
                        ))
                    }),
            )
            .app_data(web::Data::new(app_state.clone()))
            .service(
                web::scope("/api").service(
                    web::scope("/v1")
                        .service(
                            web::scope("/users")
                                .route("", web::get().to(get_users))
                                .route("", web::post().to(create_users))
                                .route("/{id}", web::get().to(get_users_by_id))
                                .route("/{id}", web::put().to(update_users))
                                .route("/{id}", web::delete().to(delete_users)),
                        )
                        .service(
                            web::scope("/auth")
                                .service(
                                    web::scope("/signup")
                                        .route("", web::post().to(signup))
                                        .route("/verify", web::get().to(signup_verify)),
                                )
                                .route("/signin", web::post().to(signin))
                                .route("/signout", web::post().to(signout)),
                        ),
                ),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    global::shutdown_tracer_provider();
    Ok(())
}

fn init_telemetry() {
    let app_name = "rust-backend";

    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(app_name)
        .install_batch(TokioCurrentThread)
        .expect("Failed to install OpenTelemetry tracer .");
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let formatting_layer = BunyanFormattingLayer::new(app_name.into(), std::io::stdout);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(telemetry)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.")
}
