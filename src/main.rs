use axum::{Router, http::HeaderMap, routing::get};
use error::Result;
use eyre::Context;
use ical::generator::IcalCalendar;
use reqwest::header;
use tokio::signal;

mod error;

async fn get_calendar() -> Result<IcalCalendar> {
    let url = std::env::var("ICS_URL").wrap_err("environment variable 'ICS_URL' not set")?;
    let res = reqwest::get(url).await?;

    let calendar = ical::IcalParser::new(res.bytes().await?.as_ref())
        .collect::<Vec<_>>()
        .remove(0)?;

    Ok(calendar)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt().init();
    std::env::var("ICS_URL").wrap_err("environment variable 'ICS_URL' not set")?;

    let app = Router::new().route("/", get(handler));

    tracing::info!("Starting server");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let a = axum::serve(listener, app);

    if std::env::var("IN_DOCKER").unwrap_or_default() == "true" {
        a.with_graceful_shutdown(shutdown_signal()).await?;
    } else {
        a.await?;
    }

    Ok(())
}

pub async fn handler() -> Result<(HeaderMap, String)> {
    tracing::info!("handler called");
    let calender = get_calendar().await?;

    let ev = &calender as &dyn ical::generator::Emitter;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/calender".parse().unwrap());
    Ok((headers, ev.generate()))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
