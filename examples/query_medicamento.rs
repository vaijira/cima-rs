use anyhow::Result;
use cima_rs::{CimaClient, SearchMedicationsParams};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "query_medicamento")]
#[command(about = "Query CIMA API for medication information", long_about = None)]
struct Args {
    /// Medication registration number
    #[arg(short, long)]
    nregistro: Option<String>,

    /// National code of the presentation
    #[arg(short, long)]
    cn: Option<String>,

    /// Medication name for search
    #[arg(long)]
    nombre: Option<String>,

    /// Laboratory name for search
    #[arg(long)]
    laboratorio: Option<String>,

    /// Show medication presentations
    #[arg(short, long)]
    presentaciones: bool,

    /// Show active ingredients
    #[arg(short = 'a', long)]
    activos: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client = CimaClient::new()?;

    // If registration number or national code is provided, get specific medication
    if args.nregistro.is_some() || args.cn.is_some() {
        let medicamento = client
            .get_medication(args.nregistro.as_deref(), args.cn.as_deref())
            .await?;

        println!("=== Medicamento ===");
        println!("Nº Registro: {}", medicamento.nregistro);
        println!("Nombre: {}", medicamento.name);
        println!("Laboratorio: {}", medicamento.labtitular);
        println!("Principios Activos: {}", medicamento.pactivos);
        println!("Condiciones de prescripción: {}", medicamento.cpresc);

        if let Some(comerc) = medicamento.commercialized {
            println!("Comercializado: {}", if comerc { "Sí" } else { "No" });
        }

        if let Some(true) = medicamento.black_triangle {
            println!("⚠️  Triángulo negro (medicamento bajo vigilancia adicional)");
        }

        if let Some(true) = medicamento.orphan {
            println!("💊 Medicamento huérfano");
        }

        if args.activos && !medicamento.active_ingredients.is_empty() {
            println!("\n=== Principios Activos ===");
            for pa in &medicamento.active_ingredients {
                print!("- {}", pa.name);
                if let (Some(cantidad), Some(unidad)) = (&pa.amount, &pa.unit) {
                    print!(": {} {}", cantidad, unidad);
                }
                println!();
            }
        }

        if args.presentaciones && !medicamento.presentations.is_empty() {
            println!("\n=== Presentaciones ===");
            for pres in &medicamento.presentations {
                println!("- CN: {} - {}", pres.cn, pres.name);
                if pres.commercialized {
                    println!("  ✓ Comercializada");
                }
            }
        }

        if !medicamento.docs.is_empty() {
            println!("\n=== Documentos Disponibles ===");
            for doc in &medicamento.docs {
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
    // If name or laboratory is provided, search medications
    else if args.nombre.is_some() || args.laboratorio.is_some() {
        println!("Buscando medicamentos...\n");

        let params = SearchMedicationsParams {
            name: args.nombre.clone(),
            laboratory: args.laboratorio.clone(),
            ..Default::default()
        };

        let response = client.search_medications(&params).await?;

        println!(
            "Encontrados {} medicamentos (mostrando primeros 10):\n",
            response.total_rows
        );

        for (i, med) in response.results.iter().enumerate().take(10) {
            println!("{}. {} ({})", i + 1, med.name, med.nregistro);
            println!("   Laboratorio: {}", med.labtitular);
            if let Some(comerc) = med.commercialized {
                println!("   Comercializado: {}", if comerc { "Sí" } else { "No" });
            }
            println!();
        }

        if response.total_rows > 10 {
            println!(
                "... y {} más (total: {})",
                response.total_rows - response.results.len().min(10) as u32,
                response.total_rows
            );
        }
    } else {
        eprintln!("Error: Debe proporcionar --nregistro, --cn, --nombre o --laboratorio");
        eprintln!("Ejemplo: cargo run --example query_medicamento -- --nregistro 51347");
        std::process::exit(1);
    }

    Ok(())
}
