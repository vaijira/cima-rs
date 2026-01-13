use anyhow::Result;
use cima_rs::{
    CimaClient, MaestraParams, SearchMedicamentosParams, SearchPresentacionesParams, TipoMaestra,
};

/// Helper to create a client for tests
fn create_client() -> Result<CimaClient> {
    CimaClient::new()
}

#[tokio::test]
async fn test_get_medicamento_by_nregistro() -> Result<()> {
    let client = create_client()?;

    let med = client.get_medicamento(Some("72112"), None).await?;

    assert_eq!(med.nregistro, "72112");
    assert!(med.nombre.contains("PARACETAMOL"));
    assert!(!med.presentaciones.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_medicamento_by_cn() -> Result<()> {
    let client = create_client()?;

    let med = client.get_medicamento(None, Some("672442")).await?;

    assert_eq!(med.nregistro, "72112");
    assert!(med.nombre.contains("PARACETAMOL"));

    Ok(())
}

#[tokio::test]
async fn test_search_medicamentos() -> Result<()> {
    let client = create_client()?;

    let params = SearchMedicamentosParams {
        nombre: Some("paracetamol".to_string()),
        ..Default::default()
    };

    let response = client.search_medicamentos(&params).await?;

    assert!(response.total_rows > 0);
    assert!(!response.results.is_empty());
    assert_eq!(response.page, 1);

    Ok(())
}

#[tokio::test]
async fn test_search_medicamentos_with_filters() -> Result<()> {
    let client = create_client()?;

    let params = SearchMedicamentosParams {
        nombre: Some("ibuprofeno".to_string()),
        comerc: Some(1), // Only commercialized
        ..Default::default()
    };

    let response = client.search_medicamentos(&params).await?;

    assert!(response.total_rows > 0);
    // Check that at least some results are commercialized
    let has_commercialized = response.results.iter().any(|med| med.comerc == Some(true));
    assert!(has_commercialized);

    Ok(())
}

#[tokio::test]
async fn test_get_presentacion() -> Result<()> {
    let client = create_client()?;

    let pres = client.get_presentacion("672442").await?;

    assert_eq!(pres.cn, "672442");
    assert!(pres.nombre.contains("PARACETAMOL"));

    Ok(())
}

#[tokio::test]
async fn test_search_presentaciones() -> Result<()> {
    let client = create_client()?;

    let params = SearchPresentacionesParams {
        nregistro: Some("72112".to_string()),
        ..Default::default()
    };

    let response = client.search_presentaciones(&params).await?;

    assert!(response.total_rows > 0);
    assert!(!response.results.is_empty());
    // Verify we got results (can't check nregistro since it's not in PresentacionList)

    Ok(())
}

#[tokio::test]
async fn test_get_problemas_suministro_all() -> Result<()> {
    let client = create_client()?;

    let response = client.get_problemas_suministro_all().await?;

    assert!(response.total_rows > 0);
    assert_eq!(response.page, 1);
    assert!(!response.results.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_problemas_suministro_by_cn() -> Result<()> {
    let client = create_client()?;

    // First get all problems to find a CN with issues
    let all_response = client.get_problemas_suministro_all().await?;

    if let Some(first_problem) = all_response.results.first() {
        let cn = &first_problem.cn;
        let response = client.get_problemas_suministro(cn).await?;

        assert!(response.total_rows > 0);
        assert!(response.results.iter().all(|p| p.cn == *cn));
    }

    Ok(())
}

#[tokio::test]
async fn test_get_notas_seguridad() -> Result<()> {
    let client = create_client()?;

    // Use a known medication with safety notes
    let notas = client.get_notas_seguridad("72112").await?;

    // This medication should have safety notes based on our earlier testing
    assert!(!notas.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_materiales_informativos() -> Result<()> {
    let client = create_client()?;

    // Use the same medication - this now returns a single Material object
    let _material = client.get_materiales_informativos("72112").await?;

    // Materiales returns a single Material object, just verify the call succeeded
    // The object may have empty lists if no materials are available

    Ok(())
}

#[tokio::test]
async fn test_get_maestra_by_nombre() -> Result<()> {
    let client = create_client()?;

    let params = MaestraParams {
        nombre: Some("par".to_string()),
        ..Default::default()
    };

    let response = client
        .get_maestra(TipoMaestra::PrincipiosActivos, &params)
        .await?;

    assert!(response.total_rows > 0);
    assert!(!response.results.is_empty());
    // Check that results contain "par" in the name
    assert!(
        response
            .results
            .iter()
            .any(|item| item.nombre.to_lowercase().contains("par"))
    );

    Ok(())
}

#[tokio::test]
async fn test_get_maestra_by_id() -> Result<()> {
    let client = create_client()?;

    let params = MaestraParams {
        id: Some(12), // PARACETAMOL
        ..Default::default()
    };

    let response = client
        .get_maestra(TipoMaestra::PrincipiosActivos, &params)
        .await?;

    assert!(response.total_rows > 0);
    assert!(response.results.iter().all(|item| item.id == Some(12)));

    Ok(())
}

#[tokio::test]
async fn test_get_maestra_formas_farmaceuticas() -> Result<()> {
    let client = create_client()?;

    let params = MaestraParams {
        nombre: Some("comp".to_string()),
        ..Default::default()
    };

    let response = client
        .get_maestra(TipoMaestra::FormasFarmaceuticas, &params)
        .await?;

    assert!(response.total_rows > 0);
    assert!(!response.results.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_registro_cambios() -> Result<()> {
    let client = create_client()?;

    // Use a recent date to get some changes
    let response = client.get_registro_cambios("01/01/2024", None).await?;

    // There should be MANY changes since 2024
    assert!(response.total_rows > 100);
    assert!(!response.results.is_empty());

    Ok(())
}

#[tokio::test]
#[ignore] // API BUG: Passing nregistro parameter returns 500 error for numeric IDs,
// null response for complex IDs. Verified via curl - this is a CIMA API issue.
async fn test_get_registro_cambios_specific_med() -> Result<()> {
    let client = create_client()?;

    // This test would work if the API supported the nregistro parameter correctly
    // Workaround: Fetch all and filter client-side
    let response = client
        .get_registro_cambios("01/01/2020", Some(&["72112"]))
        .await?;

    // Filter to only show changes for this specific medication
    assert!(response.results.iter().all(|c| c.nregistro == "72112"));

    Ok(())
}

#[tokio::test]
async fn test_search_vmpp() -> Result<()> {
    let client = create_client()?;

    // Search by name
    let params = cima_rs::endpoints::SearchVmppParams {
        nombre: Some("paracetamol".to_string()),
        ..Default::default()
    };

    let response = client.search_vmpp(&params).await?;

    // Should get some results for paracetamol
    assert!(response.total_rows > 0);
    assert!(!response.results.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_pagination_works() -> Result<()> {
    let client = create_client()?;

    let params = SearchMedicamentosParams {
        nombre: Some("a".to_string()), // Short name to get many results
        pagina: Some(2),               // Request second page
        ..Default::default()
    };

    let response = client.search_medicamentos(&params).await?;

    if response.total_rows > response.page_size {
        // If there are multiple pages, verify we got page 2
        assert_eq!(response.page, 2);
    }

    Ok(())
}
