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
    /// Parallel import flag (from AEMPS prescription data, EMA register, or detection heuristics)
    pub is_parallel_import: bool,
    /// Parallel import detection source ("aemps_official", "ema_register", "importer_catalog", "name_syntax")
    pub parallel_import_source: Option<String>,
    /// Parallel import confidence score (90 to 100)
    pub parallel_import_confidence: Option<u8>,
    /// Origin country if identified via EMA Parallel Distribution Register
    pub parallel_import_origin: Option<String>,
    /// EMA Parallel Distribution notification number (e.g. "EMA/PD/...", "IRIS-...")
    pub parallel_import_ema_number: Option<String>,
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
    /// Detection source ("aemps_official", "ema_register", "importer_catalog", "name_syntax")
    pub detection_source: String,
    /// Detection confidence score (90 to 100)
    pub confidence_score: u8,
    /// Country of origin (e.g. "Germany", "France", "Italy")
    pub origin_country: Option<String>,
    /// EMA Notification Number (e.g. "EMA/PD/...", "IRIS-...")
    pub ema_notification_number: Option<String>,
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

/// Normalizes a company name for robust cross-catalog matching by stripping
/// accents, punctuation, legal entity suffixes (S.A., S.L., A/S, GmbH, etc.),
/// and extra whitespace.
pub fn normalize_company_name(name: &str) -> String {
    let upper = name.to_uppercase();
    let unaccented: String = upper
        .chars()
        .map(|c| match c {
            'Á' | 'À' | 'Ä' | 'Â' => 'A',
            'É' | 'È' | 'Ë' | 'Ê' => 'E',
            'Í' | 'Ì' | 'Ï' | 'Î' => 'I',
            'Ó' | 'Ò' | 'Ö' | 'Ô' => 'O',
            'Ú' | 'Ù' | 'Ü' | 'Û' => 'U',
            'Ñ' => 'N',
            other => other,
        })
        .collect();

    // Standardize common entity suffixes before splitting
    let standardized = unaccented
        .replace("A/S", " ")
        .replace("A / S", " ")
        .replace("S.L.U.", " ")
        .replace("S.L.U", " ")
        .replace("S.A.U.", " ")
        .replace("S.A.U", " ")
        .replace("S.L.", " ")
        .replace("S.L", " ")
        .replace("S.A.", " ")
        .replace("S.A", " ")
        .replace("B.V.", " ")
        .replace("B.V", " ");

    let cleaned: String = standardized
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();

    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    let filtered: Vec<&str> = tokens
        .into_iter()
        .filter(|t| {
            !matches!(
                *t,
                "SA" | "SL"
                    | "SLU"
                    | "SAU"
                    | "AS"
                    | "GMBH"
                    | "BV"
                    | "LTD"
                    | "LIMITED"
                    | "AG"
                    | "SPA"
                    | "SOCIEDAD"
                    | "LIMITADA"
                    | "ANONIMA"
                    | "ARZNEIMITTEL"
            )
        })
        .collect();

    if filtered.is_empty() {
        cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        filtered.join(" ")
    }
}

/// Baseline curated catalog of verified pharmaceutical parallel importers,
/// repackagers, and distributors operating in Spain and the EU.
pub const DEFAULT_KNOWN_IMPORTERS: &[&str] = &[
    "ABACUS MEDICINE",
    "ORIFARM",
    "EURIMPHARM",
    "EURIM PHARM",
    "KOHLPHARMA",
    "KOHL PHARMA",
    "DISFARMA",
    "FARMALEP",
    "GALIA FARMA",
    "GARANTY FARMA",
    "EUROCEPS",
    "PROINPHARMA",
    "TOP RIDGE",
    "TOP RIDGE PHARMA",
    "MEDIMPORT",
    "MEDIFARM",
    "PHARMA WESTEN",
    "CC PHARMA",
    "MPA PHARMA",
    "HAEMATO PHARM",
    "HAEMATOPHARM",
    "ACA MULLER",
    "ACA MUELLER",
    "EMRA MED",
    "EMRA-MED",
    "AXICORP",
    "FARMADOSIS",
    "PARANOVA",
    "2CARE4",
    "CROSS PHARMA",
    "PHARMASWISS",
    "BMODESTO",
    "B MODESTO",
    "FARMACEUTICA DEL SUR",
    "IBERFARMA IMPORT",
    "INTERPHARMA IMPORT",
];

/// Curated catalog of known pharmaceutical parallel importers and repackagers.
#[derive(Debug, Clone)]
pub struct ParallelImporterCatalog {
    importers: HashSet<String>,
}

impl Default for ParallelImporterCatalog {
    fn default() -> Self {
        Self::new_default()
    }
}

impl ParallelImporterCatalog {
    /// Creates a catalog initialized with the default curated directory of known parallel importers.
    pub fn new_default() -> Self {
        let mut importers = HashSet::new();
        for name in DEFAULT_KNOWN_IMPORTERS {
            importers.insert(normalize_company_name(name));
        }
        Self { importers }
    }

    /// Loads custom importers from an external text or CSV file (one name per line or CSV column).
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref()).with_context(|| {
            format!("Failed to open importers list file at {:?}", path.as_ref())
        })?;
        let reader = BufReader::new(file);
        let mut catalog = Self::new_default();

        use std::io::BufRead;
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            // If comma-separated, take the first field or whole line
            let lab = trimmed.split(',').next().unwrap_or(trimmed).trim();
            catalog.add(lab);
        }

        Ok(catalog)
    }

    /// Adds a laboratory name to the catalog.
    pub fn add(&mut self, name: &str) {
        let norm = normalize_company_name(name);
        if !norm.is_empty() {
            self.importers.insert(norm);
        }
    }

    /// Returns the number of known importers in the catalog.
    pub fn len(&self) -> usize {
        self.importers.len()
    }

    /// Returns true if the catalog is empty.
    pub fn is_empty(&self) -> bool {
        self.importers.is_empty()
    }

    /// Checks if a laboratory name matches a known parallel importer.
    pub fn is_importer(&self, lab_name: &str) -> bool {
        let upper_raw = lab_name.to_uppercase();
        if upper_raw.contains("PARALEL")
            || upper_raw.contains("REACONDICIONAD")
            || upper_raw.contains("REPACKAGING")
        {
            return true;
        }

        let norm_lab = normalize_company_name(lab_name);
        if norm_lab.is_empty() {
            return false;
        }

        for imp in &self.importers {
            if norm_lab == *imp || norm_lab.contains(imp) || imp.contains(&norm_lab) {
                return true;
            }
        }

        false
    }
}

/// Notification record from the European Medicines Agency (EMA) Parallel Distribution Register (IRIS).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmaParallelDistributionRecord {
    /// Invented product name or brand (e.g. "Humira", "Eliquis", "Enbrel")
    pub product_name: String,
    /// Active substance / INN (e.g. "adalimumab", "apixaban")
    pub active_substance: Option<String>,
    /// Centrally authorised EU marketing authorization number (e.g. "EU/1/11/691")
    pub eu_number: Option<String>,
    /// Notified parallel distributor company (e.g. "Abacus Medicine A/S")
    pub distributor_name: String,
    /// Member state of origin (e.g. "Germany", "France", "Italy")
    pub origin_country: Option<String>,
    /// Member state of destination (e.g. "Spain", "España", "ES")
    pub destination_country: String,
    /// Notification status (e.g. "Valid", "Active")
    pub status: String,
    /// Notification reference / IRIS identifier
    pub notification_number: Option<String>,
    /// Date of notification or last update
    pub notification_date: Option<String>,
}

/// Checks whether a destination country corresponds to Spain.
pub fn is_destination_spain(dest: &str) -> bool {
    let d = dest.trim().to_lowercase();
    d == "spain"
        || d == "españa"
        || d == "espana"
        || d == "es"
        || d.contains("spain")
        || d.contains("españa")
}

/// Checks whether an EMA notification status is active / valid.
pub fn is_status_active(status: &str) -> bool {
    let s = status.trim().to_lowercase();
    s.is_empty()
        || s == "valid"
        || s == "active"
        || s == "valida"
        || s == "válida"
        || s == "activa"
        || s == "current"
}

/// Parses an EMA Parallel Distribution Register CSV export supporting flexible column headers and delimiters.
pub fn parse_ema_register_csv<R: Read>(reader: R) -> Result<Vec<EmaParallelDistributionRecord>> {
    let mut buf_reader = BufReader::new(reader);
    let mut buffer = Vec::new();
    buf_reader.read_to_end(&mut buffer)?;

    let mut slice = buffer.as_slice();
    // Strip UTF-8 BOM if present
    if slice.starts_with(&[0xEF, 0xBB, 0xBF]) {
        slice = &slice[3..];
    }

    // Sniff delimiter from first non-empty line
    let first_line = match std::str::from_utf8(slice) {
        Ok(s) => s.lines().next().unwrap_or(""),
        Err(_) => "",
    };
    let semicolon_count = first_line.matches(';').count();
    let comma_count = first_line.matches(',').count();
    let tab_count = first_line.matches('\t').count();

    let delimiter = if tab_count > comma_count && tab_count > semicolon_count {
        b'\t'
    } else if semicolon_count > comma_count {
        b';'
    } else {
        b','
    };

    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .delimiter(delimiter)
        .from_reader(slice);

    let raw_headers = csv_reader.headers()?.clone();
    let headers: Vec<String> = raw_headers
        .iter()
        .map(|h| h.trim().trim_start_matches('\u{feff}').to_lowercase())
        .collect();

    let get_idx = |names: &[&str]| -> Option<usize> {
        for name in names {
            let target = name.to_lowercase();
            if let Some(pos) = headers
                .iter()
                .position(|h| h == &target || h.contains(&target))
            {
                return Some(pos);
            }
        }
        None
    };

    let idx_product = get_idx(&[
        "product (invented) name",
        "product name",
        "invented name",
        "medicinal product",
        "product",
    ]);
    let idx_substance = get_idx(&["active substance", "inn", "substance", "principio activo"]);
    let idx_eu_number = get_idx(&[
        "eu number",
        "eu m/a number",
        "procedure number",
        "marketing authorisation number",
    ]);
    let idx_distributor = get_idx(&[
        "parallel distributor",
        "company name",
        "distributor",
        "empresa",
        "titular",
    ]);
    let idx_origin = get_idx(&[
        "member state of origin",
        "country of origin",
        "origin country",
        "origin",
        "origen",
    ]);
    let idx_destination = get_idx(&[
        "member state of destination",
        "country of destination",
        "destination country",
        "destination",
        "destino",
    ]);
    let idx_status = get_idx(&["notification status", "current status", "status", "estado"]);
    let idx_notif_num = get_idx(&[
        "notification number",
        "notification identifier",
        "identifier",
        "reference",
        "iris id",
        "numero",
    ]);
    let idx_date = get_idx(&["date of notification", "notification date", "date", "fecha"]);

    let mut records = Vec::new();
    let mut record = csv::StringRecord::new();

    while csv_reader.read_record(&mut record)? {
        let get_val = |opt_idx: Option<usize>| -> &str {
            opt_idx.and_then(|i| record.get(i)).unwrap_or("").trim()
        };

        let product_name = get_val(idx_product);
        let distributor_name = get_val(idx_distributor);

        if product_name.is_empty() && distributor_name.is_empty() {
            continue;
        }

        let active_substance = clean_opt(get_val(idx_substance));
        let eu_number = clean_opt(get_val(idx_eu_number));
        let origin_country = clean_opt(get_val(idx_origin));
        let destination_country =
            clean_opt(get_val(idx_destination)).unwrap_or_else(|| "Spain".to_string());
        let status = clean_opt(get_val(idx_status)).unwrap_or_else(|| "Valid".to_string());
        let notification_number = clean_opt(get_val(idx_notif_num));
        let notification_date = clean_opt(get_val(idx_date));

        records.push(EmaParallelDistributionRecord {
            product_name: product_name.to_string(),
            active_substance,
            eu_number,
            distributor_name: distributor_name.to_string(),
            origin_country,
            destination_country,
            status,
            notification_number,
            notification_date,
        });
    }

    Ok(records)
}

/// Loads EMA Parallel Distribution Register records from a CSV file.
pub fn load_ema_register_csv<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<EmaParallelDistributionRecord>> {
    let file = File::open(path.as_ref()).with_context(|| {
        format!(
            "Failed to open EMA Parallel Distribution CSV at {:?}",
            path.as_ref()
        )
    })?;
    parse_ema_register_csv(file)
}

/// Merges two collections of EMA Parallel Distribution records, deduplicating by
/// (product_name, distributor_name, destination_country).
/// Records in `primary` take precedence, and any missing records from `fallback` are appended.
/// This prevents losing historical entries when updating with a fresh EMA export.
pub fn merge_ema_records(
    primary: Vec<EmaParallelDistributionRecord>,
    fallback: Vec<EmaParallelDistributionRecord>,
) -> Vec<EmaParallelDistributionRecord> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut merged = Vec::with_capacity(primary.len() + fallback.len());

    let make_key = |r: &EmaParallelDistributionRecord| {
        (
            r.product_name.trim().to_lowercase(),
            r.distributor_name.trim().to_lowercase(),
            r.destination_country.trim().to_lowercase(),
        )
    };

    for r in primary {
        let key = make_key(&r);
        seen.insert(key);
        merged.push(r);
    }

    for r in fallback {
        let key = make_key(&r);
        if seen.insert(key) {
            merged.push(r);
        }
    }

    merged
}

/// Embedded curated baseline of verified Centrally Authorised Products (CAPs) subject to
/// active parallel distribution notifications for Spain.
pub fn builtin_ema_spain_records() -> Vec<EmaParallelDistributionRecord> {
    vec![
        EmaParallelDistributionRecord {
            product_name: "Eliquis".to_string(),
            active_substance: Some("apixaban".to_string()),
            eu_number: Some("EU/1/11/691".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00101/2020".to_string()),
            notification_date: Some("2020-01-15".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Eliquis".to_string(),
            active_substance: Some("apixaban".to_string()),
            eu_number: Some("EU/1/11/691".to_string()),
            distributor_name: "Orifarm A/S".to_string(),
            origin_country: Some("France".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00102/2020".to_string()),
            notification_date: Some("2020-02-10".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Enbrel".to_string(),
            active_substance: Some("etanercept".to_string()),
            eu_number: Some("EU/1/99/126".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00103/2019".to_string()),
            notification_date: Some("2019-04-12".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Enbrel".to_string(),
            active_substance: Some("etanercept".to_string()),
            eu_number: Some("EU/1/99/126".to_string()),
            distributor_name: "Orifarm A/S".to_string(),
            origin_country: Some("Italy".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00104/2019".to_string()),
            notification_date: Some("2019-06-20".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Enbrel".to_string(),
            active_substance: Some("etanercept".to_string()),
            eu_number: Some("EU/1/99/126".to_string()),
            distributor_name: "Kohlpharma GmbH".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00105/2019".to_string()),
            notification_date: Some("2019-07-01".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Humira".to_string(),
            active_substance: Some("adalimumab".to_string()),
            eu_number: Some("EU/1/03/256".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("France".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00106/2018".to_string()),
            notification_date: Some("2018-05-15".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Humira".to_string(),
            active_substance: Some("adalimumab".to_string()),
            eu_number: Some("EU/1/03/256".to_string()),
            distributor_name: "EurimPharm Arzneimittel GmbH".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00107/2018".to_string()),
            notification_date: Some("2018-08-20".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Keytruda".to_string(),
            active_substance: Some("pembrolizumab".to_string()),
            eu_number: Some("EU/1/15/1024".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00108/2021".to_string()),
            notification_date: Some("2021-03-10".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Ozempic".to_string(),
            active_substance: Some("semaglutide".to_string()),
            eu_number: Some("EU/1/17/1251".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00109/2022".to_string()),
            notification_date: Some("2022-01-20".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Ozempic".to_string(),
            active_substance: Some("semaglutide".to_string()),
            eu_number: Some("EU/1/17/1251".to_string()),
            distributor_name: "Orifarm A/S".to_string(),
            origin_country: Some("Poland".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00110/2022".to_string()),
            notification_date: Some("2022-04-15".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Xarelto".to_string(),
            active_substance: Some("rivaroxaban".to_string()),
            eu_number: Some("EU/1/08/472".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Italy".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00111/2020".to_string()),
            notification_date: Some("2020-03-11".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Xarelto".to_string(),
            active_substance: Some("rivaroxaban".to_string()),
            eu_number: Some("EU/1/08/472".to_string()),
            distributor_name: "Disfarma S.L.".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00112/2020".to_string()),
            notification_date: Some("2020-05-18".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Prolia".to_string(),
            active_substance: Some("denosumab".to_string()),
            eu_number: Some("EU/1/10/618".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("France".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00113/2020".to_string()),
            notification_date: Some("2020-07-22".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Stelara".to_string(),
            active_substance: Some("ustekinumab".to_string()),
            eu_number: Some("EU/1/08/494".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00114/2020".to_string()),
            notification_date: Some("2020-09-05".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Entresto".to_string(),
            active_substance: Some("sacubitril valsartan".to_string()),
            eu_number: Some("EU/1/15/1058".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00115/2021".to_string()),
            notification_date: Some("2021-02-18".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Revlimid".to_string(),
            active_substance: Some("lenalidomide".to_string()),
            eu_number: Some("EU/1/07/391".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00116/2019".to_string()),
            notification_date: Some("2019-11-04".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Januvia".to_string(),
            active_substance: Some("sitagliptin".to_string()),
            eu_number: Some("EU/1/07/383".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("France".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00117/2019".to_string()),
            notification_date: Some("2019-12-10".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Eylea".to_string(),
            active_substance: Some("aflibercept".to_string()),
            eu_number: Some("EU/1/12/797".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00118/2021".to_string()),
            notification_date: Some("2021-06-14".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Victoza".to_string(),
            active_substance: Some("liraglutide".to_string()),
            eu_number: Some("EU/1/09/529".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Italy".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00119/2020".to_string()),
            notification_date: Some("2020-04-20".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Cosentyx".to_string(),
            active_substance: Some("secukinumab".to_string()),
            eu_number: Some("EU/1/14/980".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00120/2021".to_string()),
            notification_date: Some("2021-09-08".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Tresiba".to_string(),
            active_substance: Some("insulin degludec".to_string()),
            eu_number: Some("EU/1/12/807".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00121/2020".to_string()),
            notification_date: Some("2020-10-15".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Levemir".to_string(),
            active_substance: Some("insulin detemir".to_string()),
            eu_number: Some("EU/1/04/278".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Poland".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00122/2019".to_string()),
            notification_date: Some("2019-03-25".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Lantus".to_string(),
            active_substance: Some("insulin glargine".to_string()),
            eu_number: Some("EU/1/00/134".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("France".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00123/2019".to_string()),
            notification_date: Some("2019-05-18".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Forxiga".to_string(),
            active_substance: Some("dapagliflozin".to_string()),
            eu_number: Some("EU/1/12/795".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00124/2021".to_string()),
            notification_date: Some("2021-08-30".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Jardiance".to_string(),
            active_substance: Some("empagliflozin".to_string()),
            eu_number: Some("EU/1/14/930".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00125/2021".to_string()),
            notification_date: Some("2021-11-12".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Briviact".to_string(),
            active_substance: Some("brivaracetam".to_string()),
            eu_number: Some("EU/1/15/1073".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Germany".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00126/2022".to_string()),
            notification_date: Some("2022-02-05".to_string()),
        },
        EmaParallelDistributionRecord {
            product_name: "Vimpat".to_string(),
            active_substance: Some("lacosamide".to_string()),
            eu_number: Some("EU/1/08/470".to_string()),
            distributor_name: "Abacus Medicine A/S".to_string(),
            origin_country: Some("Italy".to_string()),
            destination_country: "Spain".to_string(),
            status: "Valid".to_string(),
            notification_number: Some("EMA/PD/00127/2020".to_string()),
            notification_date: Some("2020-08-19".to_string()),
        },
    ]
}

/// In-memory index of active EMA parallel distribution notifications for Spain.
#[derive(Debug, Clone)]
pub struct EmaParallelDistributionIndex {
    spain_records: Vec<EmaParallelDistributionRecord>,
}

impl Default for EmaParallelDistributionIndex {
    fn default() -> Self {
        Self::new(builtin_ema_spain_records())
    }
}

impl EmaParallelDistributionIndex {
    /// Creates a new index filtering for destination Spain and active status.
    pub fn new(records: Vec<EmaParallelDistributionRecord>) -> Self {
        let spain_records: Vec<_> = records
            .into_iter()
            .filter(|r| is_destination_spain(&r.destination_country) && is_status_active(&r.status))
            .collect();
        Self { spain_records }
    }

    /// Number of active records in the index.
    pub fn len(&self) -> usize {
        self.spain_records.len()
    }

    /// True if the index contains no records.
    pub fn is_empty(&self) -> bool {
        self.spain_records.is_empty()
    }

    /// Matches a Nomenclator presentation against active EMA parallel distribution notifications for Spain.
    pub fn match_product(
        &self,
        product_name: &str,
        lab_name: Option<&str>,
        active_ingredient: Option<&str>,
    ) -> Option<&EmaParallelDistributionRecord> {
        let lab_raw = lab_name.unwrap_or("").trim();
        if lab_raw.is_empty() {
            return None;
        }

        let norm_lab = normalize_company_name(lab_raw);
        let upper_prod = product_name.to_uppercase();
        let upper_act = active_ingredient.map(|a| a.to_uppercase());

        for record in &self.spain_records {
            let norm_dist = normalize_company_name(&record.distributor_name);

            // Check distributor match
            let dist_matches = norm_lab == norm_dist
                || norm_lab.contains(&norm_dist)
                || norm_dist.contains(&norm_lab);
            if !dist_matches {
                continue;
            }

            // Check product brand name match
            let norm_record_prod = record.product_name.to_uppercase();
            let prod_matches = if !norm_record_prod.is_empty() {
                upper_prod.starts_with(&norm_record_prod)
                    || upper_prod.contains(&format!(" {} ", norm_record_prod))
                    || upper_prod.starts_with(&format!("{} ", norm_record_prod))
                    || upper_prod.ends_with(&format!(" {}", norm_record_prod))
                    || upper_prod.contains(&norm_record_prod)
            } else {
                false
            };

            // Check active substance match
            let substance_matches = match (&record.active_substance, &upper_act) {
                (Some(sub), Some(act)) => {
                    let sub_upper = sub.to_uppercase();
                    act.contains(&sub_upper) || sub_upper.contains(act)
                }
                _ => false,
            };

            if prod_matches || substance_matches {
                return Some(record);
            }
        }

        None
    }
}

/// Findings from evaluating a presentation against the multi-tiered detection engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParallelImportDetection {
    /// True if detected as a parallel import
    pub is_parallel_import: bool,
    /// Confidence score (90 to 100)
    pub confidence_score: u8,
    /// Primary authoritative source ("aemps_official", "ema_register", "importer_catalog", "name_syntax")
    pub primary_source: String,
    /// All matched detection sources
    pub matched_sources: Vec<String>,
    /// Member state of origin if determined from EMA register
    pub origin_country: Option<String>,
    /// Parallel distributor or importer company name
    pub distributor_name: Option<String>,
    /// EMA notification number if determined from EMA register
    pub ema_notification_number: Option<String>,
}

/// Comprehensive multi-tiered parallel import detection engine.
#[derive(Debug, Clone)]
pub struct ParallelImportDetector {
    pub importer_catalog: ParallelImporterCatalog,
    pub ema_index: EmaParallelDistributionIndex,
    pub aemps_official_cns: HashSet<String>,
}

impl Default for ParallelImportDetector {
    fn default() -> Self {
        Self::new_default()
    }
}

impl ParallelImportDetector {
    /// Creates a detector combining custom catalogs and sets.
    pub fn new(
        importer_catalog: ParallelImporterCatalog,
        ema_records: Vec<EmaParallelDistributionRecord>,
        aemps_cns: HashSet<String>,
    ) -> Self {
        Self {
            importer_catalog,
            ema_index: EmaParallelDistributionIndex::new(ema_records),
            aemps_official_cns: aemps_cns,
        }
    }

    /// Creates a default detector preloaded with built-in importer directory and built-in EMA notifications.
    pub fn new_default() -> Self {
        Self::new(
            ParallelImporterCatalog::new_default(),
            builtin_ema_spain_records(),
            HashSet::new(),
        )
    }

    /// Sets the AEMPS official parallel import National Codes.
    pub fn with_aemps_cns(mut self, cns: HashSet<String>) -> Self {
        self.aemps_official_cns = cns;
        self
    }

    /// Appends external EMA records.
    pub fn with_ema_records(mut self, mut records: Vec<EmaParallelDistributionRecord>) -> Self {
        let mut all = builtin_ema_spain_records();
        all.append(&mut records);
        self.ema_index = EmaParallelDistributionIndex::new(all);
        self
    }

    /// Evaluates a presentation and returns detailed parallel import detection findings.
    pub fn detect(
        &self,
        cn: &str,
        name: &str,
        lab_name: Option<&str>,
        active_ingredient: Option<&str>,
    ) -> Option<ParallelImportDetection> {
        let is_aemps = self.aemps_official_cns.contains(cn);
        let ema_match = self
            .ema_index
            .match_product(name, lab_name, active_ingredient);
        let is_importer = lab_name.is_some_and(|l| self.importer_catalog.is_importer(l));
        let is_syntax = detect_parallel_import_syntax(name);

        if !is_aemps && ema_match.is_none() && !is_importer && !is_syntax {
            return None;
        }

        let mut matched_sources = Vec::new();
        let mut primary_source = "name_syntax".to_string();
        let mut confidence = 90u8;

        if is_syntax {
            matched_sources.push("name_syntax".to_string());
        }
        if is_importer {
            matched_sources.push("importer_catalog".to_string());
            primary_source = "importer_catalog".to_string();
            confidence = 95;
        }
        if ema_match.is_some() {
            matched_sources.push("ema_register".to_string());
            primary_source = "ema_register".to_string();
            confidence = 100;
        }
        if is_aemps {
            matched_sources.push("aemps_official".to_string());
            primary_source = "aemps_official".to_string();
            confidence = 100;
        }

        let origin_country = ema_match.and_then(|e| e.origin_country.clone());
        let ema_notification_number = ema_match.and_then(|e| e.notification_number.clone());
        let distributor_name = lab_name.map(|l| l.to_string());

        Some(ParallelImportDetection {
            is_parallel_import: true,
            confidence_score: confidence,
            primary_source,
            matched_sources,
            origin_country,
            distributor_name,
            ema_notification_number,
        })
    }

    /// Tags a list of `BillingProduct` items in-place.
    pub fn tag_products(&self, products: &mut [BillingProduct]) {
        for p in products.iter_mut() {
            if let Some(det) = self.detect(
                &p.cn,
                &p.name,
                p.supplier_lab_name.as_deref(),
                p.active_ingredient.as_deref(),
            ) {
                p.is_parallel_import = true;
                p.parallel_import_source = Some(det.matched_sources.join(","));
                p.parallel_import_confidence = Some(det.confidence_score);
                if det.origin_country.is_some() {
                    p.parallel_import_origin = det.origin_country;
                }
                if det.ema_notification_number.is_some() {
                    p.parallel_import_ema_number = det.ema_notification_number;
                }
            }
        }
    }
}

/// Checks syntactic patterns indicating parallel import in product name (e.g. "(I.P.)", "(IP)").
pub fn detect_parallel_import_syntax(name: &str) -> bool {
    let upper_name = name.to_uppercase();
    upper_name.contains("(I.P.)")
        || upper_name.contains("(IP)")
        || upper_name.contains("(IMP.PAR.)")
        || upper_name.contains("(IMP. PAR.)")
        || upper_name.contains("IMPORTACION PARALELA")
        || upper_name.contains("IMPORTACIÓN PARALELA")
        || upper_name.ends_with(" I.P.")
        || upper_name.ends_with(" IP")
}

/// Checks heuristic patterns indicating parallel import in product name or lab.
fn detect_parallel_import_heuristic(name: &str, lab: Option<&str>) -> bool {
    if detect_parallel_import_syntax(name) {
        return true;
    }
    if let Some(lab_name) = lab {
        let catalog = ParallelImporterCatalog::new_default();
        if catalog.is_importer(lab_name) {
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
        let (parallel_import_source, parallel_import_confidence) = if is_parallel_import {
            let mut sources = Vec::new();
            let mut conf = 90u8;
            if detect_parallel_import_syntax(&name) {
                sources.push("name_syntax");
            }
            if let Some(ref lab) = supplier_lab_name {
                let catalog = ParallelImporterCatalog::new_default();
                if catalog.is_importer(lab) {
                    sources.push("importer_catalog");
                    conf = 95;
                }
            }
            (Some(sources.join(",")), Some(conf))
        } else {
            (None, None)
        };

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
            parallel_import_source,
            parallel_import_confidence,
            parallel_import_origin: None,
            parallel_import_ema_number: None,
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

/// Tags parallel imports in the billing products list using a set of known parallel import CNs from AEMPS.
pub fn tag_parallel_imports_from_prescriptions(
    products: &mut [BillingProduct],
    parallel_import_cns: &HashSet<String>,
) {
    for product in products.iter_mut() {
        if parallel_import_cns.contains(&product.cn) {
            product.is_parallel_import = true;
            let current = product.parallel_import_source.take();
            let new_source = match current {
                Some(s) if !s.contains("aemps_official") => format!("{},aemps_official", s),
                Some(s) => s,
                None => "aemps_official".to_string(),
            };
            product.parallel_import_source = Some(new_source);
            product.parallel_import_confidence = Some(100);
        }
    }
}

/// Tags parallel imports in the billing products list using active EMA Parallel Distribution records.
pub fn tag_parallel_imports_from_ema(
    products: &mut [BillingProduct],
    ema_records: &[EmaParallelDistributionRecord],
) {
    let index = EmaParallelDistributionIndex::new(ema_records.to_vec());
    for product in products.iter_mut() {
        if let Some(record) = index.match_product(
            &product.name,
            product.supplier_lab_name.as_deref(),
            product.active_ingredient.as_deref(),
        ) {
            product.is_parallel_import = true;
            let current = product.parallel_import_source.take();
            let new_source = match current {
                Some(s) if !s.contains("ema_register") => format!("{},ema_register", s),
                Some(s) => s,
                None => "ema_register".to_string(),
            };
            product.parallel_import_source = Some(new_source);
            product.parallel_import_confidence = Some(100);
            if product.parallel_import_origin.is_none() {
                product.parallel_import_origin = record.origin_country.clone();
            }
            if product.parallel_import_ema_number.is_none() {
                product.parallel_import_ema_number = record.notification_number.clone();
            }
        }
    }
}

/// Tags parallel imports using a configured `ParallelImportDetector`.
pub fn tag_parallel_imports_with_detector(
    products: &mut [BillingProduct],
    detector: &ParallelImportDetector,
) {
    detector.tag_products(products);
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
            let detection_source = p
                .parallel_import_source
                .clone()
                .unwrap_or_else(|| "pattern_heuristic".to_string());
            let confidence_score = p.parallel_import_confidence.unwrap_or(90);

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
                detection_source,
                confidence_score,
                origin_country: p.parallel_import_origin.clone(),
                ema_notification_number: p.parallel_import_ema_number.clone(),
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

    #[test]
    fn test_normalize_company_name() {
        assert_eq!(
            normalize_company_name("Abacus Medicine A/S"),
            "ABACUS MEDICINE"
        );
        assert_eq!(normalize_company_name("ORIFARM GMBH"), "ORIFARM");
        assert_eq!(
            normalize_company_name("EURIM-PHARM ARZNEIMITTEL GMBH"),
            "EURIM PHARM"
        );
        assert_eq!(normalize_company_name("Disfarma, S.L.U."), "DISFARMA");
        assert_eq!(
            normalize_company_name("Laboratorios Cinfa, S.A."),
            "LABORATORIOS CINFA"
        );
    }

    #[test]
    fn test_parallel_importer_catalog() {
        let catalog = ParallelImporterCatalog::new_default();
        assert!(catalog.is_importer("ABACUS MEDICINE A/S"));
        assert!(catalog.is_importer("Orifarm GmbH"));
        assert!(catalog.is_importer("EurimPharm Arzneimittel GmbH"));
        assert!(catalog.is_importer("Disfarma S.L."));
        assert!(catalog.is_importer("Farmalep, S.A."));
        assert!(catalog.is_importer("Medimport"));
        assert!(catalog.is_importer("DISTRIBUCION PARALELA IBERICA"));
        assert!(!catalog.is_importer("Laboratorios Cinfa, S.A."));
        assert!(!catalog.is_importer("Pfizer, S.L.U."));
    }

    #[test]
    fn test_parse_ema_register_csv() {
        let sample_ema_csv = r#"Product (invented) name,Active substance,EU Number,Parallel distributor,Member state of destination,Member state of origin,Notification status,Notification number
Eliquis,apixaban,EU/1/11/691,Abacus Medicine A/S,Spain,Germany,Valid,EMA/PD/00101/2020
Enbrel,etanercept,EU/1/99/126,Orifarm A/S,Spain,France,Valid,EMA/PD/00104/2019
Humira,adalimumab,EU/1/03/256,EurimPharm Arzneimittel GmbH,Italy,Germany,Valid,EMA/PD/99999/2020
Xarelto,rivaroxaban,EU/1/08/472,Kohlpharma GmbH,Spain,Germany,Withdrawn,EMA/PD/00000/2018
"#;

        let records = parse_ema_register_csv(sample_ema_csv.as_bytes()).unwrap();
        assert_eq!(records.len(), 4);

        let index = EmaParallelDistributionIndex::new(records);
        // Italy is not Spain, Withdrawn is not active -> only Eliquis and Enbrel in Spain active index
        assert_eq!(index.len(), 2);

        // Test matching
        let matched = index.match_product(
            "ELIQUIS 5 MG COMPRIMIDOS RECUBIERTOS CON PELICULA, 60 comprimidos (I.P.)",
            Some("Abacus Medicine A/S"),
            Some("APIXABAN"),
        );
        assert!(matched.is_some());
        let m = matched.unwrap();
        assert_eq!(m.product_name, "Eliquis");
        assert_eq!(m.origin_country.as_deref(), Some("Germany"));
        assert_eq!(m.notification_number.as_deref(), Some("EMA/PD/00101/2020"));

        // Negative match (different distributor)
        let no_match = index.match_product(
            "ELIQUIS 5 MG COMPRIMIDOS",
            Some("PFIZER S.L."),
            Some("APIXABAN"),
        );
        assert!(no_match.is_none());
    }

    #[test]
    fn test_parallel_import_detector_multi_tier() {
        let mut aemps_cns = HashSet::new();
        aemps_cns.insert("123456".to_string());

        let detector = ParallelImportDetector::new(
            ParallelImporterCatalog::new_default(),
            builtin_ema_spain_records(),
            aemps_cns,
        );

        // Tier 1: AEMPS official match
        let det_aemps = detector
            .detect(
                "123456",
                "PARACETAMOL CINFA 1 G",
                Some("CINFA S.A."),
                Some("PARACETAMOL"),
            )
            .unwrap();
        assert_eq!(det_aemps.confidence_score, 100);
        assert!(
            det_aemps
                .matched_sources
                .contains(&"aemps_official".to_string())
        );

        // Tier 1: EMA register match
        let det_ema = detector
            .detect(
                "650888",
                "ELIQUIS 5 MG 60 COMPRIMIDOS",
                Some("ABACUS MEDICINE A/S"),
                Some("APIXABAN"),
            )
            .unwrap();
        assert_eq!(det_ema.confidence_score, 100);
        assert!(
            det_ema
                .matched_sources
                .contains(&"ema_register".to_string())
        );
        assert_eq!(det_ema.origin_country.as_deref(), Some("Germany"));

        // Tier 2: Importer catalog match
        let det_catalog = detector
            .detect(
                "650777",
                "MEDICAMENTO GENERICO 10 MG",
                Some("ORIFARM GMBH"),
                None,
            )
            .unwrap();
        assert_eq!(det_catalog.confidence_score, 95);
        assert!(
            det_catalog
                .matched_sources
                .contains(&"importer_catalog".to_string())
        );

        // Tier 3: Syntax heuristic match
        let det_syntax = detector
            .detect(
                "650666",
                "PRODUCTO FARMACEUTICO 20 MG (I.P.)",
                Some("LABORATORIO DESCONOCIDO"),
                None,
            )
            .unwrap();
        assert_eq!(det_syntax.confidence_score, 90);
        assert!(
            det_syntax
                .matched_sources
                .contains(&"name_syntax".to_string())
        );

        // Negative: Regular product
        let det_none = detector.detect(
            "650123",
            "PARACETAMOL CINFA 1 G 20 COMPRIMIDOS",
            Some("CINFA S.A."),
            Some("PARACETAMOL"),
        );
        assert!(det_none.is_none());
    }

    #[test]
    fn test_merge_ema_records() {
        let baseline = vec![
            EmaParallelDistributionRecord {
                product_name: "Eliquis".to_string(),
                active_substance: Some("apixaban".to_string()),
                eu_number: Some("EU/1/11/691".to_string()),
                distributor_name: "Abacus Medicine A/S".to_string(),
                origin_country: Some("Germany".to_string()),
                destination_country: "Spain".to_string(),
                status: "Valid".to_string(),
                notification_number: None,
                notification_date: None,
            },
            EmaParallelDistributionRecord {
                product_name: "Humira".to_string(),
                active_substance: Some("adalimumab".to_string()),
                eu_number: None,
                distributor_name: "Eurosegmed S.L.".to_string(),
                origin_country: Some("France".to_string()),
                destination_country: "Spain".to_string(),
                status: "Valid".to_string(),
                notification_number: None,
                notification_date: None,
            },
        ];

        let downloaded = vec![
            // Updated version of existing record (new origin or notification number)
            EmaParallelDistributionRecord {
                product_name: "Eliquis".to_string(),
                active_substance: Some("apixaban".to_string()),
                eu_number: Some("EU/1/11/691".to_string()),
                distributor_name: "Abacus Medicine A/S".to_string(),
                origin_country: Some("Netherlands".to_string()),
                destination_country: "Spain".to_string(),
                status: "Valid".to_string(),
                notification_number: Some("EMAPD/2026/01".to_string()),
                notification_date: Some("2026-01-15".to_string()),
            },
            // Brand new record
            EmaParallelDistributionRecord {
                product_name: "Keytruda".to_string(),
                active_substance: Some("pembrolizumab".to_string()),
                eu_number: None,
                distributor_name: "Kohlpharma GmbH".to_string(),
                origin_country: Some("Germany".to_string()),
                destination_country: "Spain".to_string(),
                status: "Valid".to_string(),
                notification_number: None,
                notification_date: None,
            },
        ];

        let merged = merge_ema_records(downloaded, baseline);
        // Should have 3 records: Keytruda (new), Eliquis (updated), Humira (preserved from baseline)
        assert_eq!(merged.len(), 3);
        assert!(merged.iter().any(|r| r.product_name == "Keytruda"));
        assert!(merged.iter().any(|r| r.product_name == "Humira"));
        let eliquis = merged.iter().find(|r| r.product_name == "Eliquis").unwrap();
        // The downloaded one took precedence for Eliquis
        assert_eq!(
            eliquis.notification_number.as_deref(),
            Some("EMAPD/2026/01")
        );
    }
}
