use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::instrument;

const BASE_URL: &str = "https://cima.aemps.es/cima/rest";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Cliente para interactuar con la API REST de CIMA
#[derive(Clone, Debug)]
pub struct CimaClient {
    base_url: String,
    pub(crate) client: Client,
}

impl CimaClient {
    /// Crea un nuevo cliente CIMA con configuración por defecto
    pub fn new() -> Result<Self> {
        Self::with_base_url(BASE_URL)
    }

    /// Crea un cliente con una URL base personalizada (útil para testing)
    pub fn with_base_url(base_url: &str) -> Result<Self> {
        tracing::debug!(base_url, "Creating CIMA client");

        let client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .user_agent("cima-rs/0.0.1")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: base_url.to_string(),
            client,
        })
    }

    /// Construye una URL completa para un endpoint
    pub(crate) fn build_url(&self, endpoint: &str) -> String {
        format!("{}/{}", self.base_url, endpoint)
    }

    /// Realiza una petición GET y deserializa la respuesta JSON
    #[instrument(skip(self), fields(url))]
    pub(crate) async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = self.build_url(endpoint);
        tracing::Span::current().record("url", &url);

        tracing::debug!("Sending GET request");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to send GET request to {}", url))?;

        let status = response.status();
        tracing::debug!(%status, "Received response");

        if !status.is_success() {
            tracing::error!(%status, %url, "API returned error status");
            anyhow::bail!("API returned error status {}: {}", status, url);
        }

        response
            .json::<T>()
            .await
            .with_context(|| format!("Failed to deserialize JSON response from {}", url))
    }

    /// Realiza una petición GET con parámetros query
    #[instrument(skip(self, params), fields(url, param_count = params.len()))]
    pub(crate) async fn get_with_params<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, String)],
    ) -> Result<T> {
        let mut url = self.build_url(endpoint);

        // Build query string manually
        if !params.is_empty() {
            url.push('?');
            for (i, (key, value)) in params.iter().enumerate() {
                if i > 0 {
                    url.push('&');
                }
                url.push_str(key);
                url.push('=');
                url.push_str(&urlencoding::encode(value));
            }
        }

        tracing::Span::current().record("url", &url);
        tracing::debug!(params = ?params, "Sending GET request with parameters");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to send GET request to {}", url))?;

        let status = response.status();
        tracing::debug!(%status, "Received response");

        if !status.is_success() {
            tracing::error!(%status, %url, "API returned error status");
            anyhow::bail!("API returned error status {}: {}", status, url);
        }

        response
            .json::<T>()
            .await
            .with_context(|| format!("Failed to deserialize JSON response from {}", url))
    }

    /// Realiza una petición POST con body JSON
    #[instrument(skip(self, body), fields(url))]
    pub(crate) async fn post<T: DeserializeOwned, B: serde::Serialize + ?Sized>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.build_url(endpoint);
        tracing::Span::current().record("url", &url);

        tracing::debug!("Sending POST request");

        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("Failed to send POST request to {}", url))?;

        let status = response.status();
        tracing::debug!(%status, "Received response");

        if !status.is_success() {
            tracing::error!(%status, %url, "API returned error status");
            anyhow::bail!("API returned error status {}: {}", status, url);
        }

        response
            .json::<T>()
            .await
            .with_context(|| format!("Failed to deserialize JSON response from {}", url))
    }

    /// Realiza una petición GET y devuelve el contenido como texto
    #[instrument(skip(self), fields(url))]
    pub(crate) async fn get_text(&self, endpoint: &str) -> Result<String> {
        let url = self.build_url(endpoint);
        tracing::Span::current().record("url", &url);

        tracing::debug!("Sending GET request for text content");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to send GET request to {}", url))?;

        let status = response.status();
        tracing::debug!(%status, "Received response");

        if !status.is_success() {
            tracing::error!(%status, %url, "API returned error status");
            anyhow::bail!("API returned error status {}: {}", status, url);
        }

        response
            .text()
            .await
            .with_context(|| format!("Failed to read text response from {}", url))
    }
}

impl Default for CimaClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default CIMA client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url() {
        let client = CimaClient::new().unwrap();
        assert_eq!(
            client.build_url("medicamento"),
            "https://cima.aemps.es/cima/rest/medicamento"
        );
    }

    #[test]
    fn test_custom_base_url() {
        let client = CimaClient::with_base_url("http://localhost:8080").unwrap();
        assert_eq!(client.build_url("test"), "http://localhost:8080/test");
    }
}
