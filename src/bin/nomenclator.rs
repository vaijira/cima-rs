use cima_rs::downloader::download_and_extract_nomenclator;
use cima_rs::parser::{
    parse_atc_xml_to_csv, parse_dcp_xml_to_csv, parse_dcpf_xml_to_csv, parse_dcsa_xml_to_csv,
    parse_envases_xml_to_csv, parse_excipientes_xml_to_csv,
    parse_forma_farmaceutica_simplificada_xml_to_csv, parse_forma_farmaceutica_xml_to_csv,
    parse_laboratorio_xml_to_csv, parse_prescription_xml_to_csvs,
    parse_principio_activo_xml_to_csv, parse_situacion_registro_xml_to_csv,
    parse_unidad_contenido_xml_to_csv, parse_via_administracion_xml_to_csv,
};
use cima_rs::{
    CimaClient, MasterDataParams, MasterDataType, SearchMedicationsParams,
    SearchPresentationsParams,
};
use clap::{Parser, Subcommand};
use futures::stream::{self, StreamExt};
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A tool to work with AEMPS CIMA nomenclator data",
    long_about = "This tool provides access to AEMPS CIMA (Centro de Información Online de Medicamentos) \
                  data through both XML/CSV conversion and REST API queries."
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Download XML files and convert to CSV format
    Csv {
        /// Directory where the generated CSV files will be stored
        #[arg(
            short,
            long,
            default_value = "csv_output",
            help = "Output directory for CSV files"
        )]
        output_dir: PathBuf,

        /// Directory where the downloaded XML files will be extracted and stored
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
    },
    /// Query the CIMA REST API
    Api {
        #[command(subcommand)]
        api_command: ApiCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ApiCommands {
    /// Query medication information
    Medicamento {
        /// Registration number
        #[arg(long, group = "identifier")]
        nregistro: Option<String>,

        /// National code
        #[arg(long, group = "identifier")]
        cn: Option<String>,

        /// Show presentations
        #[arg(short, long)]
        presentaciones: bool,

        /// Show active ingredients
        #[arg(short, long)]
        activos: bool,
    },
    /// Search medications
    SearchMedicamentos {
        /// Medication name
        #[arg(long)]
        nombre: Option<String>,

        /// Laboratory name
        #[arg(long)]
        laboratorio: Option<String>,

        /// Active ingredient name
        #[arg(long)]
        principio_activo: Option<String>,

        /// ATC code or description
        #[arg(long)]
        atc: Option<String>,

        /// Only commercialized medications
        #[arg(long)]
        comercializados: bool,

        /// Only orphan medications
        #[arg(long)]
        huerfanos: bool,

        /// Only medications with black triangle
        #[arg(long)]
        triangulo: bool,

        /// Limit results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Query presentation information
    Presentacion {
        /// National code
        #[arg(long)]
        cn: String,
    },
    /// Search presentations
    SearchPresentaciones {
        /// Registration number
        #[arg(long)]
        nregistro: Option<String>,

        /// VMP code
        #[arg(long)]
        vmp: Option<String>,

        /// Only commercialized
        #[arg(long)]
        comercializados: bool,

        /// Limit results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Get supply problems
    SupplyProblems {
        /// National code (if not provided, returns all)
        #[arg(long)]
        cn: Option<String>,
    },
    /// Get safety notes for a medication
    SafetyNotes {
        /// Registration number
        #[arg(long)]
        nregistro: String,
    },
    /// Get change log
    Changes {
        /// Date from which to get changes (format: dd/mm/yyyy)
        #[arg(long)]
        desde: String,

        /// Limit to specific registration numbers
        #[arg(long)]
        nregistro: Vec<String>,
    },
    /// Query master data catalogs
    Maestra {
        /// Type of master data: pa (principios activos), ff (formas farmaceuticas),
        /// va (vias administracion), lab (laboratorios), atc (codigos ATC)
        #[arg(long)]
        tipo: String,

        /// Name filter
        #[arg(long)]
        nombre: Option<String>,

        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    match args.command {
        Commands::Csv {
            output_dir,
            work_dir,
            concurrency,
        } => process_csv(output_dir, work_dir, concurrency).await,
        Commands::Api { api_command } => process_api(api_command).await,
    }
}

async fn process_csv(
    output_dir: PathBuf,
    work_dir: PathBuf,
    concurrency: Option<usize>,
) -> anyhow::Result<()> {
    // Ensure directories exist
    fs::create_dir_all(&output_dir)?;
    fs::create_dir_all(&work_dir)?;

    // Determine concurrency level based on CPU cores
    let num_cores = num_cpus::get();
    let concurrency = concurrency.unwrap_or(num_cores);

    tracing::info!(work_dir = ?work_dir, "Target work directory");
    tracing::info!(output_dir = ?output_dir, "Target output directory");
    tracing::info!(num_cores, "Available CPU cores");
    tracing::info!(concurrency, "Concurrency level");

    // 1. Download and extract
    tracing::info!("Downloading and extracting AEMPS Nomenclator data");
    download_and_extract_nomenclator(&work_dir).await?;

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
    tracing::info!(
        file_count = mapping.len(),
        concurrency,
        "Parsing dictionary files"
    );

    let results: Vec<_> = stream::iter(mapping)
        .map(|(xml_name, csv_name, parser_fn)| {
            let xml_path = work_dir.join(xml_name);
            let csv_path = output_dir.join(csv_name);
            let xml_name = xml_name.to_string();
            let csv_name = csv_name.to_string();

            async move {
                if !xml_path.exists() {
                    tracing::warn!(file = %xml_name, "File not found, skipping");
                    return Ok((xml_name, csv_name, false));
                }

                // Spawn blocking task for CPU-bound XML parsing
                tracing::debug!(xml = %xml_name, csv = %csv_name, "Starting parse task");
                let result =
                    tokio::task::spawn_blocking(move || parser_fn(xml_path, csv_path)).await;

                match result {
                    Ok(Ok(())) => {
                        tracing::info!(xml = %xml_name, csv = %csv_name, "Completed parse");
                        Ok((xml_name, csv_name, true))
                    }
                    Ok(Err(e)) => {
                        tracing::error!(xml = %xml_name, error = %e, "Parse failed");
                        Err(e)
                    }
                    Err(e) => {
                        tracing::error!(xml = %xml_name, error = %e, "Task join failed");
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
        let xml_path = work_dir.join("Prescripcion.xml");
        if xml_path.exists() {
            tracing::info!("Parsing Prescripcion.xml to 7 CSV files");
            match parse_prescription_xml_to_csvs(&xml_path, &output_dir) {
                Ok(()) => {
                    tracing::info!("Completed all prescription CSV files");
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
                    tracing::error!(error = ?e, "Failed to parse Prescripcion.xml");
                    // Print full error chain for debugging
                    eprintln!("Prescription parse error: {:#}", e);
                    Err(e)
                }
            }
        } else {
            tracing::warn!("Prescripcion.xml not found, skipping");
            Ok(())
        }
    };

    // 5. Report results
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let failed = results.iter().filter(|r| r.is_err()).count();
    let prescription_success = prescription_result.is_ok();

    tracing::info!(
        successful,
        failed,
        prescription_success,
        "CSV parsing completed"
    );

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
    println!("  📁 Output directory: {:?}", output_dir);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if failed > 0 || !prescription_success {
        anyhow::bail!("Some files failed to parse");
    }

    Ok(())
}

async fn process_api(api_command: ApiCommands) -> anyhow::Result<()> {
    tracing::debug!("Creating CIMA client for API query");
    let client = CimaClient::new()?;

    match api_command {
        ApiCommands::Medicamento {
            nregistro,
            cn,
            presentaciones,
            activos,
        } => {
            let med = client
                .get_medication(nregistro.as_deref(), cn.as_deref())
                .await?;

            println!("=== Medicamento ===");
            println!("Nº Registro: {}", med.nregistro);
            println!("Nombre: {}", med.name);
            println!("Laboratorio: {}", med.labtitular);
            println!("Principios Activos: {}", med.pactivos);
            println!("Condiciones de prescripción: {}", med.cpresc);

            if let Some(comerc) = med.commercialized {
                println!("Comercializado: {}", if comerc { "Sí" } else { "No" });
            }

            if let Some(triangulo) = med.black_triangle
                && triangulo
            {
                println!("⚠️  Triángulo negro (medicamento bajo vigilancia adicional)");
            }

            if let Some(huerfano) = med.orphan
                && huerfano
            {
                println!("💊 Medicamento huérfano");
            }

            if activos && !med.active_ingredients.is_empty() {
                println!("\n=== Principios Activos ===");
                for pa in &med.active_ingredients {
                    print!("- {}", pa.name);
                    if let (Some(cantidad), Some(unidad)) = (&pa.amount, &pa.unit) {
                        print!(": {} {}", cantidad, unidad);
                    }
                    println!();
                }
            }

            if presentaciones && !med.presentations.is_empty() {
                println!("\n=== Presentaciones ===");
                for pres in &med.presentations {
                    println!("- CN: {} - {}", pres.cn, pres.name);
                    if pres.commercialized {
                        println!("  ✓ Comercializada");
                    }
                }
            }

            if !med.docs.is_empty() {
                println!("\n=== Documentos Disponibles ===");
                for doc in &med.docs {
                    let tipo = match doc.doc_type {
                        1 => "Ficha Técnica",
                        2 => "Prospecto",
                        3 => "Informe Público Evaluación",
                        4 => "Plan de gestión de riesgos",
                        _ => "Otro",
                    };
                    println!("- {}: {}", tipo, doc.url);
                }
            }
        }
        ApiCommands::SearchMedicamentos {
            nombre,
            laboratorio,
            principio_activo,
            atc,
            comercializados,
            huerfanos,
            triangulo,
            limit,
        } => {
            let params = SearchMedicationsParams {
                name: nombre,
                laboratory: laboratorio,
                active_ingredient_1: principio_activo,
                atc,
                commercialized: if comercializados { Some(1) } else { None },
                orphan: if huerfanos { Some(1) } else { None },
                black_triangle: if triangulo { Some(1) } else { None },
                ..Default::default()
            };

            let response = client.search_medications(&params).await?;

            tracing::info!(
                "Found {} total medications (page {} of {}, showing {} results)",
                response.total_rows,
                response.page,
                response.total_rows.div_ceil(response.page_size),
                response.results.len()
            );

            for (i, med) in response.results.iter().enumerate().take(limit) {
                println!("{}. {} ({})", i + 1, med.name, med.nregistro);
                println!("   Laboratorio: {}", med.labtitular);
                if let Some(comerc) = med.commercialized {
                    println!("   Comercializado: {}", if comerc { "Sí" } else { "No" });
                }
                println!();
            }

            if response.results.len() > limit {
                tracing::info!(
                    "Showing {} of {} results from page",
                    limit,
                    response.results.len()
                );
            }
        }
        ApiCommands::Presentacion { cn } => {
            let pres = client.get_presentation(&cn).await?;

            println!("=== Presentación ===");
            println!("Código Nacional: {}", pres.cn);
            println!("Nombre: {}", pres.name);
            println!(
                "Comercializada: {}",
                if pres.commercialized { "Sí" } else { "No" }
            );
        }
        ApiCommands::SearchPresentaciones {
            nregistro,
            vmp,
            comercializados,
            limit,
        } => {
            let params = SearchPresentationsParams {
                registration_number: nregistro,
                vmp,
                commercialized: if comercializados { Some(1) } else { None },
                ..Default::default()
            };

            let response = client.search_presentations(&params).await?;

            tracing::info!(
                "Found {} total presentations (page {} of {}, showing {} results)",
                response.total_rows,
                response.page,
                response.total_rows.div_ceil(response.page_size),
                response.results.len()
            );

            for (i, p) in response.results.iter().enumerate().take(limit) {
                println!("{}. CN: {} - {}", i + 1, p.cn, p.name);
                if p.commercialized {
                    println!("   ✓ Comercializada");
                }
                println!();
            }

            if response.results.len() > limit {
                tracing::info!(
                    "Showing {} of {} results from page",
                    limit,
                    response.results.len()
                );
            }
        }
        ApiCommands::SupplyProblems { cn } => {
            if let Some(codigo) = cn {
                let response = client.get_supply_problems(&codigo).await?;
                tracing::info!(
                    "Found {} supply problems for CN {} (page {} of {})",
                    response.total_rows,
                    codigo,
                    response.page,
                    response.total_rows.div_ceil(response.page_size)
                );

                for (i, prob) in response.results.iter().enumerate() {
                    println!("{}. CN: {} - {}", i + 1, prob.cn, prob.name);
                    println!("   Activo: {}", if prob.active { "Sí" } else { "No" });
                    if let Some(obs) = &prob.observations {
                        println!("   Observaciones: {}", obs);
                    }
                    println!();
                }
            } else {
                let response = client.get_all_supply_problems().await?;
                tracing::info!(
                    "Found {} total supply problems (page {} of {})",
                    response.total_rows,
                    response.page,
                    response.total_rows.div_ceil(response.page_size)
                );

                for (i, prob) in response.results.iter().enumerate() {
                    println!("{}. CN: {} - {}", i + 1, prob.cn, prob.name);
                    println!("   Activo: {}", if prob.active { "Sí" } else { "No" });
                    if let Some(obs) = &prob.observations {
                        println!("   Observaciones: {}", obs);
                    }
                    println!();
                }
            }
        }
        ApiCommands::SafetyNotes { nregistro } => {
            let notas = client.get_safety_notes(&nregistro).await?;

            println!("Notas de Seguridad: {}\n", notas.len());

            for (i, nota) in notas.iter().enumerate() {
                println!("{}. {} - {}", i + 1, nota.num, nota.subject);
                println!("   URL: {}", nota.url);
                println!();
            }
        }
        ApiCommands::Changes { desde, nregistro } => {
            let nregs: Vec<&str> = nregistro.iter().map(|s| s.as_str()).collect();
            let nregs_opt = if nregs.is_empty() {
                None
            } else {
                Some(nregs.as_slice())
            };

            let response = client.get_change_log(&desde, nregs_opt).await?;

            tracing::info!(
                "Found {} total changes since {} (page {} of {})",
                response.total_rows,
                desde,
                response.page,
                response.total_rows.div_ceil(response.page_size)
            );

            for (i, cambio) in response.results.iter().enumerate() {
                println!("{}. Nº Registro: {}", i + 1, cambio.nregistro);
                let tipo = match cambio.change_type {
                    1 => "Nuevo",
                    2 => "Baja",
                    3 => "Modificado",
                    _ => "Desconocido",
                };
                println!("   Tipo: {}", tipo);
                if !cambio.changes.is_empty() {
                    println!("   Cambios: {}", cambio.changes.join(", "));
                }
                println!();
            }
        }
        ApiCommands::Maestra {
            tipo,
            nombre,
            limit,
        } => {
            // Validate that at least one filter parameter is provided (API requires this)
            if nombre.is_none() {
                tracing::warn!(
                    "No filter parameters provided. The CIMA API requires at least one filter parameter (nombre, id, codigo, etc.)"
                );
                tracing::warn!(
                    "The maestra CLI currently only supports --nombre. Other parameters can be used via the library API."
                );
                eprintln!("Error: The --nombre parameter is required for this command");
                eprintln!(
                    "(The API supports id, codigo, estupefaciente, etc., but the CLI currently only exposes --nombre)"
                );
                eprintln!("Example: nomenclator api maestra --tipo pa --nombre 'paracetamol'");
                std::process::exit(1);
            }

            let tipo_maestra = match tipo.as_str() {
                "pa" => MasterDataType::ActiveIngredients,
                "ff" => MasterDataType::PharmaceuticalForms,
                "va" => MasterDataType::AdministrationRoutes,
                "lab" => MasterDataType::Laboratories,
                "atc" => MasterDataType::AtcCodes,
                _ => anyhow::bail!(
                    "Tipo de maestra desconocido: {}. Use: pa, ff, va, lab, atc",
                    tipo
                ),
            };

            let params = MasterDataParams {
                name: nombre,
                ..Default::default()
            };

            let response = client.get_master_data(tipo_maestra, &params).await?;

            tracing::info!(
                "Found {} total items (page {} of {})",
                response.total_rows,
                response.page,
                response.total_rows.div_ceil(response.page_size)
            );

            for (i, item) in response.results.iter().enumerate().take(limit) {
                print!("{}. {}", i + 1, item.name);
                if let Some(codigo) = &item.code {
                    print!(" ({})", codigo);
                } else if let Some(id) = item.id {
                    print!(" (ID: {})", id);
                }
                println!();
            }

            if response.results.len() > limit {
                tracing::info!(
                    "Showing {} of {} results from page",
                    limit,
                    response.results.len()
                );
            }
        }
    }

    Ok(())
}
