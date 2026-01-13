use crate::api_client::CimaClient;
use crate::models::Nota;
use anyhow::{Context, Result};

impl CimaClient {
    /// Obtiene notas de seguridad asociadas a un medicamento
    pub async fn get_notas_seguridad(&self, nregistro: &str) -> Result<Vec<Nota>> {
        let params = vec![("nregistro", nregistro.to_string())];

        self.get_with_params("notas", &params)
            .await
            .context("Failed to get notas de seguridad")
    }
}
