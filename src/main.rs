mod errors;
mod handlers;
mod models;
mod validators;

use crate::handlers::user_handlers::{
    create_users, delete_users, get_users, get_users_by_id, update_users,
};
use std::env;

use crate::errors::APILayerError;
use crate::handlers::signup_handlers::{signup, signup_verify};
use crate::models::app_state::AppState;
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
    let pool = PgPool::connect(&db_url).await.unwrap();
    let app_state = AppState { pool };

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
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
                            web::scope("/signup")
                                .route("", web::post().to(signup))
                                .route("/verify", web::get().to(signup_verify)),
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
