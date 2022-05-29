use scraper::Html;

async fn get(url: &str) -> Result<String, reqwest::Error> {
    Ok(reqwest::get(url).await?.text().await?)
}

pub async fn download(url: &str) -> Result<Html, String> {
    let body = get(url).await.map_err(|err| err.to_string())?;
    Ok(Html::parse_document(&body))
}
