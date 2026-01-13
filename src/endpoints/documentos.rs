use crate::api_client::CimaClient;
use crate::models::{Seccion, TipoDocumento};
use anyhow::{Context, Result};

impl CimaClient {
    /// Obtiene la lista de secciones de un documento (sin contenido)
    pub async fn get_doc_secciones(
        &self,
        tipo_doc: TipoDocumento,
        nregistro: &str,
    ) -> Result<Vec<Seccion>> {
        let endpoint = format!("docSegmentado/secciones/{}", tipo_doc as u8);
        let params = vec![("nregistro", nregistro.to_string())];

        self.get_with_params(&endpoint, &params)
            .await
            .context("Failed to get document sections")
    }

    /// Obtiene el contenido de las secciones de un documento
    pub async fn get_doc_contenido(
        &self,
        tipo_doc: TipoDocumento,
        nregistro: &str,
        seccion: Option<&str>,
    ) -> Result<Vec<Seccion>> {
        let endpoint = format!("docSegmentado/contenido/{}", tipo_doc as u8);
        let mut params = vec![("nregistro", nregistro.to_string())];

        if let Some(sec) = seccion {
            params.push(("seccion", sec.to_string()));
        }

        self.get_with_params(&endpoint, &params)
            .await
            .context("Failed to get document content")
    }

    /// Obtiene la ficha técnica completa en HTML
    pub async fn get_ficha_tecnica_html(&self, nregistro: &str) -> Result<String> {
        let url = format!(
            "https://cima.aemps.es/cima/dochtml/ft/{}/FichaTecnica.html",
            nregistro
        );

        self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to fetch ficha tecnica HTML".to_string())?
            .text()
            .await
            .context("Failed to read ficha tecnica HTML")
    }

    /// Obtiene una sección específica de la ficha técnica en HTML
    pub async fn get_ficha_tecnica_seccion_html(
        &self,
        nregistro: &str,
        seccion: &str,
    ) -> Result<String> {
        let url = format!(
            "https://cima.aemps.es/cima/dochtml/ft/{}/{}/FichaTecnica.html",
            nregistro, seccion
        );

        self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to fetch ficha tecnica section HTML".to_string())?
            .text()
            .await
            .context("Failed to read ficha tecnica section HTML")
    }

    /// Obtiene el prospecto completo en HTML
    pub async fn get_prospecto_html(&self, nregistro: &str) -> Result<String> {
        let url = format!(
            "https://cima.aemps.es/cima/dochtml/p/{}/Prospecto.html",
            nregistro
        );

        self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to fetch prospecto HTML".to_string())?
            .text()
            .await
            .context("Failed to read prospecto HTML")
    }

    /// Obtiene una sección específica del prospecto en HTML
    pub async fn get_prospecto_seccion_html(
        &self,
        nregistro: &str,
        seccion: &str,
    ) -> Result<String> {
        let url = format!(
            "https://cima.aemps.es/cima/dochtml/p/{}/{}/Prospecto.html",
            nregistro, seccion
        );

        self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to fetch prospecto section HTML".to_string())?
            .text()
            .await
            .context("Failed to read prospecto section HTML")
    }
}
