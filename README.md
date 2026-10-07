# cima-rs

Rust library providing access to the [AEMPS CIMA](https://cima.aemps.es/cima/publico/home.html) (Centro de Información Online de Medicamentos de la AEMPS) nomenclator data and REST API.

## Features

- **XML Data Dumps**: Download and parse CIMA nomenclator XML files to CSV
- **SNS Billing Nomenclator**: Download and export official Ministerio de Sanidad *Nomenclátor de Facturación* data to relational database-ready CSVs:
  - **Agrupaciones Homogéneas (AH)**: Official group codes and descriptions
  - **Pricing**: PVP con IVA, Precio de Referencia, Precios Menores (PM), and Precios Más Bajos (PAB)
  - **Funding & Co-payment**: ALTA/BAJA status, co-payment tiers (NORMAL, ESPECIAL, SIN APORTACION)
  - **Parallel Imports (I.P.)**: Identification and cross-referencing of parallel-distributed medicines
- **SNOMED CT España / AEMPS Clinical Terminology**: Full 4-tier hierarchical representation:
  - **VTM** (*Virtual Therapeutic Moiety* / DCSA): Active substance
  - **VMP** (*Virtual Medicinal Product* / DCP): Substance + strength + pharmaceutical form
  - **VMPP** (*Virtual Medicinal Product Pack* / DCPF): VMP + pack size/count
  - **AMPP** (*Actual Medicinal Product Pack* / Presentación comercial): Authorized commercial pack with Código Nacional (CN)
  - Bi-directional $O(1)$ indexing, bioequivalent presentation lookup, and database-ready relational CSV exports
- **REST API Client**: Complete async client for the CIMA REST API
  - Medication information (`medicamentos`)
  - Commercial presentations (`presentaciones`)
  - Supply problems (`psuministro`)
  - Clinical descriptions (VMP/VMPP)
  - Safety notes and informative materials
  - Segmented documents (ficha técnica, prospecto)
  - Master data catalogs
  - Change logs
- **CLI Tool**: `nomenclator` binary for XML/CSV conversion, billing relational export, terminology hierarchy queries, and API queries

## Installation

```bash
cargo install cima-rs
```

## Usage

### CLI Tool: `nomenclator`

The `nomenclator` binary provides four modes of operation:

#### CSV Mode: Download and Convert XML to CSV

```bash
# Download XML files and convert to CSV
nomenclator csv --output-dir ./output --work-dir ./data

# With custom concurrency
nomenclator csv --concurrency 8

# Also download and process billing and terminology relational CSVs in one step
nomenclator csv --output-dir ./output --include-billing --include-terminology
```

This will:

- Download the latest CIMA nomenclator ZIP file
- Extract all XML files
- Parse them in parallel to CSV format
- Generate 20+ CSV files ready for database import

#### Billing Mode: Nomenclátor de Facturación (Homogeneous Groups, Prices & Parallel Imports)

```bash
# Download and export relational CSV files ready for database import
nomenclator billing export --output-dir ./billing_output

# Export with AEMPS prescriptions, official EMA Parallel Distribution Register, and custom importers catalog
nomenclator billing export \
  --input-file ./nomenclator.csv \
  --prescriptions-csv ./output/prescriptions.csv \
  --ema-register ./ema_parallel_distribution.csv \
  --importers-list ./importers.txt \
  --output-dir ./billing_output

# Query a specific Agrupación Homogénea (shows PM, PAB, and presentations)
nomenclator billing group --code 1941

# Filter and list parallel imports with confidence score and source filters
nomenclator billing parallel-imports --only-active --min-confidence 95 --source ema --limit 20
```

This exports 4 clean relational CSV files:
1. `billing_products.csv`: Full normalized product and pricing catalog (includes parallel import detection flags, confidence score, and origin country)
2. `homogeneous_groups.csv`: Agrupaciones Homogéneas with official Precio Menor (PM) and calculated Precio Más Bajos (PAB)
3. `group_presentations.csv`: Relational mapping table linking presentations to groups and identifying `[★ PAB]` items
4. `parallel_imports.csv`: Identified parallel import presentations with confidence scores (90-100%), detection provenance, country of origin, and EMA notification references

##### Multi-Tiered Parallel Import Detection Algorithm

`cima-rs` implements a 3-pillar algorithm combining national and EU regulatory data sources:

1. **Directorio de Empresas Importadoras y Reacondicionadoras (Parallel Importer Catalog)**:
   - Curated catalog of verified pharmaceutical parallel distributors and repackagers operating in Spain and the EU (*Abacus Medicine, Orifarm, EurimPharm, Kohlpharma, Disfarma, Farmalep, Galia Farma, Garanty Farma, Euroceps, Proinpharma, Top Ridge Pharma, etc.*).
   - Legal entity suffix normalization (*S.A., S.L., A/S, GmbH, B.V.*) to match corporate variants.
   - Support for custom external importer lists via `--importers-list <PATH>`.
   - **Confidence: 95%** (`importer_catalog`).

2. **Cruce con CIMA / AEMPS (Official National AIP & Prescriptions)**:
   - Authoritative offline matching against AEMPS `Prescripcion.xml` / `prescriptions.csv` (`<importacion_paralela>1</importacion_paralela>`).
   - Online verification of individual medication dossiers via CIMA REST API (`nomenclator api check-import --cn <CN>`).
   - **Confidence: 100%** (`aemps_official`).

3. **Cruce con el Registro de Distribución Paralela de la EMA (Centrally Authorised Products - IRIS)**:
   - Cross-referencing against the European Medicines Agency (EMA) public register of parallel distribution notifications for destination Spain (*España / ES*).
   - Built-in baseline of high-impact centrally authorized drugs subject to parallel distribution into Spain (*Eliquis, Enbrel, Humira, Keytruda, Ozempic, Xarelto, Prolia, Stelara, Entresto, Revlimid, Januvia, etc.*).
   - Ingests official EMA IRIS CSV exports via `--ema-register <PATH_OR_URL>` or auto-detects `ema_parallel_distribution.csv` in the work directory. Non-destructively merges downloaded records with the built-in baseline so historical notices are preserved even when the EMA registry list changes.
   - Extracts member state of origin (*Germany, France, Italy, Poland, etc.*) and EMA notification identifiers.
   - **Confidence: 100%** (`ema_register`).

4. **Name Syntax Heuristic**:
   - Detects parallel import markers in product names: `(I.P.)`, `(IP)`, `(IMP.PAR.)`, `IMPORTACION PARALELA`.
   - **Confidence: 90%** (`name_syntax`).

#### Terminology Mode: SNOMED CT España / AEMPS Clinical Hierarchy

```bash
# Build terminology index from parsed AEMPS CSVs and export relational CSVs
nomenclator terminology export --input-dir ./output --output-dir ./terminology_output

# Query the full clinical hierarchy for a Código Nacional (CN)
nomenclator terminology hierarchy --cn 650123

# Find bioequivalent commercial/generic presentations sharing the same VMP
nomenclator terminology equivalents --cn 650123

# Search concepts across VTM, VMP, or AMPP
nomenclator terminology search --query "Paracetamol"
```

This exports 5 relational CSV files ready for database ingestion:
1. `terminology_vtm.csv`: Virtual Therapeutic Moieties (`code`, `name`)
2. `terminology_vmp.csv`: Virtual Medicinal Products (`code`, `name`, `vtm_code`)
3. `terminology_vmpp.csv`: Virtual Medicinal Product Packs (`code`, `name`, `vmp_code`)
4. `terminology_ampp.csv`: Actual Medicinal Product Packs (`code`, `cn`, `name`, `vmpp_code`, `laboratory`, `is_commercialized`)
5. `terminology_hierarchy.csv`: Denormalized full hierarchy row per commercial presentation

#### API Mode: Query REST API

```bash
# Get specific medication by registration number
nomenclator api medicamento --nregistro 51347 --presentaciones --activos

# Search medications
nomenclator api search-medicamentos --nombre "Paracetamol" --limit 20
nomenclator api search-medicamentos --laboratorio "Pfizer" --comercializados

# Get presentation details
nomenclator api presentacion --cn 12345678

# Check if a medication in CIMA is an authorized parallel import (AIP)
nomenclator api check-import --cn 650999
nomenclator api check-import --nregistro 82941

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

```rust,no_run
use cima_rs::{CimaClient, SearchMedicationsParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = CimaClient::new()?;
    
    // Get specific medication by registration number
    let med = client.get_medication(Some("51347"), None).await?;
    println!("Medication: {}", med.name);
    
    // Search medications
    let params = SearchMedicationsParams {
        name: Some("Paracetamol".to_string()),
        ..Default::default()
    };
    let results = client.search_medications(&params).await?;
    
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

#### Billing Nomenclator & Homogeneous Groups

```rust,no_run
use cima_rs::billing::{parse_billing_csv, compute_homogeneous_groups, export_billing_data_to_csvs};
use anyhow::Result;

fn main() -> Result<()> {
    let products = parse_billing_csv("billing_data/nomenclator_facturacion.csv")?;
    let groups = compute_homogeneous_groups(&products);
    println!("Processed {} homogeneous groups", groups.len());

    let summary = export_billing_data_to_csvs(&products, "billing_csv_output")?;
    println!("Exported {} products to database CSVs", summary.total_products);
    Ok(())
}
```

#### SNOMED CT España / AEMPS Clinical Terminology

```rust,no_run
use cima_rs::terminology::{build_from_aemps_csv_dir, export_terminology_to_csvs};
use anyhow::Result;

fn main() -> Result<()> {
    // Build O(1) bi-directional hierarchy index from parsed AEMPS CSVs
    let index = build_from_aemps_csv_dir("csv_output")?;

    // Traverse upwards: CN -> VMPP -> VMP -> VTM
    if let Some(hierarchy) = index.lookup_by_cn("650123") {
        println!("Presentation: {}", hierarchy.ampp_name);
        println!("VMP: {:?}", hierarchy.vmp_name);
        println!("VTM: {:?}", hierarchy.vtm_name);
    }

    // Traverse downwards: find all bioequivalent commercial presentations for a VMP
    let equivalents = index.get_equivalents_by_cn("650123");
    for eq in equivalents {
        println!("Bioequivalent: CN {} - {}", eq.cn, eq.name);
    }

    // Export relational CSVs ready for database ingestion
    export_terminology_to_csvs(&index, "terminology_csv_output")?;
    Ok(())
}
```

## API Endpoints

All endpoints return structured Rust types with serde serialization support:

- `get_medication()` - Get medication details
- `search_medications()` - Search medications with filters
- `search_in_technical_sheet()` - Search in technical sheets
- `get_presentation()` - Get presentation details
- `search_presentations()` - Search presentations
- `get_all_supply_problems()` - Get all supply problems
- `get_supply_problems()` - Get supply problems by CN
- `search_clinical_descriptions()` - Search clinical descriptions
- `get_safety_notes()` - Get safety notes
- `get_informative_materials()` - Get informative materials
- `get_document_sections()` - Get document sections
- `get_document_content()` - Get document content
- `get_master_data()` - Get master data catalogs
- `get_change_log()` - Get change logs

## Requirements

- Rust 1.91+
- Tokio async runtime

## License

See LICENSE file.
