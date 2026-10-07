use crate::api_client::CimaClient;
use crate::models::{Medication, MedicationSummary};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Medication search parameters
#[derive(Debug, Default, Clone)]
pub struct SearchMedicationsParams {
    /// Medication name
    pub name: Option<String>,
    /// Laboratory name
    pub laboratory: Option<String>,
    /// Active ingredient 1 name
    pub active_ingredient_1: Option<String>,
    /// Active ingredient 2 name
    pub active_ingredient_2: Option<String>,
    /// Active ingredient 1 ID
    pub active_ingredient_1_id: Option<i32>,
    /// Active ingredient 2 ID
    pub active_ingredient_2_id: Option<i32>,
    /// National code
    pub national_code: Option<String>,
    /// ATC code or description
    pub atc: Option<String>,
    /// Registration number
    pub registration_number: Option<String>,
    /// Number of active ingredients
    pub active_ingredient_count: Option<i32>,
    /// 1: has black triangle, 0: no black triangle
    pub black_triangle: Option<u8>,
    /// 1: orphan drug, 0: not orphan
    pub orphan: Option<u8>,
    /// 1: biosimilar, 0: not biosimilar
    pub biosimilar: Option<u8>,
    /// Substitutable type (1-5)
    pub substitutable_type: Option<u8>,
    /// VMP code ID
    pub vmp: Option<String>,
    /// 1: commercialized, 0: not commercialized
    pub commercialized: Option<u8>,
    /// 1: only authorized, 0: only not authorized
    pub authorized: Option<u8>,
    /// 1: requires prescription, 0: no prescription
    pub prescription: Option<u8>,
    /// 1: narcotics
    pub narcotic: Option<u8>,
    /// 1: psychotropics
    pub psychotropic: Option<u8>,
    /// 1: narcotics or psychotropics
    pub narcotic_or_psychotropic: Option<u8>,
    /// Page number
    pub page: Option<u32>,
}

impl SearchMedicationsParams {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build query parameters as vector of tuples
    pub(crate) fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();

        if let Some(ref v) = self.name {
            params.push(("nombre", v.clone()));
        }
        if let Some(ref v) = self.laboratory {
            params.push(("laboratorio", v.clone()));
        }
        if let Some(ref v) = self.active_ingredient_1 {
            params.push(("practiv1", v.clone()));
        }
        if let Some(ref v) = self.active_ingredient_2 {
            params.push(("practiv2", v.clone()));
        }
        if let Some(v) = self.active_ingredient_1_id {
            params.push(("idpractiv1", v.to_string()));
        }
        if let Some(v) = self.active_ingredient_2_id {
            params.push(("idpractiv2", v.to_string()));
        }
        if let Some(ref v) = self.national_code {
            params.push(("cn", v.clone()));
        }
        if let Some(ref v) = self.atc {
            params.push(("atc", v.clone()));
        }
        if let Some(ref v) = self.registration_number {
            params.push(("nregistro", v.clone()));
        }
        if let Some(v) = self.active_ingredient_count {
            params.push(("npactiv", v.to_string()));
        }
        if let Some(v) = self.black_triangle {
            params.push(("triangulo", v.to_string()));
        }
        if let Some(v) = self.orphan {
            params.push(("huerfano", v.to_string()));
        }
        if let Some(v) = self.biosimilar {
            params.push(("biosimilar", v.to_string()));
        }
        if let Some(v) = self.substitutable_type {
            params.push(("sust", v.to_string()));
        }
        if let Some(ref v) = self.vmp {
            params.push(("vmp", v.clone()));
        }
        if let Some(v) = self.commercialized {
            params.push(("comerc", v.to_string()));
        }
        if let Some(v) = self.authorized {
            params.push(("autorizados", v.to_string()));
        }
        if let Some(v) = self.prescription {
            params.push(("receta", v.to_string()));
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

/// Query for searching in technical data sheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalSheetQuery {
    /// Section to search (1-10, can have sublevels "4.1")
    #[serde(rename = "seccion")]
    pub section: String,
    /// Text to search for
    #[serde(rename = "texto")]
    pub text: String,
    /// 1: must contain text, 0: must not contain
    #[serde(rename = "contiene")]
    pub contains: u8,
}

impl CimaClient {
    /// Get medication information by registration number or national code
    pub async fn get_medication(
        &self,
        registration_number: Option<&str>,
        national_code: Option<&str>,
    ) -> Result<Medication> {
        let mut params = Vec::new();

        if let Some(nr) = registration_number {
            params.push(("nregistro", nr.to_string()));
        }
        if let Some(cn) = national_code {
            params.push(("cn", cn.to_string()));
        }

        if params.is_empty() {
            anyhow::bail!("Must provide either registration_number or national_code");
        }

        self.get_with_params("medicamento", &params)
            .await
            .context("Failed to get medication")
    }

    /// Search medications according to specified parameters
    ///
    /// Returns a paginated response with medication search results.
    pub async fn search_medications(
        &self,
        params: &SearchMedicationsParams,
    ) -> Result<crate::models::PaginatedResponse<MedicationSummary>> {
        let query_params = params.to_query_params();

        self.get_with_params("medicamentos", &query_params)
            .await
            .context("Failed to search medications")
    }

    /// Search medications by content in technical data sheet
    pub async fn search_in_technical_sheet(
        &self,
        queries: &[TechnicalSheetQuery],
    ) -> Result<Vec<MedicationSummary>> {
        self.post("buscarEnFichaTecnica", queries)
            .await
            .context("Failed to search in technical sheet")
    }

    /// Checks whether a medication dossier in CIMA corresponds to an authorized parallel import (AIP / I.P.).
    pub async fn check_parallel_import(
        &self,
        registration_number: Option<&str>,
        national_code: Option<&str>,
    ) -> Result<crate::models::ParallelImportDossierInfo> {
        let med = self.get_medication(registration_number, national_code).await?;
        let catalog = crate::billing::ParallelImporterCatalog::new_default();
        let is_lab_importer = catalog.is_importer(&med.labtitular);
        let has_syntax = crate::billing::detect_parallel_import_syntax(&med.name);
        let is_aip_reg = med.nregistro.to_uppercase().contains("AIP");

        let mut sources = Vec::new();
        let mut notes = Vec::new();

        if is_aip_reg {
            sources.push("aemps_dossier_aip".to_string());
            notes.push(format!("Registration number contains AIP marker: {}", med.nregistro));
        }
        if is_lab_importer {
            sources.push("importer_catalog".to_string());
            notes.push(format!(
                "Marketing authorization holder is a known parallel distributor: {}",
                med.labtitular
            ));
        }
        if has_syntax {
            sources.push("name_syntax".to_string());
            notes.push("Product name includes parallel import marker (I.P.)".to_string());
        }
        if let Some(ema) = med.ema
            && ema
            && is_lab_importer
        {
            sources.push("ema_centrally_authorized".to_string());
            notes.push(
                "Centrally authorized EMA product held by parallel distributor in Spain"
                    .to_string(),
            );
        }

        let is_pi = !sources.is_empty();
        let confidence = if is_aip_reg || (!sources.is_empty() && is_lab_importer && has_syntax) {
            100
        } else if is_lab_importer {
            95
        } else if has_syntax {
            90
        } else {
            0
        };

        Ok(crate::models::ParallelImportDossierInfo {
            nregistro: med.nregistro,
            name: med.name,
            labtitular: med.labtitular,
            is_parallel_import: is_pi,
            confidence_score: confidence,
            detection_source: if sources.is_empty() {
                "none".to_string()
            } else {
                sources.join(",")
            },
            notes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_params_to_query() {
        let params = SearchMedicationsParams {
            name: Some("Paracetamol".to_string()),
            black_triangle: Some(1),
            page: Some(2),
            ..Default::default()
        };

        let query = params.to_query_params();
        assert_eq!(query.len(), 3);
        assert!(
            query
                .iter()
                .any(|(k, v)| k == &"nombre" && v == "Paracetamol")
        );
        assert!(query.iter().any(|(k, v)| k == &"triangulo" && v == "1"));
        assert!(query.iter().any(|(k, v)| k == &"pagina" && v == "2"));
    }

    #[test]
    fn test_technical_sheet_query_serialization() {
        let query = TechnicalSheetQuery {
            section: "4.1".to_string(),
            text: "cáncer".to_string(),
            contains: 1,
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("4.1"));
        assert!(json.contains("cáncer"));
    }
}
