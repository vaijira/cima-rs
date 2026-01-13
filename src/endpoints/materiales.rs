use crate::api_client::CimaClient;
use crate::models::Material;
use anyhow::{Context, Result};

impl CimaClient {
    /// Obtiene materiales informativos asociados a un medicamento
    ///
    /// Returns a single Material object (not an array)
    pub async fn get_materiales_informativos(&self, nregistro: &str) -> Result<Material> {
        let params = vec![("nregistro", nregistro.to_string())];

        self.get_with_params("materiales", &params)
            .await
            .context("Failed to get materiales informativos")
    }
}
