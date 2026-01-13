use crate::api_client::CimaClient;
use crate::models::{Presentacion, PresentacionList};
use anyhow::{Context, Result};

/// Parámetros de búsqueda de presentaciones
#[derive(Debug, Default, Clone)]
pub struct SearchPresentacionesParams {
    /// Código nacional
    pub cn: Option<String>,
    /// Nº de registro
    pub nregistro: Option<String>,
    /// ID del código VMP
    pub vmp: Option<String>,
    /// ID del código VMPP
    pub vmpp: Option<String>,
    /// ID del principio activo
    pub idpractiv1: Option<i32>,
    /// 1: comercializados, 0: no comercializados
    pub comerc: Option<u8>,
    /// 1: estupefacientes
    pub estupefaciente: Option<u8>,
    /// 1: psicótropos
    pub psicotropo: Option<u8>,
    /// 1: estupefacientes o psicótropos
    pub estuopsico: Option<u8>,
    /// Número de página
    pub pagina: Option<u32>,
}

impl SearchPresentacionesParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();

        if let Some(ref v) = self.cn {
            params.push(("cn", v.clone()));
        }
        if let Some(ref v) = self.nregistro {
            params.push(("nregistro", v.clone()));
        }
        if let Some(ref v) = self.vmp {
            params.push(("vmp", v.clone()));
        }
        if let Some(ref v) = self.vmpp {
            params.push(("vmpp", v.clone()));
        }
        if let Some(v) = self.idpractiv1 {
            params.push(("idpractiv1", v.to_string()));
        }
        if let Some(v) = self.comerc {
            params.push(("comerc", v.to_string()));
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
        if let Some(v) = self.pagina {
            params.push(("pagina", v.to_string()));
        }

        params
    }
}

impl CimaClient {
    /// Obtiene información de una presentación por código nacional
    pub async fn get_presentacion(&self, cn: &str) -> Result<Presentacion> {
        let endpoint = format!("presentacion/{}", cn);
        self.get(&endpoint)
            .await
            .context("Failed to get presentacion")
    }

    /// Busca presentaciones según los parámetros especificados
    ///
    /// Returns a paginated response with presentation search results.
    pub async fn search_presentaciones(
        &self,
        params: &SearchPresentacionesParams,
    ) -> Result<crate::models::PaginatedResponse<PresentacionList>> {
        let query_params = params.to_query_params();

        self.get_with_params("presentaciones", &query_params)
            .await
            .context("Failed to search presentaciones")
    }
}
