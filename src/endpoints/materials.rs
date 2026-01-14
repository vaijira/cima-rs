use crate::api_client::CimaClient;
use crate::models::SafetyMaterial;
use anyhow::{Context, Result};

impl CimaClient {
    /// Get informative materials associated with a medication
    ///
    /// Returns a single SafetyMaterial object (not an array)
    pub async fn get_informative_materials(
        &self,
        registration_number: &str,
    ) -> Result<SafetyMaterial> {
        let params = vec![("nregistro", registration_number.to_string())];

        self.get_with_params("materiales", &params)
            .await
            .context("Failed to get informative materials")
    }
}
