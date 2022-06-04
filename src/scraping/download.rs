use scraper::Html;

async fn get(url: &str) -> Result<String, reqwest::Error> {
    Ok(reqwest::get(url).await?.text().await?)
}

pub async fn download(url: &str) -> Result<String, String> {
    Ok(get(url).await.map_err(|err| err.to_string())?)
}

pub async fn download_html(url: &str) -> Result<Html, String> {
    let body = download(url).await?;
    Ok(Html::parse_document(&body))
}
