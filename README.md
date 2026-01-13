# cima-rs

Rust library providing access to the [AEMPS CIMA](https://cima.aemps.es/cima/publico/home.html) (Centro de Información Online de Medicamentos de la AEMPS) nomenclator data and REST API.

## Features

- **XML Data Dumps**: Download and parse CIMA nomenclator XML files to CSV
- **REST API Client**: Complete async client for the CIMA REST API
  - Medication information (`medicamentos`)
  - Commercial presentations (`presentaciones`)
  - Supply problems (`psuministro`)
  - Clinical descriptions (VMP/VMPP)
  - Safety notes and informative materials
  - Segmented documents (ficha técnica, prospecto)
  - Master data catalogs
  - Change logs
- **CLI Tool**: `nomenclator` binary for both CSV conversion and API queries

## Installation

```bash
cargo install --path .
```

## Usage

### CLI Tool: `nomenclator`

The `nomenclator` binary provides two modes of operation:

#### CSV Mode: Download and Convert XML to CSV

```bash
# Download XML files and convert to CSV
nomenclator csv --output-dir ./output --work-dir ./data

# With custom concurrency
nomenclator csv --concurrency 8
```

This will:

- Download the latest CIMA nomenclator ZIP file
- Extract all XML files
- Parse them in parallel to CSV format
- Generate 20+ CSV files ready for database import

#### API Mode: Query REST API

```bash
# Get specific medication by registration number
nomenclator api medicamento --nregistro 51347 --presentaciones --activos

# Search medications
nomenclator api search-medicamentos --nombre "Paracetamol" --limit 20
nomenclator api search-medicamentos --laboratorio "Pfizer" --comercializados

# Get presentation details
nomenclator api presentacion --cn 12345678

# Get supply problems
nomenclator api supply-problems
nomenclator api supply-problems --cn 12345678

# Get safety notes
nomenclator api safety-notes --nregistro 51347

# Get changes since a date
nomenclator api changes --desde "01/01/2024"

# Query master data
nomenclator api maestra --tipo pa --nombre "Paracetamol"
nomenclator api maestra --tipo lab --limit 50
```

Available master data types (`--tipo`):

- `pa` - Principios activos (active ingredients)
- `ff` - Formas farmacéuticas (pharmaceutical forms)
- `va` - Vías de administración (administration routes)
- `lab` - Laboratorios (laboratories)
- `atc` - Códigos ATC (ATC codes)

### Rust Library API

```rust
use cima_rs::{CimaClient, SearchMedicamentosParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = CimaClient::new()?;
    
    // Get specific medication by registration number
    let med = client.get_medicamento(Some("51347"), None).await?;
    println!("Medication: {}", med.nombre);
    
    // Search medications
    let params = SearchMedicamentosParams {
        nombre: Some("Paracetamol".to_string()),
        ..Default::default()
    };
    let results = client.search_medicamentos(&params).await?;
    
    Ok(())
}
```

See `examples/query_medicamento.rs` for a complete example.

#### Multi-CSV Parser (Recommended)

```rust,no_run
use cima_rs::parser::parse_prescription_xml_to_csvs;
use anyhow::Result;

fn main() -> Result<()> {
    parse_prescription_xml_to_csvs("input.xml", "output_dir")?;
    Ok(())
}
```

This generates multiple normalized CSV files:

## API Endpoints

All endpoints return structured Rust types with serde serialization support:

- `get_medicamento()` - Get medication details
- `search_medicamentos()` - Search medications with filters
- `buscar_en_ficha_tecnica()` - Search in technical sheets
- `get_presentacion()` - Get presentation details
- `search_presentaciones()` - Search presentations
- `get_problemas_suministro_all()` - Get all supply problems
- `get_problemas_suministro()` - Get supply problems by CN
- `search_vmpp()` - Search clinical descriptions
- `get_notas_seguridad()` - Get safety notes
- `get_materiales_informativos()` - Get informative materials
- `get_doc_secciones()` - Get document sections
- `get_doc_contenido()` - Get document content
- `get_maestra()` - Get master data catalogs
- `get_registro_cambios()` - Get change logs

## Requirements

- Rust 1.91+
- Tokio async runtime

## License

See LICENSE file.
