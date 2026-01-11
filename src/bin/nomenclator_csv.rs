use clap::Parser;
use cima_rs::downloader::download_and_extract_nomenclator;
use cima_rs::parser::{
    parse_atc_xml_to_csv, parse_dcp_xml_to_csv, parse_dcpf_xml_to_csv, parse_dcsa_xml_to_csv,
    parse_envases_xml_to_csv, parse_excipientes_xml_to_csv,
    parse_forma_farmaceutica_simplificada_xml_to_csv, parse_forma_farmaceutica_xml_to_csv,
    parse_laboratorio_xml_to_csv, parse_prescription_xml_to_csv, parse_principio_activo_xml_to_csv,
    parse_situacion_registro_xml_to_csv, parse_unidad_contenido_xml_to_csv,
    parse_via_administracion_xml_to_csv,
};
use std::path::PathBuf;
use std::fs;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A tool to download and convert AEMPS Nomenclator XML files to CSV.",
    long_about = "This tool automates the process of downloading the latest prescription data from AEMPS, \
                  extracting the XML files, and parsing them into specialized CSV files suitable for \
                  PostgreSQL import."
)]
struct Args {
    /// Directory where the generated CSV files will be stored.
    #[arg(short, long, default_value = "csv_output", help = "Output directory for CSV files")]
    output_dir: PathBuf,

    /// Directory where the downloaded XML files will be extracted and stored.
    #[arg(short, long, default_value = "nomenclator_data", help = "Working directory for XML files")]
    work_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Ensure directories exist
    fs::create_dir_all(&args.output_dir)?;
    fs::create_dir_all(&args.work_dir)?;

    println!("Target work directory: {:?}", args.work_dir);
    println!("Target output directory: {:?}", args.output_dir);

    // 1. Download and extract
    println!("Downloading and extracting AEMPS Nomenclator data...");
    download_and_extract_nomenclator(&args.work_dir).await?;

    // 2. Define files to parse
    let mapping = vec![
        ("DICCIONARIO_ATC.xml", "atc.csv", parse_atc_xml_to_csv as fn(PathBuf, PathBuf) -> anyhow::Result<()>),
        ("DICCIONARIO_DCP.xml", "dcp.csv", parse_dcp_xml_to_csv as fn(PathBuf, PathBuf) -> anyhow::Result<()>),
        ("DICCIONARIO_DCPF.xml", "dcpf.csv", parse_dcpf_xml_to_csv),
        ("DICCIONARIO_DCSA.xml", "dcsa.csv", parse_dcsa_xml_to_csv),
        ("DICCIONARIO_ENVASES.xml", "envases.csv", parse_envases_xml_to_csv),
        ("DICCIONARIO_EXCIPIENTES_DECL_OBLIGATORIA.xml", "excipientes.csv", parse_excipientes_xml_to_csv),
        ("DICCIONARIO_FORMA_FARMACEUTICA.xml", "forma_farmaceutica.csv", parse_forma_farmaceutica_xml_to_csv),
        ("DICCIONARIO_FORMA_FARMACEUTICA_SIMPLIFICADAS.xml", "forma_farmaceutica_simplificada.csv", parse_forma_farmaceutica_simplificada_xml_to_csv),
        ("DICCIONARIO_LABORATORIOS.xml", "laboratorios.csv", parse_laboratorio_xml_to_csv),
        ("DICCIONARIO_PRINCIPIOS_ACTIVOS.xml", "principios_activos.csv", parse_principio_activo_xml_to_csv),
        ("DICCIONARIO_SITUACION_REGISTRO.xml", "situacion_registro.csv", parse_situacion_registro_xml_to_csv),
        ("DICCIONARIO_UNIDAD_CONTENIDO.xml", "unidad_contenido.csv", parse_unidad_contenido_xml_to_csv),
        ("DICCIONARIO_VIAS_ADMINISTRACION.xml", "vias_administracion.csv", parse_via_administracion_xml_to_csv),
        ("Prescripcion.xml", "prescripciones.csv", parse_prescription_xml_to_csv),
    ];

    // 3. Process each file
    for (xml_name, csv_name, parser_fn) in mapping {
        let xml_path = args.work_dir.join(xml_name);
        let csv_path = args.output_dir.join(csv_name);

        if xml_path.exists() {
            println!("Parsing {} to {}...", xml_name, csv_name);
            parser_fn(xml_path, csv_path)?;
        } else {
            println!("Warning: File not found, skipping: {:?}", xml_path);
        }
    }

    println!("Done! All CSV files are in {:?}", args.output_dir);

    Ok(())
}
