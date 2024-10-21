use reqwest::Client;
use serde::de::DeserializeOwned;

pub async fn get<T: DeserializeOwned>(url: &str) -> Result<T, reqwest::Error> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    let res = client.get(url).send().await?.json::<T>().await?;
    Ok(res)
}
