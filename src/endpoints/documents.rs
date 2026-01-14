use crate::api_client::CimaClient;
use crate::models::{DocumentType, Section};
use anyhow::{Context, Result};

impl CimaClient {
    /// Get document sections list (without content)
    pub async fn get_document_sections(
        &self,
        doc_type: DocumentType,
        registration_number: &str,
    ) -> Result<Vec<Section>> {
        let endpoint = format!("docSegmentado/secciones/{}", doc_type as u8);
        let params = vec![("nregistro", registration_number.to_string())];

        self.get_with_params(&endpoint, &params)
            .await
            .context("Failed to get document sections")
    }

    /// Get document section content
    pub async fn get_document_content(
        &self,
        doc_type: DocumentType,
        registration_number: &str,
        section: Option<&str>,
    ) -> Result<Vec<Section>> {
        let endpoint = format!("docSegmentado/contenido/{}", doc_type as u8);
        let mut params = vec![("nregistro", registration_number.to_string())];

        if let Some(sec) = section {
            params.push(("seccion", sec.to_string()));
        }

        self.get_with_params(&endpoint, &params)
            .await
            .context("Failed to get document content")
    }

    /// Get complete technical data sheet in HTML
    pub async fn get_technical_sheet_html(&self, registration_number: &str) -> Result<String> {
        let url = format!(
            "https://cima.aemps.es/cima/dochtml/ft/{}/FichaTecnica.html",
            registration_number
        );

        self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to fetch technical sheet HTML".to_string())?
            .text()
            .await
            .context("Failed to read technical sheet HTML")
    }

    /// Get a specific section of the technical data sheet in HTML
    pub async fn get_technical_sheet_section_html(
        &self,
        registration_number: &str,
        section: &str,
    ) -> Result<String> {
        let url = format!(
            "https://cima.aemps.es/cima/dochtml/ft/{}/{}/FichaTecnica.html",
            registration_number, section
        );

        self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to fetch technical sheet section HTML".to_string())?
            .text()
            .await
            .context("Failed to read technical sheet section HTML")
    }

    /// Get complete package leaflet in HTML
    pub async fn get_package_leaflet_html(&self, registration_number: &str) -> Result<String> {
        let url = format!(
            "https://cima.aemps.es/cima/dochtml/p/{}/Prospecto.html",
            registration_number
        );

        self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to fetch package leaflet HTML".to_string())?
            .text()
            .await
            .context("Failed to read package leaflet HTML")
    }

    /// Get a specific section of the package leaflet in HTML
    pub async fn get_package_leaflet_section_html(
        &self,
        registration_number: &str,
        section: &str,
    ) -> Result<String> {
        let url = format!(
            "https://cima.aemps.es/cima/dochtml/p/{}/{}/Prospecto.html",
            registration_number, section
        );

        self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "Failed to fetch package leaflet section HTML".to_string())?
            .text()
            .await
            .context("Failed to read package leaflet section HTML")
    }
}
