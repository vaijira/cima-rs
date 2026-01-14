use crate::api_client::CimaClient;
use crate::models::ChangeRecord;
use anyhow::{Context, Result};

impl CimaClient {
    /// Get change log from a specific date
    ///
    /// Returns a paginated response with medication changes
    ///
    /// # Arguments
    /// * `date` - Date in format "dd/mm/yyyy"
    /// * `registration_numbers` - Optional list of registration numbers to filter
    pub async fn get_change_log(
        &self,
        date: &str,
        registration_numbers: Option<&[&str]>,
    ) -> Result<crate::models::PaginatedResponse<ChangeRecord>> {
        let mut params = vec![("fecha", date.to_string())];

        if let Some(regs) = registration_numbers {
            for reg in regs {
                params.push(("nregistro", reg.to_string()));
            }
        }

        self.get_with_params("registroCambios", &params)
            .await
            .context("Failed to get change log")
    }
}
