mod discord_notifier;
mod encoding_fix;
mod ics_generator;
mod models;
mod sap_client;
mod state;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};

use crate::state::AppState;
use chrono::{Datelike, TimeZone, Utc};

fn build_sap_url(base_url: &str) -> String {
    let now = Utc::now();

    let start_date = if now.month() == 1 {
        Utc.with_ymd_and_hms(now.year() - 1, 12, 1, 0, 0, 0)
            .unwrap()
    } else {
        Utc.with_ymd_and_hms(now.year(), now.month() - 1, 1, 0, 0, 0)
            .unwrap()
    };

    let mut end_month = now.month() + 5;
    let mut end_year = now.year();
    if end_month > 12 {
        end_month -= 12;
        end_year += 1;
    }
    let end_date = Utc
        .with_ymd_and_hms(end_year, end_month, 28, 23, 59, 59)
        .unwrap();

    let start_timestamp = start_date.timestamp();
    let end_timestamp = end_date.timestamp();

    let mut url = base_url.to_string();

    if let Some(query_start) = url.find('?') {
        let (base, query) = url.split_at(query_start);
        let params: Vec<&str> = query[1..]
            .split('&')
            .filter(|p| !p.starts_with("start=") && !p.starts_with("end="))
            .collect();

        url = if params.is_empty() {
            base.to_string()
        } else {
            format!("{}?{}", base, params.join("&"))
        };
    }

    let separator = if url.contains('?') { "&" } else { "?" };
    format!(
        "{}{}start={}&end={}",
        url, separator, start_timestamp, end_timestamp
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    match dotenvy::dotenv() {
        Ok(path) => info!("Loaded .env from: {:?}", path),
        Err(e) => error!("Failed to load .env: {}", e),
    }

    let sap_base_url = std::env::var("SAP_URL").expect("SAP_URL environment variable not set");
    let sap_url = build_sap_url(&sap_base_url);
    let cookie = std::env::var("SAP_COOKIE").expect("SAP_COOKIE environment variable not set");
    let discord_webhook = std::env::var("DISCORD_WEBHOOK_URL").ok();
    let poll_interval_secs = std::env::var("POLL_INTERVAL_SECONDS")
        .expect("POLL_INTERVAL_SECONDS environment variable not set")
        .parse::<u64>()
        .expect("POLL_INTERVAL_SECONDS must be a valid number");
    let port = std::env::var("PORT")
        .expect("PORT environment variable not set")
        .parse::<u16>()
        .expect("PORT must be a valid port number");

    info!("Starting SAP to ICS service");
    info!("Poll interval: {} seconds", poll_interval_secs);
    info!("Server port: {}", port);
    info!("Fetching events from 1 month ago to 5 months in the future");

    let state = Arc::new(AppState::new(
        sap_url.clone(),
        cookie.clone(),
        discord_webhook.clone(),
    ));

    // Initial silent fetch to populate state and avoid spam on first start
    info!("Performing initial fetch...");
    match sap_client::fetch_events(&state.sap_url, &state.cookie).await {
        Ok(events) => {
            info!("Initial fetch: found {} events", events.len());
            state.update_events(events).await;
        }
        Err(e) => {
            error!("Initial fetch failed: {}", e);
        }
    }

    {
        let state_clone = state.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(poll_interval_secs));
            loop {
                interval.tick().await;
                if let Err(e) = poll_and_update(&state_clone).await {
                    error!("Polling error: {}", e);
                }
            }
        });
    }

    let app = Router::new()
        .route("/calendar.ics", get(serve_calendar))
        .route("/health", get(health_check))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Listening on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn poll_and_update(state: &Arc<AppState>) -> anyhow::Result<()> {
    info!("Fetching events from SAP...");

    let events = sap_client::fetch_events(&state.sap_url, &state.cookie).await?;
    info!("Fetched {} events", events.len());

    let changes = state.update_events(events).await;

    if !changes.is_empty() {
        info!("Detected {} changes", changes.len());
        if let Some(webhook_url) = &state.discord_webhook
            && let Err(e) = discord_notifier::send_notification(webhook_url, &changes).await
        {
            error!("Failed to send Discord notification: {}", e);
        }
    }

    Ok(())
}

async fn serve_calendar(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let ics_content = state.get_ics_calendar().await;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/calendar; charset=utf-8")],
        ics_content,
    )
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
