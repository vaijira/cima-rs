use crate::api_client::CimaClient;
use crate::models::SafetyNote;
use anyhow::{Context, Result};

impl CimaClient {
    /// Get safety notes associated with a medication
    pub async fn get_safety_notes(&self, registration_number: &str) -> Result<Vec<SafetyNote>> {
        let params = vec![("nregistro", registration_number.to_string())];

        self.get_with_params("notas", &params)
            .await
            .context("Failed to get safety notes")
    }
}
