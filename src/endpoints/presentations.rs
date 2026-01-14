use crate::api_client::CimaClient;
use crate::models::{Presentation, PresentationSummary};
use anyhow::{Context, Result};

/// Presentation search parameters
#[derive(Debug, Default, Clone)]
pub struct SearchPresentationsParams {
    /// National code
    pub national_code: Option<String>,
    /// Registration number
    pub registration_number: Option<String>,
    /// VMP code ID
    pub vmp: Option<String>,
    /// VMPP code ID
    pub vmpp: Option<String>,
    /// Active ingredient ID
    pub active_ingredient_id: Option<i32>,
    /// 1: commercialized, 0: not commercialized
    pub commercialized: Option<u8>,
    /// 1: narcotics
    pub narcotic: Option<u8>,
    /// 1: psychotropics
    pub psychotropic: Option<u8>,
    /// 1: narcotics or psychotropics
    pub narcotic_or_psychotropic: Option<u8>,
    /// Page number
    pub page: Option<u32>,
}

impl SearchPresentationsParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();

        if let Some(ref v) = self.national_code {
            params.push(("cn", v.clone()));
        }
        if let Some(ref v) = self.registration_number {
            params.push(("nregistro", v.clone()));
        }
        if let Some(ref v) = self.vmp {
            params.push(("vmp", v.clone()));
        }
        if let Some(ref v) = self.vmpp {
            params.push(("vmpp", v.clone()));
        }
        if let Some(v) = self.active_ingredient_id {
            params.push(("idpractiv1", v.to_string()));
        }
        if let Some(v) = self.commercialized {
            params.push(("comerc", v.to_string()));
        }
        if let Some(v) = self.narcotic {
            params.push(("estupefaciente", v.to_string()));
        }
        if let Some(v) = self.psychotropic {
            params.push(("psicotropo", v.to_string()));
        }
        if let Some(v) = self.narcotic_or_psychotropic {
            params.push(("estuopsico", v.to_string()));
        }
        if let Some(v) = self.page {
            params.push(("pagina", v.to_string()));
        }

        params
    }
}

impl CimaClient {
    /// Get presentation information by national code
    pub async fn get_presentation(&self, national_code: &str) -> Result<Presentation> {
        let endpoint = format!("presentacion/{}", national_code);
        self.get(&endpoint)
            .await
            .context("Failed to get presentation")
    }

    /// Search presentations according to specified parameters
    ///
    /// Returns a paginated response with presentation search results.
    pub async fn search_presentations(
        &self,
        params: &SearchPresentationsParams,
    ) -> Result<crate::models::PaginatedResponse<PresentationSummary>> {
        let query_params = params.to_query_params();

        self.get_with_params("presentaciones", &query_params)
            .await
            .context("Failed to search presentations")
    }
}
