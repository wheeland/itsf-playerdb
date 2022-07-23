use reqwest::Client;
use scraper::Html;

async fn get(url: &str, headers: &[(&str, &str)]) -> Result<String, reqwest::Error> {
    let client = Client::builder().cookie_store(true).build()?;

    let mut request = client.get(url);
    for header in headers {
        request = request.header(header.0, header.1);
    }

    request.send().await?.text().await
}

pub async fn download(url: &str, headers: &[(&str, &str)]) -> Result<String, String> {
    Ok(get(url, headers).await.map_err(|err| err.to_string())?)
}

pub async fn download_html(url: &str) -> Result<Html, String> {
    let body = download(url, &[]).await?;
    Ok(Html::parse_document(&body))
}
