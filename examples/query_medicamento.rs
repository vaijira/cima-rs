use anyhow::Result;
use cima_rs::{CimaClient, SearchMedicamentosParams};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "query_medicamento")]
#[command(about = "Query CIMA API for medication information", long_about = None)]
struct Args {
    /// Número de registro del medicamento
    #[arg(short, long)]
    nregistro: Option<String>,

    /// Código nacional de la presentación
    #[arg(short, long)]
    cn: Option<String>,

    /// Nombre del medicamento para búsqueda
    #[arg(long)]
    nombre: Option<String>,

    /// Nombre del laboratorio para búsqueda
    #[arg(long)]
    laboratorio: Option<String>,

    /// Mostrar presentaciones del medicamento
    #[arg(short, long)]
    presentaciones: bool,

    /// Mostrar principios activos
    #[arg(short = 'a', long)]
    activos: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client = CimaClient::new()?;

    // Si se proporciona nregistro o cn, obtener medicamento específico
    if args.nregistro.is_some() || args.cn.is_some() {
        let medicamento = client
            .get_medicamento(args.nregistro.as_deref(), args.cn.as_deref())
            .await?;

        println!("=== Medicamento ===");
        println!("Nº Registro: {}", medicamento.nregistro);
        println!("Nombre: {}", medicamento.nombre);
        println!("Laboratorio: {}", medicamento.labtitular);
        println!("Principios Activos: {}", medicamento.pactivos);
        println!("Condiciones de prescripción: {}", medicamento.cpresc);

        if let Some(comerc) = medicamento.comerc {
            println!("Comercializado: {}", if comerc { "Sí" } else { "No" });
        }

        if let Some(triangulo) = medicamento.triangulo {
            if triangulo {
                println!("⚠️  Triángulo negro (medicamento bajo vigilancia adicional)");
            }
        }

        if let Some(huerfano) = medicamento.huerfano {
            if huerfano {
                println!("💊 Medicamento huérfano");
            }
        }

        if args.activos && !medicamento.principios_activos.is_empty() {
            println!("\n=== Principios Activos ===");
            for pa in &medicamento.principios_activos {
                print!("- {}", pa.nombre);
                if let (Some(cantidad), Some(unidad)) = (&pa.cantidad, &pa.unidad) {
                    print!(": {} {}", cantidad, unidad);
                }
                println!();
            }
        }

        if args.presentaciones && !medicamento.presentaciones.is_empty() {
            println!("\n=== Presentaciones ===");
            for pres in &medicamento.presentaciones {
                println!("- CN: {} - {}", pres.cn, pres.nombre);
                if pres.comerc {
                    println!("  ✓ Comercializada");
                }
            }
        }

        if !medicamento.docs.is_empty() {
            println!("\n=== Documentos Disponibles ===");
            for doc in &medicamento.docs {
                let tipo = match doc.tipo {
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
    // Si se proporciona nombre o laboratorio, buscar medicamentos
    else if args.nombre.is_some() || args.laboratorio.is_some() {
        println!("Buscando medicamentos...\n");

        let params = SearchMedicamentosParams {
            nombre: args.nombre.clone(),
            laboratorio: args.laboratorio.clone(),
            ..Default::default()
        };

        let response = client.search_medicamentos(&params).await?;

        println!(
            "Encontrados {} medicamentos (mostrando primeros 10):\n",
            response.total_rows
        );

        for (i, med) in response.results.iter().enumerate().take(10) {
            println!("{}. {} ({})", i + 1, med.nombre, med.nregistro);
            println!("   Laboratorio: {}", med.labtitular);
            if let Some(comerc) = med.comerc {
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
