use anyhow::Context;
use std::fs;
use std::io::{self, Cursor};
use std::path::PathBuf;
use zip::ZipArchive;

const NOMENCLATOR_DUMP_URL: &str = "https://listadomedicamentos.aemps.gob.es/prescripcion.zip";

/// Official download URL for the monthly Nomenclátor de Facturación CSV from Ministerio de Sanidad.
pub const BILLING_NOMENCLATOR_EXPORT_URL: &str = "https://www.sanidad.gob.es/profesionales/nomenclator.do?metodo=buscarProductos&especialidad=%25%25%25&d-4015021-e=1&6578706f7274=1";

/// Downloads and extracts the Nomenclator dump into the specified directory.
pub async fn download_and_extract_nomenclator<P: AsRef<std::path::Path>>(
    target_dir: P,
) -> anyhow::Result<PathBuf> {
    let target_dir = target_dir.as_ref().to_path_buf();

    if target_dir.exists()
        && fs::read_dir(&target_dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
    {
        return Ok(target_dir);
    }

    fs::create_dir_all(&target_dir).context("Failed to create target directory")?;

    let response = reqwest::get(NOMENCLATOR_DUMP_URL)
        .await
        .context("Failed to download nomenclator dump")?;

    let content = response
        .bytes()
        .await
        .context("Failed to read response bytes")?;
    let reader = Cursor::new(content);
    let mut archive = ZipArchive::new(reader).context("Failed to open zip archive")?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .context("Failed to access file in zip")?;
        let outpath = target_dir.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).context("Failed to create subdirectory")?;
        } else {
            if let Some(p) = outpath.parent()
                && !p.exists()
            {
                fs::create_dir_all(p).context("Failed to create parent directory")?;
            }
            let mut outfile = fs::File::create(&outpath).context("Failed to create output file")?;
            io::copy(&mut file, &mut outfile).context("Failed to copy file content")?;
        }
    }

    Ok(target_dir)
}

/// Downloads the official monthly Nomenclátor de Facturación CSV from Ministerio de Sanidad
/// into the specified directory (`nomenclator_facturacion.csv`).
pub async fn download_billing_nomenclator<P: AsRef<std::path::Path>>(
    target_dir: P,
) -> anyhow::Result<PathBuf> {
    let target_dir = target_dir.as_ref().to_path_buf();
    fs::create_dir_all(&target_dir).context("Failed to create billing target directory")?;

    let outpath = target_dir.join("nomenclator_facturacion.csv");

    if outpath.exists() && fs::metadata(&outpath).map(|m| m.len() > 0).unwrap_or(false) {
        return Ok(outpath);
    }

    let response = reqwest::get(BILLING_NOMENCLATOR_EXPORT_URL)
        .await
        .context("Failed to download Nomenclátor de Facturación CSV from Ministerio de Sanidad")?;

    let content = response
        .bytes()
        .await
        .context("Failed to read response bytes")?;

    fs::write(&outpath, content)
        .context("Failed to write Nomenclátor de Facturación CSV to file")?;

    Ok(outpath)
}

/// Official European Medicines Agency (EMA) IRIS Parallel Distribution Register portal URL.
pub const EMA_IRIS_REGISTER_URL: &str = "https://iris.ema.europa.eu/registerpd/";

/// Downloads an EMA Parallel Distribution Register CSV into `target_dir` as `ema_parallel_distribution.csv`.
///
/// Priority order for the download source:
/// 1. Explicit `url` argument if provided.
/// 2. `EMA_PARALLEL_DISTRIBUTION_URL` environment variable if set.
/// 3. Default official portal `EMA_IRIS_REGISTER_URL` (`https://iris.ema.europa.eu/registerpd/`).
///
/// If a direct CSV or mirror URL is provided, it downloads and caches the file directly.
/// If the URL points to the EMA IRIS web portal, it initiates an automated portal session handshake
/// (fetching anti-forgery tokens from `/_layout/tokenhtml` and requesting the export service). If the
/// portal rejects automated non-interactive access (PowerPages anti-automation security challenge),
/// it returns a clear descriptive error allowing callers to fall back to the built-in curated baseline.
pub async fn download_ema_parallel_distribution_register<P: AsRef<std::path::Path>>(
    target_dir: P,
    url: Option<&str>,
) -> anyhow::Result<PathBuf> {
    let target_dir = target_dir.as_ref().to_path_buf();
    fs::create_dir_all(&target_dir).context("Failed to create target directory for EMA register")?;

    let outpath = target_dir.join("ema_parallel_distribution.csv");

    let download_url = url
        .map(|s| s.to_string())
        .or_else(|| std::env::var("EMA_PARALLEL_DISTRIBUTION_URL").ok())
        .unwrap_or_else(|| EMA_IRIS_REGISTER_URL.to_string());

    // If target is the official IRIS portal, attempt portal session handshake
    if download_url.contains("iris.ema.europa.eu") {
        match fetch_from_iris_portal(&download_url).await {
            Ok(content) => {
                fs::write(&outpath, content)
                    .context("Failed to write EMA Parallel Distribution CSV to file")?;
                return Ok(outpath);
            }
            Err(e) => {
                anyhow::bail!(
                    "EMA IRIS portal at '{}' requires an interactive browser session (Microsoft PowerPages/Dynamics): {}. Please export the CSV directly from the portal, drop 'ema_parallel_distribution.csv' into your work directory, or specify a direct mirror URL via --ema-register <URL>.",
                    download_url,
                    e
                );
            }
        }
    }

    // Direct download (CSV, TSV, or mirror URL)
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let response = client
        .get(&download_url)
        .send()
        .await
        .with_context(|| format!("Failed to download EMA register from {}", download_url))?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP request to {} returned status {}", download_url, response.status());
    }

    let content = response
        .bytes()
        .await
        .context("Failed to read EMA response bytes")?;

    // Check if the received content is HTML rather than tabular data
    let preview = std::str::from_utf8(&content[..std::cmp::min(content.len(), 256)]).unwrap_or("");
    if preview.trim_start().starts_with("<!DOCTYPE") || preview.trim_start().starts_with("<html") {
        anyhow::bail!(
            "Received HTML webpage instead of CSV dataset from {}. Please provide a direct download URL or export file.",
            download_url
        );
    }

    fs::write(&outpath, content)
        .context("Failed to write EMA Parallel Distribution CSV to file")?;

    Ok(outpath)
}

/// Attempts automated session retrieval from the EMA IRIS Microsoft PowerPages portal.
async fn fetch_from_iris_portal(portal_url: &str) -> anyhow::Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    // 1. Visit main portal page
    let page_resp = client.get(portal_url).send().await?;
    let mut cookies = Vec::new();
    for cookie in page_resp.headers().get_all("set-cookie") {
        if let Ok(c_str) = cookie.to_str()
            && let Some(c_val) = c_str.split(';').next()
        {
            cookies.push(c_val.to_string());
        }
    }
    let page_html = page_resp.text().await?;

    // 2. Request anti-forgery verification token from _layout/tokenhtml
    let token_url = "https://iris.ema.europa.eu/_layout/tokenhtml";
    let mut token_req = client.get(token_url);
    if !cookies.is_empty() {
        token_req = token_req.header("Cookie", cookies.join("; "));
    }
    let token_resp = token_req.send().await?;
    for cookie in token_resp.headers().get_all("set-cookie") {
        if let Ok(c_str) = cookie.to_str()
            && let Some(c_val) = c_str.split(';').next()
        {
            cookies.push(c_val.to_string());
        }
    }
    let token_html = token_resp.text().await?;

    let token = token_html
        .split("value=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .context("Could not extract __RequestVerificationToken from IRIS portal")?;

    // 3. Extract service URL and view layout from page HTML
    let service_id = page_html
        .split("/_services/download-as-excel/")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap_or("7b138792-1090-45b6-9241-8f8d96d8c372");

    let download_service_url = format!("https://iris.ema.europa.eu/_services/download-as-excel/{}", service_id);

    let view_layout = page_html
        .split("data-view-layouts=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap_or("");

    let mut post_req = client
        .post(&download_service_url)
        .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
        .header("X-Requested-With", "XMLHttpRequest");

    if !cookies.is_empty() {
        post_req = post_req.header("Cookie", cookies.join("; "));
    }

    let form_body = format!(
        "__RequestVerificationToken={}&viewLayout={}&sortExpression={}&filter=&page=1&pageSize=100",
        urlencoding::encode(token),
        urlencoding::encode(view_layout),
        urlencoding::encode("ema_name ASC"),
    );

    let post_resp = post_req
        .body(form_body)
        .send()
        .await?;

    let bytes = post_resp.bytes().await?;

    if bytes.starts_with(b"{\"Message\":\"Invalid Request.\"}")
        || bytes.starts_with(b"<!DOCTYPE")
        || bytes.starts_with(b"<html")
    {
        anyhow::bail!("Portal rejected automated download request (anti-automation session challenge)");
    }

    Ok(bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access to external AEMPS server
    async fn test_download_and_extract() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target_dir = temp_dir.path().join("nomenclator");
        fs::create_dir_all(&target_dir).unwrap();
        let result = download_and_extract_nomenclator(&target_dir).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), target_dir);
    }

    #[tokio::test]
    #[ignore] // Requires network access to external Ministerio server
    async fn test_download_billing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target_dir = temp_dir.path().join("billing");
        fs::create_dir_all(&target_dir).unwrap();
        let result = download_billing_nomenclator(&target_dir).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            target_dir.join("nomenclator_facturacion.csv")
        );
    }
}
