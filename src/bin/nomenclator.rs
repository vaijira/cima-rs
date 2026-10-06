use cima_rs::billing::{
    compute_homogeneous_groups, export_billing_data_to_csvs, extract_parallel_imports,
    generate_group_presentation_mappings, load_parallel_import_cns_from_prescriptions_csv,
    parse_billing_csv, tag_parallel_imports_from_prescriptions,
};
use cima_rs::downloader::{download_and_extract_nomenclator, download_billing_nomenclator};
use cima_rs::parser::{
    parse_atc_xml_to_csv, parse_dcp_xml_to_csv, parse_dcpf_xml_to_csv, parse_dcsa_xml_to_csv,
    parse_envases_xml_to_csv, parse_excipientes_xml_to_csv,
    parse_forma_farmaceutica_simplificada_xml_to_csv, parse_forma_farmaceutica_xml_to_csv,
    parse_laboratorio_xml_to_csv, parse_prescription_xml_to_csvs,
    parse_principio_activo_xml_to_csv, parse_situacion_registro_xml_to_csv,
    parse_unidad_contenido_xml_to_csv, parse_via_administracion_xml_to_csv,
};
use cima_rs::terminology::{build_from_aemps_csv_dir, export_terminology_to_csvs};
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
    about = "A tool to work with AEMPS CIMA and SNS Nomenclator data",
    long_about = "This tool provides access to AEMPS CIMA (Centro de Información Online de Medicamentos) \
                  and Ministerio de Sanidad Nomenclátor de Facturación (Homogeneous Groups, Prices, Parallel Imports) \
                  data through XML/CSV conversion, database-ready CSV export, and REST API queries."
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

        /// Also download and process Nomenclátor de Facturación (Homogeneous Groups, PM/PAB, Parallel Imports)
        #[arg(
            long,
            help = "Also process Nomenclátor de Facturación (Homogeneous Groups, PM/PAB, Parallel Imports)"
        )]
        include_billing: bool,

        /// Also export SNOMED CT España / AEMPS Terminology hierarchy CSVs (VTM -> VMP -> VMPP -> AMPP)
        #[arg(
            long,
            help = "Also export SNOMED CT España / AEMPS Terminology hierarchy CSVs (VTM -> VMP -> VMPP -> AMPP)"
        )]
        include_terminology: bool,
    },
    /// Query the CIMA REST API
    Api {
        #[command(subcommand)]
        api_command: ApiCommands,
    },
    /// Manage and export Nomenclátor de Facturación (Homogeneous Groups, PM/PAB Prices, Parallel Imports)
    Billing {
        #[command(subcommand)]
        billing_command: BillingCommands,
    },
    /// Manage and export SNOMED CT España / AEMPS Clinical Terminology (VTM -> VMP -> VMPP -> AMPP)
    Terminology {
        #[command(subcommand)]
        terminology_command: TerminologyCommands,
    },
}

#[derive(Subcommand, Debug)]
enum BillingCommands {
    /// Download the official Nomenclátor de Facturación CSV and export relational CSVs for database import
    Export {
        /// Directory where the generated relational CSV files will be stored
        #[arg(
            short,
            long,
            default_value = "billing_csv_output",
            help = "Output directory for relational CSV files"
        )]
        output_dir: PathBuf,

        /// Path to an existing Nomenclátor de Facturación CSV file (if not provided, downloads it automatically)
        #[arg(short, long, help = "Path to existing Nomenclátor de Facturación CSV")]
        input_file: Option<PathBuf>,

        /// Directory where the raw downloaded file will be stored
        #[arg(
            short,
            long,
            default_value = "billing_data",
            help = "Working directory for raw download"
        )]
        work_dir: PathBuf,

        /// Optional path to AEMPS prescriptions.csv to cross-reference parallel imports
        #[arg(
            long,
            help = "Optional path to AEMPS prescriptions.csv for exact parallel import matching"
        )]
        prescriptions_csv: Option<PathBuf>,
    },
    /// Query a specific Agrupación Homogénea (AH) by code
    Group {
        /// Group code (e.g. 1941)
        #[arg(long)]
        code: String,

        /// Path to Nomenclátor de Facturación CSV file
        #[arg(
            short,
            long,
            default_value = "billing_data/nomenclator_facturacion.csv",
            help = "Path to Nomenclátor de Facturación CSV"
        )]
        input_file: PathBuf,
    },
    /// List and filter parallel imports
    ParallelImports {
        /// Only show active (ALTA) parallel imports
        #[arg(long)]
        only_active: bool,

        /// Filter by homogeneous group code
        #[arg(long)]
        group_code: Option<String>,

        /// Path to Nomenclátor de Facturación CSV file
        #[arg(
            short,
            long,
            default_value = "billing_data/nomenclator_facturacion.csv",
            help = "Path to Nomenclátor de Facturación CSV"
        )]
        input_file: PathBuf,

        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
enum TerminologyCommands {
    /// Build terminology index from parsed AEMPS CSV directory and export relational database-ready CSVs
    Export {
        /// Directory containing parsed AEMPS CSVs (dcsa.csv, dcp.csv, dcpf.csv, prescriptions.csv)
        #[arg(
            short,
            long,
            default_value = "csv_output",
            help = "Input directory with parsed AEMPS CSV files"
        )]
        input_dir: PathBuf,

        /// Directory where relational terminology CSVs will be saved
        #[arg(
            short,
            long,
            default_value = "terminology_csv_output",
            help = "Output directory for relational terminology CSV files"
        )]
        output_dir: PathBuf,
    },
    /// Query the full clinical terminology hierarchy for a given Código Nacional (CN)
    Hierarchy {
        /// 6-digit National Code (Código Nacional)
        #[arg(long, help = "Código Nacional (CN)")]
        cn: String,

        /// Directory containing parsed AEMPS CSVs
        #[arg(
            short,
            long,
            default_value = "csv_output",
            help = "Directory with parsed AEMPS CSV files"
        )]
        input_dir: PathBuf,
    },
    /// Find bioequivalent commercial presentations (AMPP) sharing the same VMP (Virtual Medicinal Product)
    Equivalents {
        /// 6-digit National Code (Código Nacional) to find bioequivalents for
        #[arg(long, group = "identifier", help = "Código Nacional (CN)")]
        cn: Option<String>,

        /// Virtual Medicinal Product (VMP / DCP) code
        #[arg(long, group = "identifier", help = "VMP / DCP code")]
        vmp: Option<String>,

        /// Directory containing parsed AEMPS CSVs
        #[arg(
            short,
            long,
            default_value = "csv_output",
            help = "Directory with parsed AEMPS CSV files"
        )]
        input_dir: PathBuf,

        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Search terminology concepts across VTM (substances), VMP (virtual products), or AMPP (commercial packs)
    Search {
        /// Search text / query string
        #[arg(long, help = "Search query (e.g. 'Paracetamol' or 'Ibuprofeno')")]
        query: String,

        /// Search level: 'vtm' (substances), 'vmp' (virtual products), 'ampp' (commercial packs), or 'all'
        #[arg(long, default_value = "all")]
        level: String,

        /// Directory containing parsed AEMPS CSVs
        #[arg(
            short,
            long,
            default_value = "csv_output",
            help = "Directory with parsed AEMPS CSV files"
        )]
        input_dir: PathBuf,

        /// Limit results per section
        #[arg(short, long, default_value = "10")]
        limit: usize,
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
            include_billing,
            include_terminology,
        } => {
            process_csv(
                output_dir,
                work_dir,
                concurrency,
                include_billing,
                include_terminology,
            )
            .await
        }
        Commands::Api { api_command } => process_api(api_command).await,
        Commands::Billing { billing_command } => process_billing(billing_command).await,
        Commands::Terminology {
            terminology_command,
        } => process_terminology(terminology_command).await,
    }
}

async fn process_csv(
    output_dir: PathBuf,
    work_dir: PathBuf,
    concurrency: Option<usize>,
    include_billing: bool,
    include_terminology: bool,
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

    // 5. Handle Nomenclátor de Facturación if requested
    let billing_result: anyhow::Result<Option<cima_rs::BillingExportSummary>> = if include_billing {
        tracing::info!("Downloading and processing Nomenclátor de Facturación");
        match download_billing_nomenclator(&work_dir).await {
            Ok(billing_path) => match parse_billing_csv(&billing_path) {
                Ok(mut products) => {
                    let presc_csv = output_dir.join("prescriptions.csv");
                    if presc_csv.exists()
                        && let Ok(cns) = load_parallel_import_cns_from_prescriptions_csv(&presc_csv)
                    {
                        tag_parallel_imports_from_prescriptions(&mut products, &cns);
                    }
                    match export_billing_data_to_csvs(&products, &output_dir) {
                        Ok(summary) => {
                            println!("✓ Completed: billing_products.csv");
                            println!("✓ Completed: homogeneous_groups.csv");
                            println!("✓ Completed: group_presentations.csv");
                            println!("✓ Completed: parallel_imports.csv");
                            Ok(Some(summary))
                        }
                        Err(e) => {
                            tracing::error!(error = ?e, "Failed to export billing CSV files");
                            Err(e)
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Failed to parse Nomenclátor de Facturación");
                    Err(e)
                }
            },
            Err(e) => {
                tracing::error!(error = ?e, "Failed to download Nomenclátor de Facturación");
                Err(e)
            }
        }
    } else {
        Ok(None)
    };

    // 6. Handle Terminology if requested
    let terminology_result: anyhow::Result<Option<cima_rs::TerminologyExportSummary>> =
        if include_terminology {
            tracing::info!(
                "Building TerminologyIndex and exporting SNOMED CT España / AEMPS relational CSVs"
            );
            match build_from_aemps_csv_dir(&output_dir) {
                Ok(index) => match export_terminology_to_csvs(&index, &output_dir) {
                    Ok(summary) => {
                        println!("✓ Completed: terminology_vtm.csv");
                        println!("✓ Completed: terminology_vmp.csv");
                        println!("✓ Completed: terminology_vmpp.csv");
                        println!("✓ Completed: terminology_ampp.csv");
                        println!("✓ Completed: terminology_hierarchy.csv");
                        Ok(Some(summary))
                    }
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to export terminology CSV files");
                        Err(e)
                    }
                },
                Err(e) => {
                    tracing::error!(error = ?e, "Failed to build TerminologyIndex from parsed CSV files");
                    Err(e)
                }
            }
        } else {
            Ok(None)
        };

    // 7. Report results
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let failed = results.iter().filter(|r| r.is_err()).count();
    let prescription_success = prescription_result.is_ok();
    let billing_success = billing_result.is_ok();
    let terminology_success = terminology_result.is_ok();

    tracing::info!(
        successful,
        failed,
        prescription_success,
        billing_success,
        terminology_success,
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
    if include_billing {
        if billing_success {
            println!("  ✓ Billing export: Success (4 CSV files)");
        } else {
            println!("  ✗ Billing export: Failed");
        }
    }
    if include_terminology {
        if terminology_success {
            println!("  ✓ Terminology export: Success (5 CSV files)");
        } else {
            println!("  ✗ Terminology export: Failed");
        }
    }
    println!("  📁 Output directory: {:?}", output_dir);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if failed > 0
        || !prescription_success
        || (include_billing && !billing_success)
        || (include_terminology && !terminology_success)
    {
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

async fn process_billing(billing_command: BillingCommands) -> anyhow::Result<()> {
    match billing_command {
        BillingCommands::Export {
            output_dir,
            input_file,
            work_dir,
            prescriptions_csv,
        } => {
            let csv_path = match input_file {
                Some(p) => {
                    if !p.exists() {
                        anyhow::bail!("Input file does not exist: {:?}", p);
                    }
                    p
                }
                None => {
                    tracing::info!(
                        "Downloading Nomenclátor de Facturación from Ministerio de Sanidad"
                    );
                    download_billing_nomenclator(&work_dir).await?
                }
            };

            tracing::info!(file = ?csv_path, "Parsing Nomenclátor de Facturación");
            let mut products = parse_billing_csv(&csv_path)?;
            tracing::info!(total = products.len(), "Parsed billing products");

            let presc_path = prescriptions_csv.or_else(|| {
                let default_p = PathBuf::from("csv_output/prescriptions.csv");
                if default_p.exists() {
                    Some(default_p)
                } else {
                    None
                }
            });

            if let Some(ref p_path) = presc_path
                && p_path.exists()
            {
                tracing::info!(
                    prescriptions = ?p_path,
                    "Cross-referencing parallel imports with AEMPS prescriptions"
                );
                match load_parallel_import_cns_from_prescriptions_csv(p_path) {
                    Ok(cns) => {
                        let count = cns.len();
                        tag_parallel_imports_from_prescriptions(&mut products, &cns);
                        tracing::info!(matched_cns = count, "Tagged parallel imports from AEMPS");
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "Could not read prescriptions.csv for parallel imports, relying on heuristics"
                        );
                    }
                }
            }

            tracing::info!(output_dir = ?output_dir, "Exporting relational CSVs for database import");
            let summary = export_billing_data_to_csvs(&products, &output_dir)?;

            println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Nomenclátor de Facturación - Relational CSV Export Completed");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!(
                "  ✓ Total Products:               {}",
                summary.total_products
            );
            println!(
                "  ✓ Active Products (ALTA):       {}",
                summary.active_products
            );
            println!("  ✓ Homogeneous Groups (AH):      {}", summary.total_groups);
            println!(
                "  ✓ Group Presentation Mappings:  {}",
                summary.total_mappings
            );
            println!(
                "  ✓ Parallel Imports (I.P.):      {}",
                summary.total_parallel_imports
            );
            println!("\nExported CSV files (ready for database import):");
            println!(
                "  📄 billing_products.csv:         {:?}",
                summary.products_csv
            );
            println!(
                "  📄 homogeneous_groups.csv:       {:?}",
                summary.groups_csv
            );
            println!(
                "  📄 group_presentations.csv:      {:?}",
                summary.group_presentations_csv
            );
            println!(
                "  📄 parallel_imports.csv:         {:?}",
                summary.parallel_imports_csv
            );
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            Ok(())
        }
        BillingCommands::Group { code, input_file } => {
            if !input_file.exists() {
                anyhow::bail!(
                    "Nomenclátor CSV not found at {:?}. Run 'nomenclator billing export' or specify --input-file",
                    input_file
                );
            }
            let products = parse_billing_csv(&input_file)?;
            let groups = compute_homogeneous_groups(&products);
            let group = groups.iter().find(|g| g.code == code).ok_or_else(|| {
                anyhow::anyhow!("Homogeneous group with code '{}' not found", code)
            })?;

            println!("=== Agrupación Homogénea ===");
            println!("Código: {}", group.code);
            println!("Nombre: {}", group.name);
            if let Some(pm) = group.precio_menor {
                println!("Precio Menor (PM): {:.2} €", pm);
            } else {
                println!("Precio Menor (PM): N/A");
            }
            if let Some(pab) = group.precio_mas_bajo {
                println!("Precio Más Bajo (PAB): {:.2} €", pab);
            } else {
                println!("Precio Más Bajo (PAB): N/A");
            }
            println!("Total Presentaciones: {}", group.total_presentations);
            println!(
                "Presentaciones Activas (ALTA): {}",
                group.active_presentations
            );
            println!(
                "Tiene Importaciones Paralelas: {}",
                if group.has_parallel_imports {
                    "Sí"
                } else {
                    "No"
                }
            );

            let mappings = generate_group_presentation_mappings(&products, &groups);
            let group_mappings: Vec<_> = mappings.iter().filter(|m| m.group_code == code).collect();

            println!("\n=== Presentaciones en la agrupación ===");
            for (i, m) in group_mappings.iter().enumerate() {
                print!("{}. CN: {} - {}", i + 1, m.cn, m.product_name);
                if let Some(p) = m.pvp_iva {
                    print!(" | PVP: {:.2} €", p);
                }
                if m.is_precio_mas_bajo {
                    print!(" [★ PAB]");
                }
                if m.is_parallel_import {
                    print!(" [I.P.]");
                }
                if !m.is_active {
                    print!(" ({})", m.status);
                }
                println!();
            }
            Ok(())
        }
        BillingCommands::ParallelImports {
            only_active,
            group_code,
            input_file,
            limit,
        } => {
            if !input_file.exists() {
                anyhow::bail!(
                    "Nomenclátor CSV not found at {:?}. Run 'nomenclator billing export' or specify --input-file",
                    input_file
                );
            }
            let products = parse_billing_csv(&input_file)?;
            let mut pis = extract_parallel_imports(&products);

            if only_active {
                pis.retain(|p| p.is_active);
            }
            if let Some(ref g_code) = group_code {
                pis.retain(|p| p.homogeneous_group_code.as_deref() == Some(g_code.as_str()));
            }

            println!("=== Importaciones Paralelas ===");
            println!(
                "Total encontradas: {} (mostrando hasta {})\n",
                pis.len(),
                limit
            );

            for (i, pi) in pis.iter().take(limit).enumerate() {
                println!("{}. CN: {} - {}", i + 1, pi.cn, pi.name);
                if let Some(ref lab) = pi.supplier_lab_name {
                    println!("   Laboratorio: {}", lab);
                }
                if let Some(pvp) = pi.pvp_iva {
                    println!("   PVP con IVA: {:.2} €", pvp);
                }
                if let Some(ref g_name) = pi.homogeneous_group_name {
                    println!(
                        "   Agrupación: {} ({})",
                        g_name,
                        pi.homogeneous_group_code.as_deref().unwrap_or("")
                    );
                }
                println!(
                    "   Estado: {} (Detección: {})",
                    pi.status, pi.detection_source
                );
                println!();
            }
            Ok(())
        }
    }
}

async fn process_terminology(cmd: TerminologyCommands) -> anyhow::Result<()> {
    match cmd {
        TerminologyCommands::Export {
            input_dir,
            output_dir,
        } => {
            if !input_dir.exists() {
                anyhow::bail!(
                    "Input directory {:?} does not exist. Run 'nomenclator csv' first to generate AEMPS CSVs",
                    input_dir
                );
            }
            tracing::info!(dir = ?input_dir, "Building TerminologyIndex from AEMPS CSVs");
            let index = build_from_aemps_csv_dir(&input_dir)?;
            tracing::info!(
                vtms = index.vtms.len(),
                vmps = index.vmps.len(),
                vmpps = index.vmpps.len(),
                ampps = index.ampps.len(),
                "TerminologyIndex built successfully"
            );

            tracing::info!(output_dir = ?output_dir, "Exporting relational terminology CSVs");
            let summary = export_terminology_to_csvs(&index, &output_dir)?;

            println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("SNOMED CT España / AEMPS Terminology - Relational CSV Export");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!(
                "  ✓ VTM  (Denominación Común Sustancia Activa):  {}",
                summary.total_vtms
            );
            println!(
                "  ✓ VMP  (Denominación Común Principio Activo):  {}",
                summary.total_vmps
            );
            println!(
                "  ✓ VMPP (Denominación Común Formato/Envase):    {}",
                summary.total_vmpps
            );
            println!(
                "  ✓ AMPP (Presentaciones Comerciales / CN):      {}",
                summary.total_ampps
            );
            println!(
                "  ✓ Full Hierarchical Rows Resolved:             {}",
                summary.total_hierarchies
            );
            println!("\nExported CSV files (ready for database import):");
            println!("  📄 terminology_vtm.csv:        {:?}", summary.vtm_csv);
            println!("  📄 terminology_vmp.csv:        {:?}", summary.vmp_csv);
            println!("  📄 terminology_vmpp.csv:       {:?}", summary.vmpp_csv);
            println!("  📄 terminology_ampp.csv:       {:?}", summary.ampp_csv);
            println!(
                "  📄 terminology_hierarchy.csv:  {:?}",
                summary.hierarchy_csv
            );
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            Ok(())
        }
        TerminologyCommands::Hierarchy { cn, input_dir } => {
            if !input_dir.exists() {
                anyhow::bail!(
                    "Input directory {:?} does not exist. Run 'nomenclator csv' first",
                    input_dir
                );
            }
            let index = build_from_aemps_csv_dir(&input_dir)?;
            let hierarchy = index.lookup_by_cn(&cn).ok_or_else(|| {
                anyhow::anyhow!(
                    "No terminology hierarchy found for Código Nacional '{}'",
                    cn
                )
            })?;

            println!("=== Jerarquía Terapéutica Oficial (SNOMED CT España / REI) ===");
            println!("CN / AMPP:  {} - {}", hierarchy.cn, hierarchy.ampp_name);
            if let Some(ref lab) = hierarchy.laboratory {
                println!("Titular:    {}", lab);
            }
            println!(
                "Comerc.:    {}",
                if hierarchy.is_commercialized {
                    "Sí"
                } else {
                    "No"
                }
            );
            println!("---------------------------------------------------------------");
            if let (Some(code), Some(name)) = (&hierarchy.vmpp_code, &hierarchy.vmpp_name) {
                println!("↳ VMPP:     {} - {}", code, name);
            } else if let Some(code) = &hierarchy.vmpp_code {
                println!("↳ VMPP:     {}", code);
            }
            if let (Some(code), Some(name)) = (&hierarchy.vmp_code, &hierarchy.vmp_name) {
                println!("  ↳ VMP:    {} - {}", code, name);
            } else if let Some(code) = &hierarchy.vmp_code {
                println!("  ↳ VMP:    {}", code);
            }
            if let (Some(code), Some(name)) = (&hierarchy.vtm_code, &hierarchy.vtm_name) {
                println!("    ↳ VTM:  {} - {}", code, name);
            } else if let Some(code) = &hierarchy.vtm_code {
                println!("    ↳ VTM:  {}", code);
            }
            println!("---------------------------------------------------------------");
            Ok(())
        }
        TerminologyCommands::Equivalents {
            cn,
            vmp,
            input_dir,
            limit,
        } => {
            if !input_dir.exists() {
                anyhow::bail!(
                    "Input directory {:?} does not exist. Run 'nomenclator csv' first",
                    input_dir
                );
            }
            let index = build_from_aemps_csv_dir(&input_dir)?;
            let (target_vmp_code, target_label) = match (cn, vmp) {
                (Some(cn_code), _) => {
                    let hierarchy = index.lookup_by_cn(&cn_code).ok_or_else(|| {
                        anyhow::anyhow!("Presentation with CN '{}' not found in index", cn_code)
                    })?;
                    let vmp_c = hierarchy.vmp_code.ok_or_else(|| {
                        anyhow::anyhow!("Presentation CN '{}' has no associated VMP code", cn_code)
                    })?;
                    let label = hierarchy.vmp_name.unwrap_or_else(|| vmp_c.clone());
                    (vmp_c, format!("CN {} (VMP: {})", cn_code, label))
                }
                (_, Some(vmp_code)) => {
                    let label = index
                        .get_vmp(&vmp_code)
                        .map(|v| v.name.clone())
                        .unwrap_or_else(|| vmp_code.clone());
                    (vmp_code, format!("VMP: {}", label))
                }
                (None, None) => anyhow::bail!("Must provide either --cn or --vmp"),
            };

            let equivalents = index.get_ampps_by_vmp(&target_vmp_code);
            println!("=== Bioequivalentes Comerciales / Genéricos ===");
            println!("Para: {}", target_label);
            println!(
                "Total presentaciones encontradas: {} (mostrando hasta {})\n",
                equivalents.len(),
                limit
            );

            for (i, ampp) in equivalents.iter().take(limit).enumerate() {
                print!("{}. CN: {} - {}", i + 1, ampp.cn, ampp.name);
                if let Some(ref lab) = ampp.laboratory {
                    print!(" [{}]", lab);
                }
                if !ampp.is_commercialized {
                    print!(" (No comercializado)");
                }
                println!();
            }
            Ok(())
        }
        TerminologyCommands::Search {
            query,
            level,
            input_dir,
            limit,
        } => {
            if !input_dir.exists() {
                anyhow::bail!(
                    "Input directory {:?} does not exist. Run 'nomenclator csv' first",
                    input_dir
                );
            }
            let index = build_from_aemps_csv_dir(&input_dir)?;
            let level_lower = level.to_lowercase();

            println!("=== Búsqueda en Terminología para '{}' ===", query);

            if level_lower == "all" || level_lower == "vtm" {
                let vtms = index.search_vtm(&query);
                println!(
                    "\n--- VTM / DCSA (Sustancia Activa) [Total: {}] ---",
                    vtms.len()
                );
                for (i, v) in vtms.iter().take(limit).enumerate() {
                    println!("{}. [{}] {}", i + 1, v.code, v.name);
                }
            }

            if level_lower == "all" || level_lower == "vmp" {
                let vmps = index.search_vmp(&query);
                println!(
                    "\n--- VMP / DCP (Principio Activo + Dosis) [Total: {}] ---",
                    vmps.len()
                );
                for (i, v) in vmps.iter().take(limit).enumerate() {
                    println!("{}. [{}] {}", i + 1, v.code, v.name);
                }
            }

            if level_lower == "all" || level_lower == "ampp" {
                let ampps = index.search_ampp(&query);
                println!(
                    "\n--- AMPP / Presentaciones Comerciales [Total: {}] ---",
                    ampps.len()
                );
                for (i, a) in ampps.iter().take(limit).enumerate() {
                    println!("{}. CN: {} - {}", i + 1, a.cn, a.name);
                }
            }

            println!();
            Ok(())
        }
    }
}
