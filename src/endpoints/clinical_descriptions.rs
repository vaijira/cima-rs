use crate::api_client::CimaClient;
use crate::models::ClinicalDescription;
use anyhow::{Context, Result};

/// VMP/VMPP search parameters
#[derive(Debug, Default, Clone)]
pub struct SearchClinicalDescriptionParams {
    /// Active ingredient name
    pub active_ingredient: Option<String>,
    /// Active ingredient ID
    pub active_ingredient_id: Option<i32>,
    /// Dose
    pub dose: Option<String>,
    /// Pharmaceutical form name
    pub pharmaceutical_form: Option<String>,
    /// ATC code or description
    pub atc: Option<String>,
    /// Medication name
    pub name: Option<String>,
    /// If included, returns results in hierarchical mode
    pub tree_mode: bool,
    /// Page number
    pub page: Option<u32>,
}

impl SearchClinicalDescriptionParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();

        if let Some(ref v) = self.active_ingredient {
            params.push(("practiv1", v.clone()));
        }
        if let Some(v) = self.active_ingredient_id {
            params.push(("idpractiv1", v.to_string()));
        }
        if let Some(ref v) = self.dose {
            params.push(("dosis", v.clone()));
        }
        if let Some(ref v) = self.pharmaceutical_form {
            params.push(("forma", v.clone()));
        }
        if let Some(ref v) = self.atc {
            params.push(("atc", v.clone()));
        }
        if let Some(ref v) = self.name {
            params.push(("nombre", v.clone()));
        }
        if self.tree_mode {
            params.push(("modoArbol", "true".to_string()));
        }
        if let Some(v) = self.page {
            params.push(("pagina", v.to_string()));
        }

        params
    }
}

impl CimaClient {
    /// Search clinical descriptions (VMP/VMPP)
    ///
    /// Returns a paginated response with clinical descriptions
    pub async fn search_clinical_descriptions(
        &self,
        params: &SearchClinicalDescriptionParams,
    ) -> Result<crate::models::PaginatedResponse<ClinicalDescription>> {
        let query_params = params.to_query_params();

        self.get_with_params("vmpp", &query_params)
            .await
            .context("Failed to search clinical descriptions")
    }
}
