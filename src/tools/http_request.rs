use serde::de::DeserializeOwned;

pub async fn get<T: DeserializeOwned>(url: &str) -> Result<T, reqwest::Error> {
    let res = reqwest::get(url).await?.json::<T>().await?;
    Ok(res)
}
