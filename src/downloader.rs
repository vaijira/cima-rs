use anyhow::Context;
use std::fs;
use std::io::{self, Cursor};
use std::path::PathBuf;
use zip::ZipArchive;

const NOMENCLATOR_DUMP_URL: &str = "https://listadomedicamentos.aemps.gob.es/prescripcion.zip";

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
}
