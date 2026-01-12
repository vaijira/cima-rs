use cima_rs::downloader::download_and_extract_nomenclator;
use cima_rs::parser::{
    parse_atc_xml_to_csv, parse_dcp_xml_to_csv, parse_dcpf_xml_to_csv, parse_dcsa_xml_to_csv,
    parse_envases_xml_to_csv, parse_excipientes_xml_to_csv,
    parse_forma_farmaceutica_simplificada_xml_to_csv, parse_forma_farmaceutica_xml_to_csv,
    parse_laboratorio_xml_to_csv, parse_prescription_xml_to_csvs,
    parse_principio_activo_xml_to_csv, parse_situacion_registro_xml_to_csv,
    parse_unidad_contenido_xml_to_csv, parse_via_administracion_xml_to_csv,
};
use clap::Parser;
use futures::stream::{self, StreamExt};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A tool to download and convert AEMPS Nomenclator XML files to CSV.",
    long_about = "This tool automates the process of downloading the latest prescription data from AEMPS, \
                  extracting the XML files, and parsing them into specialized CSV files suitable for \
                  PostgreSQL import. Parsing is performed in parallel based on available CPU cores."
)]
struct Args {
    /// Directory where the generated CSV files will be stored.
    #[arg(
        short,
        long,
        default_value = "csv_output",
        help = "Output directory for CSV files"
    )]
    output_dir: PathBuf,

    /// Directory where the downloaded XML files will be extracted and stored.
    #[arg(
        short,
        long,
        default_value = "nomenclator_data",
        help = "Working directory for XML files"
    )]
    work_dir: PathBuf,

    /// Number of concurrent parsing tasks (defaults to number of CPU cores)
    #[arg(short, long, help = "Number of concurrent parsing tasks")]
    concurrency: Option<usize>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Ensure directories exist
    fs::create_dir_all(&args.output_dir)?;
    fs::create_dir_all(&args.work_dir)?;

    // Determine concurrency level based on CPU cores
    let num_cores = num_cpus::get();
    let concurrency = args.concurrency.unwrap_or(num_cores);

    println!("Target work directory: {:?}", args.work_dir);
    println!("Target output directory: {:?}", args.output_dir);
    println!("Available CPU cores: {}", num_cores);
    println!("Concurrency level: {}", concurrency);

    // 1. Download and extract
    println!("\nDownloading and extracting AEMPS Nomenclator data...");
    download_and_extract_nomenclator(&args.work_dir).await?;

    // 2. Define files to parse
    let mapping = vec![
        (
            "DICCIONARIO_ATC.xml",
            "atc.csv",
            parse_atc_xml_to_csv as fn(PathBuf, PathBuf) -> anyhow::Result<()>,
        ),
        (
            "DICCIONARIO_DCP.xml",
            "dcp.csv",
            parse_dcp_xml_to_csv as fn(PathBuf, PathBuf) -> anyhow::Result<()>,
        ),
        ("DICCIONARIO_DCPF.xml", "dcpf.csv", parse_dcpf_xml_to_csv),
        ("DICCIONARIO_DCSA.xml", "dcsa.csv", parse_dcsa_xml_to_csv),
        (
            "DICCIONARIO_ENVASES.xml",
            "envases.csv",
            parse_envases_xml_to_csv,
        ),
        (
            "DICCIONARIO_EXCIPIENTES_DECL_OBLIGATORIA.xml",
            "excipientes.csv",
            parse_excipientes_xml_to_csv,
        ),
        (
            "DICCIONARIO_FORMA_FARMACEUTICA.xml",
            "forma_farmaceutica.csv",
            parse_forma_farmaceutica_xml_to_csv,
        ),
        (
            "DICCIONARIO_FORMA_FARMACEUTICA_SIMPLIFICADAS.xml",
            "forma_farmaceutica_simplificada.csv",
            parse_forma_farmaceutica_simplificada_xml_to_csv,
        ),
        (
            "DICCIONARIO_LABORATORIOS.xml",
            "laboratorios.csv",
            parse_laboratorio_xml_to_csv,
        ),
        (
            "DICCIONARIO_PRINCIPIOS_ACTIVOS.xml",
            "principios_activos.csv",
            parse_principio_activo_xml_to_csv,
        ),
        (
            "DICCIONARIO_SITUACION_REGISTRO.xml",
            "situacion_registro.csv",
            parse_situacion_registro_xml_to_csv,
        ),
        (
            "DICCIONARIO_UNIDAD_CONTENIDO.xml",
            "unidad_contenido.csv",
            parse_unidad_contenido_xml_to_csv,
        ),
        (
            "DICCIONARIO_VIAS_ADMINISTRACION.xml",
            "vias_administracion.csv",
            parse_via_administracion_xml_to_csv,
        ),
        // Note: Prescripcion.xml is handled separately below (generates multiple CSVs)
    ];

    // 3. Process dictionary files in parallel using tokio streams
    println!(
        "\nParsing {} dictionary files with concurrency level {}...\n",
        mapping.len(),
        concurrency
    );

    let results: Vec<_> = stream::iter(mapping)
        .map(|(xml_name, csv_name, parser_fn)| {
            let xml_path = args.work_dir.join(xml_name);
            let csv_path = args.output_dir.join(csv_name);
            let xml_name = xml_name.to_string();
            let csv_name = csv_name.to_string();

            async move {
                if !xml_path.exists() {
                    println!("⚠️  Warning: File not found, skipping: {}", xml_name);
                    return Ok((xml_name, csv_name, false));
                }

                // Spawn blocking task for CPU-bound XML parsing
                let result =
                    tokio::task::spawn_blocking(move || parser_fn(xml_path, csv_path)).await;

                match result {
                    Ok(Ok(())) => {
                        println!("✓ Completed: {} -> {}", xml_name, csv_name);
                        Ok((xml_name, csv_name, true))
                    }
                    Ok(Err(e)) => {
                        eprintln!("✗ Failed: {} - Error: {}", xml_name, e);
                        Err(e)
                    }
                    Err(e) => {
                        eprintln!("✗ Task failed: {} - Error: {}", xml_name, e);
                        Err(anyhow::anyhow!("Task join error: {}", e))
                    }
                }
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    // 4. Handle Prescription XML separately (generates multiple CSVs)
    let prescription_result = {
        let xml_path = args.work_dir.join("Prescripcion.xml");
        if xml_path.exists() {
            println!("\n📦 Parsing Prescripcion.xml to 7 CSV files...\n");
            match parse_prescription_xml_to_csvs(&xml_path, &args.output_dir) {
                Ok(()) => {
                    println!("✓ Completed: prescriptions.csv");
                    println!("✓ Completed: prescription_forms.csv");
                    println!("✓ Completed: prescription_active_ingredients.csv");
                    println!("✓ Completed: prescription_admin_routes.csv");
                    println!("✓ Completed: prescription_atc.csv");
                    println!("✓ Completed: prescription_atc_duplicates.csv");
                    println!("✓ Completed: prescription_supply_problems.csv");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("✗ Failed: Prescripcion.xml - Error: {}", e);
                    Err(e)
                }
            }
        } else {
            println!("⚠️  Warning: Prescripcion.xml not found, skipping");
            Ok(())
        }
    };

    // 5. Report results
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let failed = results.iter().filter(|r| r.is_err()).count();
    let prescription_success = prescription_result.is_ok();

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Summary:");
    println!("  ✓ Dictionary files successful: {}", successful);
    if failed > 0 {
        println!("  ✗ Dictionary files failed: {}", failed);
    }
    if prescription_success {
        println!("  ✓ Prescription parsing: Success (7 CSV files)");
    } else {
        println!("  ✗ Prescription parsing: Failed");
    }
    println!("  📁 Output directory: {:?}", args.output_dir);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if failed > 0 || !prescription_success {
        anyhow::bail!("Some files failed to parse");
    }

    Ok(())
}
