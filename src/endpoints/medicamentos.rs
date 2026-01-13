use crate::api_client::CimaClient;
use crate::models::{Medicamento, MedicamentoList};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Parámetros de búsqueda de medicamentos
#[derive(Debug, Default, Clone)]
pub struct SearchMedicamentosParams {
    /// Nombre del medicamento
    pub nombre: Option<String>,
    /// Nombre del laboratorio
    pub laboratorio: Option<String>,
    /// Nombre del principio activo 1
    pub practiv1: Option<String>,
    /// Nombre del principio activo 2
    pub practiv2: Option<String>,
    /// ID del principio activo 1
    pub idpractiv1: Option<i32>,
    /// ID del principio activo 2
    pub idpractiv2: Option<i32>,
    /// Código nacional
    pub cn: Option<String>,
    /// Código ATC o descripción
    pub atc: Option<String>,
    /// Nº de registro
    pub nregistro: Option<String>,
    /// Nº de principios activos
    pub npactiv: Option<i32>,
    /// 1: tienen triángulo, 0: no tienen triángulo
    pub triangulo: Option<u8>,
    /// 1: huérfano, 0: no huérfano
    pub huerfano: Option<u8>,
    /// 1: biosimilar, 0: no biosimilar
    pub biosimilar: Option<u8>,
    /// Tipo de sustituible (1-5)
    pub sust: Option<u8>,
    /// ID del código VMP
    pub vmp: Option<String>,
    /// 1: comercializados, 0: no comercializados
    pub comerc: Option<u8>,
    /// 1: solo autorizados, 0: solo no autorizados
    pub autorizados: Option<u8>,
    /// 1: con receta, 0: sin receta
    pub receta: Option<u8>,
    /// 1: estupefacientes
    pub estupefaciente: Option<u8>,
    /// 1: psicótropos
    pub psicotropo: Option<u8>,
    /// 1: estupefacientes o psicótropos
    pub estuopsico: Option<u8>,
    /// Número de página
    pub pagina: Option<u32>,
}

impl SearchMedicamentosParams {
    pub fn new() -> Self {
        Self::default()
    }

    /// Construye los parámetros query como vector de tuplas
    pub(crate) fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();

        if let Some(ref v) = self.nombre {
            params.push(("nombre", v.clone()));
        }
        if let Some(ref v) = self.laboratorio {
            params.push(("laboratorio", v.clone()));
        }
        if let Some(ref v) = self.practiv1 {
            params.push(("practiv1", v.clone()));
        }
        if let Some(ref v) = self.practiv2 {
            params.push(("practiv2", v.clone()));
        }
        if let Some(v) = self.idpractiv1 {
            params.push(("idpractiv1", v.to_string()));
        }
        if let Some(v) = self.idpractiv2 {
            params.push(("idpractiv2", v.to_string()));
        }
        if let Some(ref v) = self.cn {
            params.push(("cn", v.clone()));
        }
        if let Some(ref v) = self.atc {
            params.push(("atc", v.clone()));
        }
        if let Some(ref v) = self.nregistro {
            params.push(("nregistro", v.clone()));
        }
        if let Some(v) = self.npactiv {
            params.push(("npactiv", v.to_string()));
        }
        if let Some(v) = self.triangulo {
            params.push(("triangulo", v.to_string()));
        }
        if let Some(v) = self.huerfano {
            params.push(("huerfano", v.to_string()));
        }
        if let Some(v) = self.biosimilar {
            params.push(("biosimilar", v.to_string()));
        }
        if let Some(v) = self.sust {
            params.push(("sust", v.to_string()));
        }
        if let Some(ref v) = self.vmp {
            params.push(("vmp", v.clone()));
        }
        if let Some(v) = self.comerc {
            params.push(("comerc", v.to_string()));
        }
        if let Some(v) = self.autorizados {
            params.push(("autorizados", v.to_string()));
        }
        if let Some(v) = self.receta {
            params.push(("receta", v.to_string()));
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

/// Query para búsqueda en ficha técnica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FichaTecnicaQuery {
    /// Sección donde buscar (1-10, puede tener subniveles "4.1")
    pub seccion: String,
    /// Texto a buscar
    pub texto: String,
    /// 1: debe contener el texto, 0: no debe contenerlo
    pub contiene: u8,
}

impl CimaClient {
    /// Obtiene información de un medicamento por nº de registro o código nacional
    pub async fn get_medicamento(
        &self,
        nregistro: Option<&str>,
        cn: Option<&str>,
    ) -> Result<Medicamento> {
        let mut params = Vec::new();

        if let Some(nr) = nregistro {
            params.push(("nregistro", nr.to_string()));
        }
        if let Some(codigo) = cn {
            params.push(("cn", codigo.to_string()));
        }

        if params.is_empty() {
            anyhow::bail!("Must provide either nregistro or cn");
        }

        self.get_with_params("medicamento", &params)
            .await
            .context("Failed to get medicamento")
    }

    /// Busca medicamentos según los parámetros especificados
    ///
    /// Returns a paginated response with medication search results.
    pub async fn search_medicamentos(
        &self,
        params: &SearchMedicamentosParams,
    ) -> Result<crate::models::PaginatedResponse<MedicamentoList>> {
        let query_params = params.to_query_params();

        self.get_with_params("medicamentos", &query_params)
            .await
            .context("Failed to search medicamentos")
    }

    /// Busca medicamentos por contenido en la ficha técnica
    pub async fn buscar_en_ficha_tecnica(
        &self,
        queries: &[FichaTecnicaQuery],
    ) -> Result<Vec<MedicamentoList>> {
        self.post("buscarEnFichaTecnica", queries)
            .await
            .context("Failed to search in ficha tecnica")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_params_to_query() {
        let params = SearchMedicamentosParams {
            nombre: Some("Paracetamol".to_string()),
            triangulo: Some(1),
            pagina: Some(2),
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
    fn test_ficha_tecnica_query_serialization() {
        let query = FichaTecnicaQuery {
            seccion: "4.1".to_string(),
            texto: "cáncer".to_string(),
            contiene: 1,
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("4.1"));
        assert!(json.contains("cáncer"));
    }
}
