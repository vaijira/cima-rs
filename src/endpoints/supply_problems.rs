use crate::api_client::CimaClient;
use crate::models::SupplyProblem;
use anyhow::{Context, Result};

impl CimaClient {
    /// Get all current supply problems
    ///
    /// Returns a paginated response with all active supply problems.
    pub async fn get_all_supply_problems(
        &self,
    ) -> Result<crate::models::PaginatedResponse<SupplyProblem>> {
        self.get("psuministro")
            .await
            .context("Failed to get all supply problems")
    }

    /// Get supply problems for a specific presentation by national code
    ///
    /// Returns a paginated response with supply problems for the specified CN
    pub async fn get_supply_problems(
        &self,
        national_code: &str,
    ) -> Result<crate::models::PaginatedResponse<SupplyProblem>> {
        let endpoint = format!("psuministro/{}", national_code);
        self.get(&endpoint)
            .await
            .context("Failed to get supply problems for national code")
    }
}
