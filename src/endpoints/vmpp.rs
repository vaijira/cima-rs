use crate::api_client::CimaClient;
use crate::models::DescripcionClinica;
use anyhow::{Context, Result};

/// Parámetros de búsqueda de VMP/VMPP
#[derive(Debug, Default, Clone)]
pub struct SearchVmppParams {
    /// Nombre del principio activo
    pub practiv1: Option<String>,
    /// ID del principio activo
    pub idpractiv1: Option<i32>,
    /// Dosis
    pub dosis: Option<String>,
    /// Nombre de la forma farmacéutica
    pub forma: Option<String>,
    /// Código ATC o descripción
    pub atc: Option<String>,
    /// Nombre del medicamento
    pub nombre: Option<String>,
    /// Si se incluye, devuelve resultados de modo jerárquico
    pub modo_arbol: bool,
    /// Número de página
    pub pagina: Option<u32>,
}

impl SearchVmppParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();

        if let Some(ref v) = self.practiv1 {
            params.push(("practiv1", v.clone()));
        }
        if let Some(v) = self.idpractiv1 {
            params.push(("idpractiv1", v.to_string()));
        }
        if let Some(ref v) = self.dosis {
            params.push(("dosis", v.clone()));
        }
        if let Some(ref v) = self.forma {
            params.push(("forma", v.clone()));
        }
        if let Some(ref v) = self.atc {
            params.push(("atc", v.clone()));
        }
        if let Some(ref v) = self.nombre {
            params.push(("nombre", v.clone()));
        }
        if self.modo_arbol {
            params.push(("modoArbol", "true".to_string()));
        }
        if let Some(v) = self.pagina {
            params.push(("pagina", v.to_string()));
        }

        params
    }
}

impl CimaClient {
    /// Busca descripciones clínicas (VMP/VMPP)
    ///
    /// Returns a paginated response with clinical descriptions
    pub async fn search_vmpp(
        &self,
        params: &SearchVmppParams,
    ) -> Result<crate::models::PaginatedResponse<DescripcionClinica>> {
        let query_params = params.to_query_params();

        self.get_with_params("vmpp", &query_params)
            .await
            .context("Failed to search VMP/VMPP")
    }
}
