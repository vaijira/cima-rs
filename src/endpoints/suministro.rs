use crate::api_client::CimaClient;
use crate::models::ProblemaSuministro;
use anyhow::{Context, Result};

impl CimaClient {
    /// Obtiene todos los problemas de suministro actuales
    ///
    /// Returns a paginated response with all active supply problems.
    pub async fn get_problemas_suministro_all(
        &self,
    ) -> Result<crate::models::PaginatedResponse<ProblemaSuministro>> {
        self.get("psuministro")
            .await
            .context("Failed to get all problemas de suministro")
    }

    /// Get supply problems for a specific presentation by national code
    ///
    /// Returns a paginated response with supply problems for the specified CN
    pub async fn get_problemas_suministro(
        &self,
        cn: &str,
    ) -> Result<crate::models::PaginatedResponse<crate::models::ProblemaSuministro>> {
        let endpoint = format!("psuministro/{}", cn);
        self.get(&endpoint)
            .await
            .context("Failed to get problemas de suministro for CN")
    }
}
