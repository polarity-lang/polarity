use async_trait::async_trait;
use driver::{DriverError, FileSource};
use futures::channel::oneshot;
use reqwest::Url;
use wasm_bindgen_futures::spawn_local;

#[derive(Default, Clone)]
pub struct FetchSource {
    client: reqwest::Client,
}

#[async_trait]
impl FileSource for FetchSource {
    fn manage(&mut self, _uri: &reqwest::Url) -> bool {
        true
    }

    fn manages(&self, _uri: &reqwest::Url) -> bool {
        true
    }

    async fn read_to_string(&mut self, uri: &reqwest::Url) -> Result<String, DriverError> {
        let url =
            get_base_url().join(uri.path()).map_err(|_| DriverError::FileNotFound(uri.clone()))?;

        // HACK: The rest of the codebase expects this future to be Send.
        // Unfortunately, any futures carrying JsValues across await boundaries, like
        // the reqwest calls below, are not Send.
        // To work around this, we spawn a local task to perform the operations that are not Send.
        let (tx, rx) = oneshot::channel();

        let client = self.client.clone();
        let url_clone = url.clone();

        spawn_local(async move {
            let result = async {
                let response = client.get(url_clone.clone()).send().await.map_err(|e| {
                    DriverError::Impossible(format!("Failed to fetch {}: {}", &url_clone, e))
                })?;

                let text = response.text().await.map_err(|e| {
                    DriverError::Impossible(format!(
                        "Failed to read text from {}: {}",
                        &url_clone, e
                    ))
                })?;

                Ok(text)
            }
            .await;

            let _ = tx.send(result);
        });

        rx.await.map_err(|_| DriverError::Impossible("Task cancelled".to_string()))?
    }

    async fn write_string(&mut self, uri: &reqwest::Url, _source: &str) -> Result<(), DriverError> {
        web_sys::console::warn_1(
            &format!("Attempted to write to read-only source: {}", uri).into(),
        );
        Ok(())
    }
}

/// Get the base URL from the browser
fn get_base_url() -> Url {
    let location = web_sys::window().unwrap().location();
    Url::parse(&format!("{}//{}", location.protocol().unwrap(), location.host().unwrap(),)).unwrap()
}
