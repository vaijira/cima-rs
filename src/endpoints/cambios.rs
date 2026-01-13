use crate::api_client::CimaClient;
use crate::models::RegistroCambios;
use anyhow::{Context, Result};

impl CimaClient {
    /// Obtiene el registro de cambios desde una fecha determinada
    ///
    /// Returns a paginated response with medication changes
    ///
    /// # Arguments
    /// * `fecha` - Fecha en formato "dd/mm/yyyy"
    /// * `nregistros` - Lista opcional de números de registro para filtrar
    pub async fn get_registro_cambios(
        &self,
        fecha: &str,
        nregistros: Option<&[&str]>,
    ) -> Result<crate::models::PaginatedResponse<RegistroCambios>> {
        let mut params = vec![("fecha", fecha.to_string())];

        if let Some(regs) = nregistros {
            for reg in regs {
                params.push(("nregistro", reg.to_string()));
            }
        }

        self.get_with_params("registroCambios", &params)
            .await
            .context("Failed to get registro de cambios")
    }
}
