use crate::api_client::CimaClient;
use crate::models::{Item, TipoMaestra};
use anyhow::{Context, Result};

/// Parámetros de búsqueda de maestras
#[derive(Debug, Default, Clone)]
pub struct MaestraParams {
    /// Nombre del elemento
    pub nombre: Option<String>,
    /// ID del elemento
    pub id: Option<i32>,
    /// Código del elemento
    pub codigo: Option<String>,
    /// 1: estupefacientes (solo para principios activos)
    pub estupefaciente: Option<u8>,
    /// 1: psicótropos (solo para principios activos)
    pub psicotropo: Option<u8>,
    /// 1: estupefacientes o psicótropos (solo para principios activos)
    pub estuopsico: Option<u8>,
    /// 0: devuelve tanto los asociados a medicamentos como los que no
    pub enuso: Option<u8>,
    /// Número de página
    pub pagina: Option<u32>,
}

impl MaestraParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn to_query_params(&self, tipo: TipoMaestra) -> Vec<(&str, String)> {
        let mut params = vec![("maestra", tipo.as_u8().to_string())];

        if let Some(ref v) = self.nombre {
            params.push(("nombre", v.clone()));
        }
        if let Some(v) = self.id {
            params.push(("id", v.to_string()));
        }
        if let Some(ref v) = self.codigo {
            params.push(("codigo", v.clone()));
        }
        if let Some(v) = self.estupefaciente {
            params.push(("estupefaciente", v.to_string()));
        }
        if let Some(v) = self.psicotropo {
            params.push(("psicotropo", v.to_string()));
        }
        if let Some(v) = self.estuopsico {
            params.push(("estuopsico", v.to_string()));
        }
        if let Some(v) = self.enuso {
            params.push(("enuso", v.to_string()));
        }
        if let Some(v) = self.pagina {
            params.push(("pagina", v.to_string()));
        }

        params
    }
}

impl CimaClient {
    /// Obtiene elementos de una maestra (catálogo de referencia)
    ///
    /// **Important**: The CIMA API requires at least one filter parameter to be set
    /// (nombre, id, codigo, estupefaciente, psicotropo, estuopsico, or enuso).
    /// Without any filter, the API returns 204 No Content.
    ///
    /// Returns a paginated response with master data items.
    pub async fn get_maestra(
        &self,
        tipo: TipoMaestra,
        params: &MaestraParams,
    ) -> Result<crate::models::PaginatedResponse<Item>> {
        let query_params = params.to_query_params(tipo);

        self.get_with_params("maestras", &query_params)
            .await
            .context("Failed to get maestra")
    }
}
