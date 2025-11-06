use std::collections::HashMap;

use async_trait::async_trait;
use futures::channel::oneshot;
use reqwest::Url;
use wasm_bindgen_futures::spawn_local;

use polarity_lang_driver::{DriverError, FileSource};

#[derive(Default, Clone)]
pub struct FetchSource {
    client: reqwest::Client,
    cache: HashMap<Url, String>,
}

#[async_trait]
impl FileSource for FetchSource {
    async fn exists(&mut self, uri: &reqwest::Url) -> Result<bool, DriverError> {
        let url =
            get_base_url().join(uri.path()).map_err(|_| DriverError::FileNotFound(uri.clone()))?;

        if self.cache.contains_key(&url) {
            return Ok(true);
        }

        // Prefetch: To check whether the file exists, we have to actually try to fetch it.
        // To prevent redundant fetches later, we cache the result if successful.
        match fetch_text(self.client.clone(), url.clone()).await {
            Ok(text) => {
                self.cache.insert(url, text);
                Ok(true)
            }
            Err(DriverError::FileNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    fn register(&mut self, _uri: &reqwest::Url) -> bool {
        true
    }

    fn forget(&mut self, uri: &reqwest::Url) -> bool {
        if let Ok(url) = get_base_url().join(uri.path()) {
            self.cache.remove(&url);
        }
        true
    }

    async fn read_to_string(&mut self, uri: &reqwest::Url) -> Result<String, DriverError> {
        let url =
            get_base_url().join(uri.path()).map_err(|_| DriverError::FileNotFound(uri.clone()))?;

        // If the file has already been fetched, we don't fetch it again.
        if let Some(text) = self.cache.get(&url).cloned() {
            return Ok(text);
        }

        let text = fetch_text(self.client.clone(), url.clone()).await?;
        self.cache.insert(url, text.clone());
        Ok(text)
    }

    async fn write_string(&mut self, uri: &reqwest::Url, _source: &str) -> Result<(), DriverError> {
        web_sys::console::warn_1(&format!("Attempted to write to read-only source: {uri}").into());
        Ok(())
    }
}

async fn fetch_text(client: reqwest::Client, url: Url) -> Result<String, DriverError> {
    // HACK: The rest of the codebase expects this future to be Send.
    // Unfortunately, any futures carrying JsValues across await boundaries, like
    // the reqwest calls below, are not Send.
    // To work around this, we spawn a local task to perform the operations that are not Send.
    let (tx, rx) = oneshot::channel();

    let url_clone = url.clone();
    spawn_local(async move {
        let result = async {
            let response = client.get(url_clone.clone()).send().await.map_err(|e| {
                DriverError::Impossible(format!("Failed to fetch {}: {}", &url_clone, e))
            })?;

            let response = response.error_for_status().map_err(|e| {
                if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                    DriverError::FileNotFound(url_clone.clone())
                } else {
                    DriverError::Impossible(format!("HTTP error from {}: {}", url_clone, e))
                }
            })?;

            response.text().await.map_err(|e| {
                DriverError::Impossible(format!("Failed to read text from {}: {}", &url_clone, e))
            })
        }
        .await;

        let _ = tx.send(result);
    });

    rx.await.map_err(|_| DriverError::Impossible("Task cancelled".to_string()))?
}

/// Get the base URL from the browser
fn get_base_url() -> Url {
    let location = web_sys::window().unwrap().location();
    Url::parse(&format!("{}//{}", location.protocol().unwrap(), location.host().unwrap(),)).unwrap()
}
