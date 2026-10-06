use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

/// Clean, relational-ready representation of a product in the Nomenclátor de Facturación.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BillingProduct {
    /// 6-digit National Code (Código Nacional)
    pub cn: String,
    /// Product name (Nombre del producto farmacéutico)
    pub name: String,
    /// Drug type classification (e.g. "Medicamento Generico", "Medicamento Etica")
    pub drug_type: Option<String>,
    /// Generic name for healthcare accessories/effects
    pub generic_accessory_name: Option<String>,
    /// Supplier laboratory code
    pub supplier_lab_code: Option<String>,
    /// Supplier laboratory name
    pub supplier_lab_name: Option<String>,
    /// Status in the nomenclator (e.g. "ALTA", "BAJA GENERAL", "BAJA POR NO COMERCIALIZAR")
    pub status: String,
    /// Convenience flag: true if status is "ALTA"
    pub is_active: bool,
    /// Registration date in format DD/MM/YYYY
    pub registration_date: Option<String>,
    /// Cancellation/revocation date in format DD/MM/YYYY
    pub cancellation_date: Option<String>,
    /// Beneficiary co-payment classification (e.g. "NORMAL", "ESPECIAL", "SIN APORTACION")
    pub beneficiary_copay: Option<String>,
    /// Active ingredient or active ingredients association
    pub active_ingredient: Option<String>,
    /// Retail price with VAT (PVP con IVA)
    pub pvp_iva: Option<f64>,
    /// Reference price (Precio de referencia)
    pub reference_price: Option<f64>,
    /// Lowest price of homogeneous group (Precio Menor de la agrupación)
    pub group_lowest_price: Option<f64>,
    /// Homogeneous group code (Código de la agrupación homogénea)
    pub homogeneous_group_code: Option<String>,
    /// Homogeneous group description (Nombre de la agrupación homogénea)
    pub homogeneous_group_name: Option<String>,
    /// Hospital diagnosis requirement
    pub hospital_diagnosis: bool,
    /// Long term treatment indicator
    pub long_term_treatment: bool,
    /// Special medical control indicator
    pub special_medical_control: bool,
    /// Orphan drug indicator
    pub orphan_drug: bool,
    /// Parallel import flag (from AEMPS prescription data or detection heuristics)
    pub is_parallel_import: bool,
}

/// Official Agrupación Homogénea (AH) grouping interchangeable presentations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HomogeneousGroup {
    /// Homogeneous group code (Código de la agrupación homogénea)
    pub code: String,
    /// Homogeneous group name / description
    pub name: String,
    /// Official Precio Menor (PM) from the Ministry catalog
    pub precio_menor: Option<f64>,
    /// Calculated Precio Más Bajo (PAB): lowest active PVP con IVA in the group
    pub precio_mas_bajo: Option<f64>,
    /// Total number of presentations in this group
    pub total_presentations: usize,
    /// Number of active presentations (status == "ALTA")
    pub active_presentations: usize,
    /// Whether any presentation in this group is an identified parallel import
    pub has_parallel_imports: bool,
}

/// Relational junction table mapping presentations to their homogeneous groups.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupPresentationMapping {
    /// Homogeneous group code (Foreign Key -> HomogeneousGroup.code)
    pub group_code: String,
    /// Homogeneous group name
    pub group_name: String,
    /// National Code (Foreign Key -> BillingProduct.cn)
    pub cn: String,
    /// Product name
    pub product_name: String,
    /// Retail price with VAT (PVP con IVA)
    pub pvp_iva: Option<f64>,
    /// Whether this presentation matches the lowest price (PAB) of the group
    pub is_precio_mas_bajo: bool,
    /// Status in the nomenclator
    pub status: String,
    /// True if active (ALTA)
    pub is_active: bool,
    /// True if parallel import
    pub is_parallel_import: bool,
}

/// Detailed parallel import presentation record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParallelImportRecord {
    /// National Code (Código Nacional)
    pub cn: String,
    /// Product name
    pub name: String,
    /// Supplier laboratory name
    pub supplier_lab_name: Option<String>,
    /// Status in the nomenclator
    pub status: String,
    /// True if active (ALTA)
    pub is_active: bool,
    /// Retail price with VAT (PVP con IVA)
    pub pvp_iva: Option<f64>,
    /// Reference price
    pub reference_price: Option<f64>,
    /// Homogeneous group code (if assigned)
    pub homogeneous_group_code: Option<String>,
    /// Homogeneous group name (if assigned)
    pub homogeneous_group_name: Option<String>,
    /// Active ingredient
    pub active_ingredient: Option<String>,
    /// Detection source ("aemps_prescription", "name_pattern", "supplier_lab")
    pub detection_source: String,
}

/// Summary of generated CSV files from billing export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingExportSummary {
    pub total_products: usize,
    pub active_products: usize,
    pub total_groups: usize,
    pub total_parallel_imports: usize,
    pub total_mappings: usize,
    pub products_csv: PathBuf,
    pub groups_csv: PathBuf,
    pub group_presentations_csv: PathBuf,
    pub parallel_imports_csv: PathBuf,
}

/// Parses decimal price strings supporting dots, commas, and leading dots (e.g., "2.01", "2,01", ".99").
fn parse_price(val: &str) -> Option<f64> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return None;
    }
    let sanitized = trimmed.replace(',', ".");
    sanitized.parse::<f64>().ok()
}

/// Normalizes boolean fields from Spanish "SI"/"NO"/empty strings.
fn parse_bool(val: &str) -> bool {
    let trimmed = val.trim().to_uppercase();
    trimmed == "SI" || trimmed == "S" || trimmed == "1" || trimmed == "TRUE"
}

/// Trims and converts empty strings to None.
fn clean_opt(val: &str) -> Option<String> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Normalizes Código Nacional to 6 digits with leading zeros if numeric.
fn normalize_cn(val: &str) -> String {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) && trimmed.len() < 6 {
        format!("{:0>6}", trimmed)
    } else {
        trimmed.to_string()
    }
}

/// Checks heuristic patterns indicating parallel import in product name or lab.
fn detect_parallel_import_heuristic(name: &str, lab: Option<&str>) -> bool {
    let upper_name = name.to_uppercase();
    if upper_name.contains("(I.P.)")
        || upper_name.contains("(IP)")
        || upper_name.contains("(IMP.PAR.)")
        || upper_name.contains("IMPORTACION PARALELA")
        || upper_name.contains("IMPORTACIÓN PARALELA")
        || upper_name.ends_with(" I.P.")
        || upper_name.ends_with(" IP")
    {
        return true;
    }

    if let Some(lab_name) = lab {
        let upper_lab = lab_name.to_uppercase();
        if upper_lab.contains("PARALEL")
            || upper_lab.contains("ABACUS MEDICINE")
            || upper_lab.contains("EURIMPHARM")
            || upper_lab.contains("KOHLPHARMA")
            || upper_lab.contains("MEDIFARM")
        {
            return true;
        }
    }

    false
}

/// Reads a CSV stream of the Nomenclátor de Facturación and returns normalized `BillingProduct` items.
pub fn parse_billing_csv_reader<R: Read>(reader: R) -> Result<Vec<BillingProduct>> {
    let mut buf_reader = BufReader::new(reader);

    // Consume optional UTF-8 BOM
    let mut first_bytes = [0u8; 3];
    let n = buf_reader.read(&mut first_bytes).unwrap_or(0);
    let chained_reader: Box<dyn Read> = if n >= 3 && first_bytes == [0xEF, 0xBB, 0xBF] {
        Box::new(buf_reader)
    } else {
        Box::new(std::io::Cursor::new(first_bytes[..n].to_vec()).chain(buf_reader))
    };

    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(chained_reader);

    let raw_headers = csv_reader.headers()?.clone();
    let headers: Vec<String> = raw_headers
        .iter()
        .map(|h| h.trim().trim_start_matches('\u{feff}').to_string())
        .collect();

    // Map column names to indices
    let mut col_map: HashMap<String, usize> = HashMap::new();
    for (idx, header) in headers.iter().enumerate() {
        let normalized = header.to_lowercase();
        col_map.insert(normalized, idx);
    }

    let get_idx = |names: &[&str]| -> Option<usize> {
        for name in names {
            if let Some(&idx) = col_map.get(&name.to_lowercase()) {
                return Some(idx);
            }
        }
        None
    };

    let idx_cn = get_idx(&["código nacional", "codigo nacional", "cn"])
        .context("Missing 'Código Nacional' column")?;
    let idx_name = get_idx(&[
        "nombre del producto farmacéutico",
        "nombre del producto farmaceutico",
        "nombre",
    ])
    .context("Missing product name column")?;
    let idx_drug_type = get_idx(&["tipo de fármaco", "tipo de farmaco"]);
    let idx_gen_acc = get_idx(&[
        "nombre genérico efecto y accesorio",
        "nombre generico efecto y accesorio",
    ]);
    let idx_lab_code = get_idx(&[
        "código del laboratorio ofertante",
        "codigo del laboratorio ofertante",
    ]);
    let idx_lab_name = get_idx(&["nombre del laboratorio ofertante"]);
    let idx_status = get_idx(&["estado"]);
    let idx_reg_date = get_idx(&[
        "fecha de alta en el nomenclátor",
        "fecha de alta en el nomenclator",
    ]);
    let idx_canc_date = get_idx(&[
        "fecha de baja en el nomenclátor",
        "fecha de baja en el nomenclator",
    ]);
    let idx_copay = get_idx(&["aportación del beneficiario", "aportacion del beneficiario"]);
    let idx_active_ing = get_idx(&[
        "principio activo o asociación de principios activos",
        "principio activo o asociacion de principios activos",
    ]);
    let idx_pvp = get_idx(&[
        "precio venta al público con iva",
        "precio de venta al público con iva",
        "precio venta al publico con iva",
        "pvp",
    ]);
    let idx_ref_price = get_idx(&["precio de referencia", "precio referencia"]);
    let idx_lowest_price = get_idx(&[
        "menor precio de la agrupación homogénea del producto sanitario",
        "menor precio de la agrupacion homogenea del producto sanitario",
        "menor precio de la agrupación homogéna del producto sanitario",
        "menor precio de la agrupación homogénea",
        "menor precio agrupacion",
    ]);
    let idx_group_code = get_idx(&[
        "código de la agrupación homogénea del producto sanitario",
        "codigo de la agrupacion homogenea del producto sanitario",
        "código de la agrupación homogénea",
        "codigo agrupacion homogenea",
    ]);
    let idx_group_name = get_idx(&[
        "nombre de la agrupación homogénea del producto sanitario",
        "nombre de la agrupacion homogenea del producto sanitario",
        "nombre de la agrupación homogénea",
        "nombre agrupacion homogenea",
    ]);
    let idx_hosp_diag = get_idx(&["diagnóstico hospitalario", "diagnostico hospitalario"]);
    let idx_long_term = get_idx(&[
        "tratamiento de larga duración",
        "tratamiento de larga duracion",
    ]);
    let idx_spec_med = get_idx(&["especial control médico", "especial control medico"]);
    let idx_orphan = get_idx(&["medicamento huérfano", "medicamento huerfano"]);

    let mut products = Vec::new();
    let mut record = csv::StringRecord::new();

    while csv_reader.read_record(&mut record)? {
        let get_val = |opt_idx: Option<usize>| -> &str {
            opt_idx.and_then(|i| record.get(i)).unwrap_or("").trim()
        };

        let raw_cn = get_val(Some(idx_cn));
        if raw_cn.is_empty() {
            continue;
        }

        let cn = normalize_cn(raw_cn);
        let name = get_val(Some(idx_name)).to_string();
        let drug_type = clean_opt(get_val(idx_drug_type));
        let generic_accessory_name = clean_opt(get_val(idx_gen_acc));
        let supplier_lab_code = clean_opt(get_val(idx_lab_code));
        let supplier_lab_name = clean_opt(get_val(idx_lab_name));
        let status = clean_opt(get_val(idx_status)).unwrap_or_else(|| "DESCONOCIDO".to_string());
        let is_active = status.trim().eq_ignore_ascii_case("ALTA");
        let registration_date = clean_opt(get_val(idx_reg_date));
        let cancellation_date = clean_opt(get_val(idx_canc_date));
        let beneficiary_copay = clean_opt(get_val(idx_copay));
        let active_ingredient = clean_opt(get_val(idx_active_ing));
        let pvp_iva = parse_price(get_val(idx_pvp));
        let reference_price = parse_price(get_val(idx_ref_price));
        let group_lowest_price = parse_price(get_val(idx_lowest_price));
        let homogeneous_group_code = clean_opt(get_val(idx_group_code));
        let homogeneous_group_name = clean_opt(get_val(idx_group_name));
        let hospital_diagnosis = parse_bool(get_val(idx_hosp_diag));
        let long_term_treatment = parse_bool(get_val(idx_long_term));
        let special_medical_control = parse_bool(get_val(idx_spec_med));
        let orphan_drug = parse_bool(get_val(idx_orphan));

        let is_parallel_import =
            detect_parallel_import_heuristic(&name, supplier_lab_name.as_deref());

        products.push(BillingProduct {
            cn,
            name,
            drug_type,
            generic_accessory_name,
            supplier_lab_code,
            supplier_lab_name,
            status,
            is_active,
            registration_date,
            cancellation_date,
            beneficiary_copay,
            active_ingredient,
            pvp_iva,
            reference_price,
            group_lowest_price,
            homogeneous_group_code,
            homogeneous_group_name,
            hospital_diagnosis,
            long_term_treatment,
            special_medical_control,
            orphan_drug,
            is_parallel_import,
        });
    }

    Ok(products)
}

/// Parses the Nomenclátor de Facturación CSV file at the specified path.
pub fn parse_billing_csv<P: AsRef<Path>>(path: P) -> Result<Vec<BillingProduct>> {
    let file = File::open(path.as_ref())
        .with_context(|| format!("Failed to open billing CSV file at {:?}", path.as_ref()))?;
    parse_billing_csv_reader(file)
}

/// Loads a set of parallel import National Codes (CNs) from an AEMPS `prescriptions.csv` file.
pub fn load_parallel_import_cns_from_prescriptions_csv<P: AsRef<Path>>(
    path: P,
) -> Result<HashSet<String>> {
    let file = File::open(path.as_ref())
        .with_context(|| format!("Failed to open prescriptions CSV at {:?}", path.as_ref()))?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let headers = reader.headers()?.clone();
    let cn_idx = headers
        .iter()
        .position(|h| h.trim().eq_ignore_ascii_case("cod_nacion"))
        .context("Missing 'cod_nacion' column in prescriptions.csv")?;
    let pi_idx = headers
        .iter()
        .position(|h| h.trim().eq_ignore_ascii_case("importacion_paralela"))
        .context("Missing 'importacion_paralela' column in prescriptions.csv")?;

    let mut set = HashSet::new();
    let mut record = csv::StringRecord::new();
    while reader.read_record(&mut record)? {
        let cn = record.get(cn_idx).unwrap_or("").trim();
        let pi = record.get(pi_idx).unwrap_or("").trim();
        if !cn.is_empty() && (pi == "1" || pi.eq_ignore_ascii_case("true")) {
            set.insert(normalize_cn(cn));
        }
    }

    Ok(set)
}

/// Tags parallel imports in the billing products list using a set of known parallel import CNs.
pub fn tag_parallel_imports_from_prescriptions(
    products: &mut [BillingProduct],
    parallel_import_cns: &HashSet<String>,
) {
    for product in products.iter_mut() {
        if parallel_import_cns.contains(&product.cn) {
            product.is_parallel_import = true;
        }
    }
}

/// Computes the unique Homogeneous Groups (AH) with calculated Precios Más Bajos (PAB)
/// and assigned Precios Menores (PM).
pub fn compute_homogeneous_groups(products: &[BillingProduct]) -> Vec<HomogeneousGroup> {
    struct GroupAcc {
        name: String,
        precio_menor: Option<f64>,
        total: usize,
        active: usize,
        active_prices: Vec<f64>,
        all_prices: Vec<f64>,
        has_pi: bool,
    }

    let mut map: HashMap<String, GroupAcc> = HashMap::new();

    for p in products {
        if let Some(code) = &p.homogeneous_group_code {
            let code_str = code.trim();
            if code_str.is_empty() {
                continue;
            }

            let entry = map.entry(code_str.to_string()).or_insert_with(|| GroupAcc {
                name: p.homogeneous_group_name.clone().unwrap_or_default(),
                precio_menor: p.group_lowest_price,
                total: 0,
                active: 0,
                active_prices: Vec::new(),
                all_prices: Vec::new(),
                has_pi: false,
            });

            if entry.name.is_empty() && p.homogeneous_group_name.is_some() {
                entry.name = p.homogeneous_group_name.clone().unwrap();
            }

            if entry.precio_menor.is_none() && p.group_lowest_price.is_some() {
                entry.precio_menor = p.group_lowest_price;
            }

            entry.total += 1;
            if p.is_active {
                entry.active += 1;
                if let Some(price) = p.pvp_iva {
                    entry.active_prices.push(price);
                }
            }

            if let Some(price) = p.pvp_iva {
                entry.all_prices.push(price);
            }

            if p.is_parallel_import {
                entry.has_pi = true;
            }
        }
    }

    let mut groups: Vec<HomogeneousGroup> = map
        .into_iter()
        .map(|(code, acc)| {
            // PAB (Precio Más Bajo): lowest PVP among active presentations; fallback to lowest of all
            let p_min_active = acc
                .active_prices
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);
            let precio_mas_bajo = if p_min_active.is_finite() {
                Some(p_min_active)
            } else {
                let p_min_all = acc.all_prices.iter().copied().fold(f64::INFINITY, f64::min);
                if p_min_all.is_finite() {
                    Some(p_min_all)
                } else {
                    None
                }
            };

            HomogeneousGroup {
                code,
                name: acc.name,
                precio_menor: acc.precio_menor,
                precio_mas_bajo,
                total_presentations: acc.total,
                active_presentations: acc.active,
                has_parallel_imports: acc.has_pi,
            }
        })
        .collect();

    groups.sort_by(|a, b| a.code.cmp(&b.code));
    groups
}

/// Generates relational junction rows linking presentations to their homogeneous groups,
/// evaluating whether each presentation is priced at the Precio Más Bajo (PAB).
pub fn generate_group_presentation_mappings(
    products: &[BillingProduct],
    groups: &[HomogeneousGroup],
) -> Vec<GroupPresentationMapping> {
    let pab_map: HashMap<&str, Option<f64>> = groups
        .iter()
        .map(|g| (g.code.as_str(), g.precio_mas_bajo))
        .collect();

    let mut mappings = Vec::new();

    for p in products {
        if let Some(code) = &p.homogeneous_group_code {
            let code_str = code.trim();
            if code_str.is_empty() {
                continue;
            }

            let pab = pab_map.get(code_str).copied().flatten();
            let is_pab = match (p.pvp_iva, pab) {
                (Some(pvp), Some(lowest)) => (pvp - lowest).abs() < 0.005,
                _ => false,
            };

            mappings.push(GroupPresentationMapping {
                group_code: code_str.to_string(),
                group_name: p.homogeneous_group_name.clone().unwrap_or_default(),
                cn: p.cn.clone(),
                product_name: p.name.clone(),
                pvp_iva: p.pvp_iva,
                is_precio_mas_bajo: is_pab,
                status: p.status.clone(),
                is_active: p.is_active,
                is_parallel_import: p.is_parallel_import,
            });
        }
    }

    mappings.sort_by(|a, b| {
        a.group_code
            .cmp(&b.group_code)
            .then_with(|| a.cn.cmp(&b.cn))
    });
    mappings
}

/// Extracts all parallel import presentations from the billing products list.
pub fn extract_parallel_imports(products: &[BillingProduct]) -> Vec<ParallelImportRecord> {
    let mut list = Vec::new();

    for p in products {
        if p.is_parallel_import {
            list.push(ParallelImportRecord {
                cn: p.cn.clone(),
                name: p.name.clone(),
                supplier_lab_name: p.supplier_lab_name.clone(),
                status: p.status.clone(),
                is_active: p.is_active,
                pvp_iva: p.pvp_iva,
                reference_price: p.reference_price,
                homogeneous_group_code: p.homogeneous_group_code.clone(),
                homogeneous_group_name: p.homogeneous_group_name.clone(),
                active_ingredient: p.active_ingredient.clone(),
                detection_source: if detect_parallel_import_heuristic(
                    &p.name,
                    p.supplier_lab_name.as_deref(),
                ) {
                    "pattern_heuristic".to_string()
                } else {
                    "aemps_cross_reference".to_string()
                },
            });
        }
    }

    list.sort_by(|a, b| a.cn.cmp(&b.cn));
    list
}

/// Exports all billing records, computed groups, mappings, and parallel imports
/// into 4 clean CSV files designed for direct database importation.
pub fn export_billing_data_to_csvs<P: AsRef<Path>>(
    products: &[BillingProduct],
    output_dir: P,
) -> Result<BillingExportSummary> {
    let out_dir = output_dir.as_ref();
    std::fs::create_dir_all(out_dir).context("Failed to create billing output directory")?;

    let products_path = out_dir.join("billing_products.csv");
    let groups_path = out_dir.join("homogeneous_groups.csv");
    let mappings_path = out_dir.join("group_presentations.csv");
    let pi_path = out_dir.join("parallel_imports.csv");

    // 1. Export Products
    {
        let mut writer = csv::Writer::from_path(&products_path)
            .with_context(|| format!("Failed to create {:?}", products_path))?;
        for product in products {
            writer.serialize(product)?;
        }
        writer.flush()?;
    }

    // 2. Compute and Export Homogeneous Groups
    let groups = compute_homogeneous_groups(products);
    {
        let mut writer = csv::Writer::from_path(&groups_path)
            .with_context(|| format!("Failed to create {:?}", groups_path))?;
        for group in &groups {
            writer.serialize(group)?;
        }
        writer.flush()?;
    }

    // 3. Compute and Export Group Presentations Mappings
    let mappings = generate_group_presentation_mappings(products, &groups);
    {
        let mut writer = csv::Writer::from_path(&mappings_path)
            .with_context(|| format!("Failed to create {:?}", mappings_path))?;
        for mapping in &mappings {
            writer.serialize(mapping)?;
        }
        writer.flush()?;
    }

    // 4. Extract and Export Parallel Imports
    let parallel_imports = extract_parallel_imports(products);
    {
        let mut writer = csv::Writer::from_path(&pi_path)
            .with_context(|| format!("Failed to create {:?}", pi_path))?;
        for pi in &parallel_imports {
            writer.serialize(pi)?;
        }
        writer.flush()?;
    }

    let active_products = products.iter().filter(|p| p.is_active).count();

    Ok(BillingExportSummary {
        total_products: products.len(),
        active_products,
        total_groups: groups.len(),
        total_parallel_imports: parallel_imports.len(),
        total_mappings: mappings.len(),
        products_csv: products_path,
        groups_csv: groups_path,
        group_presentations_csv: mappings_path,
        parallel_imports_csv: pi_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_BILLING_CSV: &str = r#"Código Nacional,Nombre del producto farmacéutico,Tipo de fármaco,Nombre genérico efecto y accesorio,Código del laboratorio ofertante,Nombre del laboratorio ofertante,Estado,Fecha de alta en el nomenclátor,Fecha de baja en el nomenclátor,Aportación del beneficiario,Principio activo o asociación de principios activos,Precio venta al público con IVA,Precio de referencia,Menor precio de la agrupación homogénea del producto sanitario,Código de la agrupación homogénea del producto sanitario,Nombre de la agrupación homogénea del producto sanitario,Diagnóstico hospitalario,Tratamiento de larga duración,Especial control médico,Medicamento huérfano
650123,PARACETAMOL CINFA 1 G 20 COMPRIMIDOS,Medicamento Generico,,111,CINFA S.A.,ALTA,01/01/2010,,NORMAL,PARACETAMOL,2.50,2.50,2.50,1001,PARACETAMOL 1 G 20 COMPRIMIDOS,NO,SI,NO,NO
650124,PARACETAMOL STADA 1 G 20 COMPRIMIDOS,Medicamento Generico,,222,STADA S.L.,ALTA,01/01/2011,,NORMAL,PARACETAMOL,2.40,2.50,2.50,1001,PARACETAMOL 1 G 20 COMPRIMIDOS,NO,SI,NO,NO
650125,PARACETAMOL BRAND 1 G 20 COMPRIMIDOS,Medicamento Etica,,333,BRAND PHARMA,BAJA GENERAL,01/01/2005,01/01/2020,NORMAL,PARACETAMOL,3.00,2.50,2.50,1001,PARACETAMOL 1 G 20 COMPRIMIDOS,NO,SI,NO,NO
650999,LORMETAZEPAM 1 MG 30 COMPRIMIDOS (I.P.),Medicamento Etica,,444,ABACUS MEDICINE,ALTA,01/06/2021,,NORMAL,LORMETAZEPAM,2.01,2.01,2.01,1941,LORMETAZEPAM 1 MG 30 COMPRIMIDOS,NO,NO,NO,NO
140001,aceite salicilado 50 mg/ml solucion cutanea 100 ml 1 frasco,,,,,ALTA,01/05/1981,,NORMAL,SALICILICO ACIDO,,,.99,,,,NO,NO,NO,NO
"#;

    #[test]
    fn test_parse_billing_csv() {
        let products = parse_billing_csv_reader(SAMPLE_BILLING_CSV.as_bytes()).unwrap();
        assert_eq!(products.len(), 5);

        let p1 = &products[0];
        assert_eq!(p1.cn, "650123");
        assert_eq!(p1.name, "PARACETAMOL CINFA 1 G 20 COMPRIMIDOS");
        assert_eq!(p1.drug_type.as_deref(), Some("Medicamento Generico"));
        assert_eq!(p1.status, "ALTA");
        assert!(p1.is_active);
        assert_eq!(p1.pvp_iva, Some(2.50));
        assert_eq!(p1.group_lowest_price, Some(2.50));
        assert_eq!(p1.homogeneous_group_code.as_deref(), Some("1001"));
        assert_eq!(
            p1.homogeneous_group_name.as_deref(),
            Some("PARACETAMOL 1 G 20 COMPRIMIDOS")
        );
        assert!(p1.long_term_treatment);
        assert!(!p1.hospital_diagnosis);
        assert!(!p1.is_parallel_import);

        let p4 = &products[3];
        assert_eq!(p4.cn, "650999");
        assert!(p4.is_parallel_import); // Detected by (I.P.) and ABACUS MEDICINE

        let p5 = &products[4];
        assert_eq!(p5.cn, "140001");
        assert_eq!(p5.group_lowest_price, Some(0.99)); // Parsed .99
    }

    #[test]
    fn test_compute_homogeneous_groups() {
        let products = parse_billing_csv_reader(SAMPLE_BILLING_CSV.as_bytes()).unwrap();
        let groups = compute_homogeneous_groups(&products);

        assert_eq!(groups.len(), 2);

        let g_paracetamol = groups.iter().find(|g| g.code == "1001").unwrap();
        assert_eq!(g_paracetamol.name, "PARACETAMOL 1 G 20 COMPRIMIDOS");
        assert_eq!(g_paracetamol.precio_menor, Some(2.50));
        // Stada is active at 2.40, Cinfa is 2.50, Brand is baja at 3.00 -> PAB is 2.40
        assert_eq!(g_paracetamol.precio_mas_bajo, Some(2.40));
        assert_eq!(g_paracetamol.total_presentations, 3);
        assert_eq!(g_paracetamol.active_presentations, 2);
        assert!(!g_paracetamol.has_parallel_imports);

        let g_lormetazepam = groups.iter().find(|g| g.code == "1941").unwrap();
        assert_eq!(g_lormetazepam.precio_menor, Some(2.01));
        assert_eq!(g_lormetazepam.precio_mas_bajo, Some(2.01));
        assert_eq!(g_lormetazepam.total_presentations, 1);
        assert!(g_lormetazepam.has_parallel_imports);
    }

    #[test]
    fn test_generate_group_presentation_mappings() {
        let products = parse_billing_csv_reader(SAMPLE_BILLING_CSV.as_bytes()).unwrap();
        let groups = compute_homogeneous_groups(&products);
        let mappings = generate_group_presentation_mappings(&products, &groups);

        assert_eq!(mappings.len(), 4);

        let stada_mapping = mappings.iter().find(|m| m.cn == "650124").unwrap();
        assert!(stada_mapping.is_precio_mas_bajo); // 2.40 matches lowest 2.40

        let cinfa_mapping = mappings.iter().find(|m| m.cn == "650123").unwrap();
        assert!(!cinfa_mapping.is_precio_mas_bajo); // 2.50 does not match lowest 2.40
    }

    #[test]
    fn test_parallel_imports_and_export() {
        let mut products = parse_billing_csv_reader(SAMPLE_BILLING_CSV.as_bytes()).unwrap();

        // Tag an extra CN from prescriptions cross-reference
        let mut external_cns = HashSet::new();
        external_cns.insert("650123".to_string());
        tag_parallel_imports_from_prescriptions(&mut products, &external_cns);

        let pis = extract_parallel_imports(&products);
        assert_eq!(pis.len(), 2);

        let temp_dir = tempfile::tempdir().unwrap();
        let summary = export_billing_data_to_csvs(&products, temp_dir.path()).unwrap();

        assert_eq!(summary.total_products, 5);
        assert_eq!(summary.total_groups, 2);
        assert_eq!(summary.total_parallel_imports, 2);
        assert!(summary.products_csv.exists());
        assert!(summary.groups_csv.exists());
        assert!(summary.group_presentations_csv.exists());
        assert!(summary.parallel_imports_csv.exists());
    }
}
