use crate::models::SAPEvent;
use anyhow::Result;

pub async fn fetch_events(url: &str, cookie: &str) -> Result<Vec<SAPEvent>> {
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .header("Cookie", cookie)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to fetch events: HTTP {} - {}", status, body);
    }

    let text = response.text().await?;
    let events: Vec<SAPEvent> = serde_json::from_str(&text)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {} - Response: {}", e, text))?;

    let fixed_events: Vec<SAPEvent> = events
        .into_iter()
        .map(|event| event.fix_encoding())
        .collect();

    Ok(fixed_events)
}
