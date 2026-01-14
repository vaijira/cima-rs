use crate::api_client::CimaClient;
use crate::models::{MasterDataType, MasterItem};
use anyhow::{Context, Result};

/// Master data search parameters
#[derive(Debug, Default, Clone)]
pub struct MasterDataParams {
    /// Element name
    pub name: Option<String>,
    /// Element ID
    pub id: Option<i32>,
    /// Element code
    pub code: Option<String>,
    /// 1: narcotics (only for active ingredients)
    pub narcotic: Option<u8>,
    /// 1: psychotropics (only for active ingredients)
    pub psychotropic: Option<u8>,
    /// 1: narcotics or psychotropics (only for active ingredients)
    pub narcotic_or_psychotropic: Option<u8>,
    /// 0: returns both associated with medications and those that are not
    pub in_use: Option<u8>,
    /// Page number
    pub page: Option<u32>,
}

impl MasterDataParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn to_query_params(&self, data_type: MasterDataType) -> Vec<(&str, String)> {
        let mut params = vec![("maestra", data_type.as_u8().to_string())];

        if let Some(ref v) = self.name {
            params.push(("nombre", v.clone()));
        }
        if let Some(v) = self.id {
            params.push(("id", v.to_string()));
        }
        if let Some(ref v) = self.code {
            params.push(("codigo", v.clone()));
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
        if let Some(v) = self.in_use {
            params.push(("enuso", v.to_string()));
        }
        if let Some(v) = self.page {
            params.push(("pagina", v.to_string()));
        }

        params
    }
}

impl CimaClient {
    /// Get elements from a master data catalog (reference catalog)
    ///
    /// **Important**: The CIMA API requires at least one filter parameter to be set
    /// (name, id, code, narcotic, psychotropic, narcotic_or_psychotropic, or in_use).
    /// Without any filter, the API returns 204 No Content.
    ///
    /// Returns a paginated response with master data items.
    pub async fn get_master_data(
        &self,
        data_type: MasterDataType,
        params: &MasterDataParams,
    ) -> Result<crate::models::PaginatedResponse<MasterItem>> {
        let query_params = params.to_query_params(data_type);

        self.get_with_params("maestras", &query_params)
            .await
            .context("Failed to get master data")
    }
}
