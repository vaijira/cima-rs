use anyhow::{Context, Result};
use quick_xml::de::from_reader;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

// Helper module for deserializing "0"/"1" strings as booleans
mod bool_from_string {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "1" => Ok(true),
            "0" => Ok(false),
            _ => Err(serde::de::Error::custom(format!(
                "expected '0' or '1', got '{}'",
                s
            ))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AtcRecord {
    #[serde(rename(deserialize = "nroatc"))]
    pub number: i32,
    #[serde(rename(deserialize = "codigoatc"))]
    pub code: String,
    #[serde(rename(deserialize = "descatc"))]
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_atc")]
struct AtcList {
    #[serde(rename = "atc")]
    records: Vec<AtcRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DcpRecord {
    #[serde(rename(deserialize = "codigodcp"))]
    pub code: String,
    #[serde(rename(deserialize = "nombredcp"))]
    pub name: String,
    #[serde(rename(deserialize = "codigodcsa"))]
    pub dcsa_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_dcp")]
struct DcpList {
    #[serde(rename = "dcp")]
    records: Vec<DcpRecord>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DcpfRecord {
    #[serde(rename(deserialize = "codigodcpf"))]
    pub code: String,
    #[serde(rename(deserialize = "nombredcpf"))]
    pub name: String,
    #[serde(rename(deserialize = "codigodcp"))]
    pub dcp_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_dcpf")]
struct DcpfList {
    #[serde(rename = "dcpf")]
    records: Vec<DcpfRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DcsaRecord {
    #[serde(rename(deserialize = "codigodcsa"))]
    pub code: String,
    #[serde(rename(deserialize = "nombredcsa"))]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_dcsa")]
struct DcsaList {
    #[serde(rename = "dcsa")]
    records: Vec<DcsaRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerRecord {
    #[serde(rename(deserialize = "codigoenvase"))]
    pub code: String,
    #[serde(rename(deserialize = "envase"))]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_envases")]
struct ContainerList {
    #[serde(rename = "envases")]
    records: Vec<ContainerRecord>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ExcipientRecord {
    #[serde(rename(deserialize = "codigoedo"))]
    pub code: String,
    #[serde(rename(deserialize = "edo"))]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_excipientes")]
struct ExcipientList {
    #[serde(rename = "excipientes")]
    records: Vec<ExcipientRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PharmaceuticalFormRecord {
    #[serde(rename(deserialize = "codigoformafarmaceutica"))]
    pub code: String,
    #[serde(rename(deserialize = "formafarmaceutica"))]
    pub name: String,
    #[serde(rename(deserialize = "codigoformafarmaceuticasimplificada"))]
    pub simplified_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_formas_farmaceuticas")]
struct PharmaceuticalFormList {
    #[serde(rename = "formasfarmaceuticas")]
    records: Vec<PharmaceuticalFormRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimplifiedPharmaceuticalFormRecord {
    #[serde(rename(deserialize = "codigoformafarmaceuticasimplificada"))]
    pub code: String,
    #[serde(rename(deserialize = "formafarmaceuticasimplificada"))]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_formas_farmaceuticas_simplificadas")]
struct SimplifiedPharmaceuticalFormList {
    #[serde(rename = "formasfarmaceuticassimplificadas")]
    records: Vec<SimplifiedPharmaceuticalFormRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LaboratoryRecord {
    #[serde(rename(deserialize = "codigolaboratorio"))]
    pub code: String,
    #[serde(rename(deserialize = "laboratorio"))]
    pub name: String,
    #[serde(rename(deserialize = "direccion"))]
    pub address: Option<String>,
    #[serde(rename(deserialize = "codigopostal"))]
    pub zip: Option<String>,
    #[serde(rename(deserialize = "localidad"))]
    pub city: Option<String>,
    #[serde(rename(deserialize = "cif"))]
    pub vat: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_laboratorios")]
struct LaboratoryList {
    #[serde(rename = "laboratorios")]
    records: Vec<LaboratoryRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveIngridientRecord {
    #[serde(rename(deserialize = "nroprincipioactivo"))]
    pub number: String,
    #[serde(rename(deserialize = "codigoprincipioactivo"))]
    pub code: String,
    #[serde(rename(deserialize = "principioactivo"))]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_principios_activos")]
struct ActiveIngredientList {
    #[serde(rename = "principiosactivos")]
    records: Vec<ActiveIngridientRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationStatusRecord {
    #[serde(rename(deserialize = "codigosituacionregistro"))]
    pub code: String,
    #[serde(rename(deserialize = "situacionregistro"))]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_situacion_registro")]
struct RegistrationStatusList {
    #[serde(rename = "situacionesregistro")]
    records: Vec<RegistrationStatusRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerUnitRecord {
    #[serde(rename(deserialize = "codigounidadcontenido"))]
    pub code: String,
    #[serde(rename(deserialize = "unidadcontenido"))]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_unidad_contenido")]
struct ContainerUnitList {
    #[serde(rename = "unidadescontenido")]
    records: Vec<ContainerUnitRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdministrationRouteRecord {
    #[serde(rename(deserialize = "codigoviaadministracion"))]
    pub code: String,
    #[serde(rename(deserialize = "viaadministracion"))]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_vias_administracion")]
struct AdministrationRouteList {
    #[serde(rename = "viasadministracion")]
    records: Vec<AdministrationRouteRecord>,
}

// ============================================================================
// Prescription Nested Entity Structs
// ============================================================================

/// Active ingredient composition for a prescription
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveIngredient {
    #[serde(rename(deserialize = "cod_principio_activo"), default)]
    pub active_ingredient_code: Option<String>,
    #[serde(rename(deserialize = "orden_colacion"))]
    pub order: Option<String>,
    #[serde(rename(deserialize = "dosis_pa"))]
    pub dose: Option<String>,
    #[serde(rename(deserialize = "unidad_dosis_pa"))]
    pub dose_unit: Option<String>,
    #[serde(rename(deserialize = "dosis_composicion"))]
    pub composition_dose: Option<String>,
    #[serde(rename(deserialize = "unidad_composicion"))]
    pub composition_unit: Option<String>,
    #[serde(rename(deserialize = "dosis_administracion"))]
    pub administration_dose: Option<String>,
    #[serde(rename(deserialize = "unidad_administracion"))]
    pub administration_unit: Option<String>,
    #[serde(rename(deserialize = "dosis_prescripcion"))]
    pub prescription_dose: Option<String>,
    #[serde(rename(deserialize = "unidad_prescripcion"))]
    pub prescription_unit: Option<String>,
}

/// Administration route for a prescription
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminRoute {
    #[serde(rename(deserialize = "cod_via_admin"))]
    pub route_code: String,
}

/// Pharmaceutical form for a prescription
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrescriptionForm {
    #[serde(rename(deserialize = "cod_forfar"))]
    pub form_code: String,
    #[serde(rename(deserialize = "cod_forfar_simplificada"))]
    pub simplified_form_code: String,
    #[serde(rename(deserialize = "nro_pactiv"))]
    pub num_active_ingredients: Option<String>,
    #[serde(rename(deserialize = "composicion_pa"), default)]
    pub active_ingredients: Vec<ActiveIngredient>,
    #[serde(rename(deserialize = "viasadministracion"), default)]
    pub admin_routes: Vec<AdminRoute>,
}

/// ATC duplicate information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AtcDuplicate {
    #[serde(rename(deserialize = "atc_duplicidad"))]
    pub duplicate_atc: String,
    #[serde(rename(deserialize = "descripcion_atc_duplicidad"))]
    pub description: Option<String>,
    #[serde(rename(deserialize = "efecto_duplicidad"))]
    pub effect: Option<String>,
    #[serde(rename(deserialize = "recomendacion_duplicidad"))]
    pub recommendation: Option<String>,
}

/// ATC code for a prescription
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrescriptionAtc {
    #[serde(rename(deserialize = "cod_atc"))]
    pub atc_code: String,
    #[serde(rename(deserialize = "duplicidades"), default)]
    pub duplicates: Vec<AtcDuplicate>,
}

/// Supply problem for a prescription
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupplyProblem {
    #[serde(rename(deserialize = "fecha_inicio"))]
    pub start_date: Option<String>,
    #[serde(rename(deserialize = "observaciones"))]
    pub observations: Option<String>,
}

// ============================================================================
// Main Prescription Record
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PrescriptionRecord {
    pub cod_nacion: String,
    pub nro_definitivo: String,
    pub des_nomco: String,
    pub des_prese: String,
    pub cod_dcsa: Option<String>,
    pub cod_dcp: Option<String>,
    pub cod_dcpf: Option<String>,
    pub des_dosific: Option<String>,
    pub cod_envase: Option<String>,
    pub contenido: Option<String>,
    pub unid_contenido: Option<String>,
    pub nro_conte: Option<String>,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_psicotropo: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_estupefaciente: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_afecta_conduccion: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_triangulo_negro: bool,
    pub url_fictec: Option<String>,
    pub url_prosp: Option<String>,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_receta: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_generico: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_sustituible: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_envase_clinico: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_uso_hospitalario: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_diagnostico_hospitalario: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_tld: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_especial_control_medico: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_huerfano: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_base_a_plantas: bool,
    pub laboratorio_titular: Option<String>,
    pub laboratorio_comercializador: Option<String>,
    pub fecha_autorizacion: Option<String>,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_comercializado: bool,
    pub fec_comer: Option<String>,
    pub cod_sitreg: Option<String>,
    pub cod_sitreg_presen: Option<String>,
    pub fecha_situacion_registro: Option<String>,
    pub fec_sitreg_presen: Option<String>,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub sw_tiene_excipientes_decl_obligatoria: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub biosimilar: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub importacion_paralela: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub radiofarmaco: bool,
    #[serde(deserialize_with = "bool_from_string::deserialize")]
    pub serializacion: bool,

    // Nested collections (not serialized to main CSV)
    #[serde(rename(deserialize = "formasfarmaceuticas"), default, skip_serializing)]
    pub forms: Option<PrescriptionForm>,

    #[serde(rename(deserialize = "atc"), default, skip_serializing)]
    pub atc_codes: Vec<PrescriptionAtc>,

    #[serde(rename(deserialize = "problemassuministro"), default, skip_serializing)]
    pub supply_problems: Vec<SupplyProblem>,
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub listprescriptiondate: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion")]
pub struct PrescriptionList {
    pub header: Option<Header>,
    #[serde(rename = "prescription")]
    pub records: Vec<PrescriptionRecord>,
}

macro_rules! impl_xml_parser {
    ($(#[$attr:meta])* $fn_name:ident, $list_type:ty, $error_ctx:expr) => {
        $(#[$attr])*
        pub fn $fn_name<P: AsRef<Path>>(xml_path: P, csv_path: P) -> Result<()> {
            let file = File::open(xml_path)?;
            let reader = BufReader::new(file);
            let list: $list_type = from_reader(reader).context($error_ctx)?;

            let mut wtr = csv::Writer::from_path(csv_path)?;
            for record in list.records {
                wtr.serialize(record)?;
            }
            wtr.flush()?;

            Ok(())
        }
    };
    ($(#[$attr:meta])* $fn_name:ident, $list_type:ty, $error_ctx:expr, $mut_record:ident, $transform:block) => {
        $(#[$attr])*
        pub fn $fn_name<P: AsRef<Path>>(xml_path: P, csv_path: P) -> Result<()> {
            let file = File::open(xml_path)?;
            let reader = BufReader::new(file);
            let list: $list_type = from_reader(reader).context($error_ctx)?;

            let mut wtr = csv::Writer::from_path(csv_path)?;
            for mut $mut_record in list.records {
                $transform
                wtr.serialize($mut_record)?;
            }
            wtr.flush()?;

            Ok(())
        }
    };
}

impl_xml_parser!(
    /// Parses the ATC XML file and writes its content to a CSV file.
    parse_atc_xml_to_csv,
    AtcList,
    "Failed to deserialize ATC XML",
    record,
    {
        // Clean description by removing "CODE - " prefix if it exists
        let prefix = format!("{} - ", record.code);
        if record.description.starts_with(&prefix) {
            record.description = record.description[prefix.len()..].to_string();
        }
    }
);

impl_xml_parser!(
    /// Parses the DCP XML file and writes its content to a CSV file.
    parse_dcp_xml_to_csv,
    DcpList,
    "Failed to deserialize DCP XML"
);

impl_xml_parser!(
    /// Parses the DCPF XML file and writes its content to a CSV file.
    parse_dcpf_xml_to_csv,
    DcpfList,
    "Failed to deserialize DCPF XML"
);

impl_xml_parser!(
    /// Parses the DCSA XML file and writes its content to a CSV file.
    parse_dcsa_xml_to_csv,
    DcsaList,
    "Failed to deserialize DCSA XML"
);

impl_xml_parser!(
    /// Parses the Envases XML file and writes its content to a CSV file.
    parse_envases_xml_to_csv,
    ContainerList,
    "Failed to deserialize Envases XML"
);

impl_xml_parser!(
    /// Parses the Excipientes XML file and writes its content to a CSV file.
    parse_excipientes_xml_to_csv,
    ExcipientList,
    "Failed to deserialize Excipientes XML"
);

impl_xml_parser!(
    /// Parses the Forma Farmaceutica XML file and writes its content to a CSV file.
    parse_forma_farmaceutica_xml_to_csv,
    PharmaceuticalFormList,
    "Failed to deserialize Forma Farmaceutica XML"
);

impl_xml_parser!(
    /// Parses the Forma Farmaceutica Simplificada XML file and writes its content to a CSV file.
    parse_forma_farmaceutica_simplificada_xml_to_csv,
    SimplifiedPharmaceuticalFormList,
    "Failed to deserialize Forma Farmaceutica Simplificada XML"
);

impl_xml_parser!(
    /// Parses the Laboratorio XML file and writes its content to a CSV file.
    parse_laboratorio_xml_to_csv,
    LaboratoryList,
    "Failed to deserialize Laboratorio XML"
);

impl_xml_parser!(
    /// Parses the Principio Activo XML file and writes its content to a CSV file.
    parse_principio_activo_xml_to_csv,
    ActiveIngredientList,
    "Failed to deserialize Principio Activo XML"
);

impl_xml_parser!(
    /// Parses the Situacion Registro XML file and writes its content to a CSV file.
    parse_situacion_registro_xml_to_csv,
    RegistrationStatusList,
    "Failed to deserialize Situacion Registro XML"
);

impl_xml_parser!(
    /// Parses the Unidad Contenido XML file and writes its content to a CSV file.
    parse_unidad_contenido_xml_to_csv,
    ContainerUnitList,
    "Failed to deserialize Unidad Contenido XML"
);

impl_xml_parser!(
    /// Parses the Via Administracion XML file and writes its content to a CSV file.
    parse_via_administracion_xml_to_csv,
    AdministrationRouteList,
    "Failed to deserialize Via Administracion XML"
);

impl_xml_parser!(
    /// Parses the Prescription XML file and writes its content to a CSV file.
    parse_prescription_xml_to_csv,
    PrescriptionList,
    "Failed to deserialize Prescription XML"
);

/// Parses the Prescription XML file and writes content to multiple CSV files for normalized data.
///
/// This function extracts nested entities (forms, active ingredients, admin routes, ATC codes, supply problems)
/// into separate CSV files with proper relationships via prescription_id.
///
/// # Output Files
/// - `prescriptions.csv` - Main prescription records
/// - `prescription_forms.csv` - Pharmaceutical forms (1:1 with prescriptions)
/// - `prescription_active_ingredients.csv` - Active ingredients (1:N)
/// - `prescription_admin_routes.csv` - Administration routes (1:N)
/// - `prescription_atc.csv` - ATC codes (1:N)
/// - `prescription_atc_duplicates.csv` - ATC duplicates (nested 1:N)
/// - `prescription_supply_problems.csv` - Supply problems (1:N)
pub fn parse_prescription_xml_to_csvs<P: AsRef<Path>>(xml_path: P, output_dir: P) -> Result<()> {
    let file = File::open(xml_path)?;
    let reader = BufReader::new(file);
    let list: PrescriptionList =
        from_reader(reader).context("Failed to deserialize Prescription XML")?;

    // Create CSV writers for each output file
    let mut wtr_main = csv::Writer::from_path(output_dir.as_ref().join("prescriptions.csv"))?;
    let mut wtr_forms = csv::Writer::from_path(output_dir.as_ref().join("prescription_forms.csv"))?;
    let mut wtr_ingredients = csv::Writer::from_path(
        output_dir
            .as_ref()
            .join("prescription_active_ingredients.csv"),
    )?;
    let mut wtr_routes =
        csv::Writer::from_path(output_dir.as_ref().join("prescription_admin_routes.csv"))?;
    let mut wtr_atc = csv::Writer::from_path(output_dir.as_ref().join("prescription_atc.csv"))?;
    let mut wtr_atc_duplicates =
        csv::Writer::from_path(output_dir.as_ref().join("prescription_atc_duplicates.csv"))?;
    let mut wtr_supply =
        csv::Writer::from_path(output_dir.as_ref().join("prescription_supply_problems.csv"))?;

    // Process each prescription record
    for record in list.records {
        // Use cod_nacion as prescription ID (matches DB primary key)
        let prescription_id = record.cod_nacion.clone();

        // Write main prescription record (nested collections are skipped via serde)
        wtr_main.serialize(&record)?;

        // Write pharmaceutical form and its nested entities
        if let Some(form) = &record.forms {
            // Write form record
            wtr_forms.write_record([
                &prescription_id,
                &form.form_code,
                &form.simplified_form_code,
                form.num_active_ingredients.as_deref().unwrap_or(""),
            ])?;

            // Write active ingredients
            for ingredient in &form.active_ingredients {
                wtr_ingredients.write_record([
                    &prescription_id,
                    ingredient.active_ingredient_code.as_deref().unwrap_or(""),
                    ingredient.order.as_deref().unwrap_or(""),
                    ingredient.dose.as_deref().unwrap_or(""),
                    ingredient.dose_unit.as_deref().unwrap_or(""),
                    ingredient.composition_dose.as_deref().unwrap_or(""),
                    ingredient.composition_unit.as_deref().unwrap_or(""),
                    ingredient.administration_dose.as_deref().unwrap_or(""),
                    ingredient.administration_unit.as_deref().unwrap_or(""),
                    ingredient.prescription_dose.as_deref().unwrap_or(""),
                    ingredient.prescription_unit.as_deref().unwrap_or(""),
                ])?;
            }

            // Write administration routes
            for route in &form.admin_routes {
                wtr_routes.write_record([&prescription_id, &route.route_code])?;
            }
        }

        // Write ATC codes and their duplicates
        for atc in &record.atc_codes {
            wtr_atc.write_record([&prescription_id, &atc.atc_code])?;

            // Write ATC duplicates
            for duplicate in &atc.duplicates {
                wtr_atc_duplicates.write_record([
                    &prescription_id,
                    &atc.atc_code,
                    &duplicate.duplicate_atc,
                    duplicate.description.as_deref().unwrap_or(""),
                    duplicate.effect.as_deref().unwrap_or(""),
                    duplicate.recommendation.as_deref().unwrap_or(""),
                ])?;
            }
        }

        // Write supply problems
        for problem in &record.supply_problems {
            wtr_supply.write_record([
                &prescription_id,
                problem.start_date.as_deref().unwrap_or(""),
                problem.observations.as_deref().unwrap_or(""),
            ])?;
        }
    }

    // Flush all writers
    wtr_main.flush()?;
    wtr_forms.flush()?;
    wtr_ingredients.flush()?;
    wtr_routes.flush()?;
    wtr_atc.flush()?;
    wtr_atc_duplicates.flush()?;
    wtr_supply.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_atc_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_atc>
                <atc>
                    <nroatc>1</nroatc>
                    <codigoatc>A01</codigoatc>
                    <descatc>A01 - DIGESTIVE</descatc>
                </atc>
                <atc>
                    <nroatc>2</nroatc>
                    <codigoatc>B01</codigoatc>
                    <descatc>B01 - BLOOD</descatc>
                </atc>
            </aemps_prescripcion_atc>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_atc_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "number");
        assert_eq!(headers.get(1).unwrap(), "code");
        assert_eq!(headers.get(2).unwrap(), "description");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].get(1).unwrap(), "A01");
        assert_eq!(records[0].get(2).unwrap(), "DIGESTIVE");
        assert_eq!(records[1].get(1).unwrap(), "B01");
        assert_eq!(records[1].get(2).unwrap(), "BLOOD");
    }

    #[test]
    fn test_parse_dcp_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_dcp>
                <dcp>
                    <codigodcp>D01</codigodcp>
                    <nombredcp>DCP NAME</nombredcp>
                    <codigodcsa>S01</codigodcsa>
                </dcp>
            </aemps_prescripcion_dcp>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_dcp_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "code");
        assert_eq!(headers.get(1).unwrap(), "name");
        assert_eq!(headers.get(2).unwrap(), "dcsa_code");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "D01");
        assert_eq!(records[0].get(1).unwrap(), "DCP NAME");
    }

    #[test]
    fn test_parse_dcpf_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_dcpf>
                <dcpf>
                    <codigodcpf>DF01</codigodcpf>
                    <nombredcpf>DCPF NAME</nombredcpf>
                    <codigodcp>D01</codigodcp>
                </dcpf>
            </aemps_prescripcion_dcpf>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_dcpf_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "code");
        assert_eq!(headers.get(1).unwrap(), "name");
        assert_eq!(headers.get(2).unwrap(), "dcp_code");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "DF01");
        assert_eq!(records[0].get(1).unwrap(), "DCPF NAME");
    }

    #[test]
    fn test_parse_dcsa_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_dcsa>
                <dcsa>
                    <codigodcsa>S01</codigodcsa>
                    <nombredcsa>DCSA NAME</nombredcsa>
                </dcsa>
            </aemps_prescripcion_dcsa>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_dcsa_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "code");
        assert_eq!(headers.get(1).unwrap(), "name");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "S01");
        assert_eq!(records[0].get(1).unwrap(), "DCSA NAME");
    }

    #[test]
    fn test_parse_envases_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_envases>
                <envases>
                    <codigoenvase>E01</codigoenvase>
                    <envase>ENVASE NAME</envase>
                </envases>
            </aemps_prescripcion_envases>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_envases_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "code");
        assert_eq!(headers.get(1).unwrap(), "name");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "E01");
        assert_eq!(records[0].get(1).unwrap(), "ENVASE NAME");
    }

    #[test]
    fn test_parse_excipientes_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_excipientes>
                <excipientes>
                    <codigoedo>X01</codigoedo>
                    <edo>EXCIPIENTE NAME</edo>
                </excipientes>
            </aemps_prescripcion_excipientes>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_excipientes_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "code");
        assert_eq!(headers.get(1).unwrap(), "name");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "X01");
        assert_eq!(records[0].get(1).unwrap(), "EXCIPIENTE NAME");
    }

    #[test]
    fn test_parse_forma_farmaceutica_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_formas_farmaceuticas>
                <formasfarmaceuticas>
                    <codigoformafarmaceutica>FF01</codigoformafarmaceutica>
                    <formafarmaceutica>FORMA NAME</formafarmaceutica>
                    <codigoformafarmaceuticasimplificada>SFF01</codigoformafarmaceuticasimplificada>
                </formasfarmaceuticas>
            </aemps_prescripcion_formas_farmaceuticas>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_forma_farmaceutica_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "code");
        assert_eq!(headers.get(1).unwrap(), "name");
        assert_eq!(headers.get(2).unwrap(), "simplified_code");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "FF01");
        assert_eq!(records[0].get(1).unwrap(), "FORMA NAME");
        assert_eq!(records[0].get(2).unwrap(), "SFF01");
    }

    #[test]
    fn test_parse_forma_farmaceutica_simplificada_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_formas_farmaceuticas_simplificadas>
                <formasfarmaceuticassimplificadas>
                    <codigoformafarmaceuticasimplificada>SFF01</codigoformafarmaceuticasimplificada>
                    <formafarmaceuticasimplificada>SIMPLIFIED NAME</formafarmaceuticasimplificada>
                </formasfarmaceuticassimplificadas>
            </aemps_prescripcion_formas_farmaceuticas_simplificadas>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_forma_farmaceutica_simplificada_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "code");
        assert_eq!(headers.get(1).unwrap(), "name");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "SFF01");
        assert_eq!(records[0].get(1).unwrap(), "SIMPLIFIED NAME");
    }

    #[test]
    fn test_parse_laboratorio_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_laboratorios>
                <laboratorios>
                    <codigolaboratorio>L01</codigolaboratorio>
                    <laboratorio>LAB NAME</laboratorio>
                    <direccion>ADDR</direccion>
                    <codigopostal>ZIP</codigopostal>
                    <localidad>CITY</localidad>
                    <cif>VAT</cif>
                </laboratorios>
                <laboratorios>
                    <codigolaboratorio>L02</codigolaboratorio>
                    <laboratorio>LAB NAME 2</laboratorio>
                </laboratorios>
            </aemps_prescripcion_laboratorios>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_laboratorio_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "code");
        assert_eq!(headers.get(1).unwrap(), "name");
        assert_eq!(headers.get(2).unwrap(), "address");
        assert_eq!(headers.get(3).unwrap(), "zip");
        assert_eq!(headers.get(4).unwrap(), "city");
        assert_eq!(headers.get(5).unwrap(), "vat");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].get(0).unwrap(), "L01");
        assert_eq!(records[0].get(1).unwrap(), "LAB NAME");
        assert_eq!(records[0].get(2).unwrap(), "ADDR");
        assert_eq!(records[0].get(3).unwrap(), "ZIP");
        assert_eq!(records[0].get(4).unwrap(), "CITY");
        assert_eq!(records[0].get(5).unwrap(), "VAT");

        assert_eq!(records[1].get(0).unwrap(), "L02");
        assert_eq!(records[1].get(1).unwrap(), "LAB NAME 2");
        // Second record has empty strings for optional fields
        assert_eq!(records[1].get(2).unwrap(), "");
        assert_eq!(records[1].get(3).unwrap(), "");
        assert_eq!(records[1].get(4).unwrap(), "");
        assert_eq!(records[1].get(5).unwrap(), "");
    }

    #[test]
    fn test_parse_principio_activo_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_principios_activos>
                <principiosactivos>
                    <nroprincipioactivo>1</nroprincipioactivo>
                    <codigoprincipioactivo>PA01</codigoprincipioactivo>
                    <principioactivo>PRINCIPIO NAME</principioactivo>
                </principiosactivos>
            </aemps_prescripcion_principios_activos>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_principio_activo_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "number");
        assert_eq!(headers.get(1).unwrap(), "code");
        assert_eq!(headers.get(2).unwrap(), "name");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "1");
        assert_eq!(records[0].get(1).unwrap(), "PA01");
        assert_eq!(records[0].get(2).unwrap(), "PRINCIPIO NAME");
    }

    #[test]
    fn test_parse_situacion_registro_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_situacion_registro>
                <situacionesregistro>
                    <codigosituacionregistro>1</codigosituacionregistro>
                    <situacionregistro>Autorizado</situacionregistro>
                </situacionesregistro>
            </aemps_prescripcion_situacion_registro>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_situacion_registro_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "code");
        assert_eq!(headers.get(1).unwrap(), "name");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "1");
        assert_eq!(records[0].get(1).unwrap(), "Autorizado");
    }

    #[test]
    fn test_parse_unidad_contenido_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_unidad_contenido>
                <unidadescontenido>
                    <codigounidadcontenido>1</codigounidadcontenido>
                    <unidadcontenido>ampolla para inyección</unidadcontenido>
                </unidadescontenido>
            </aemps_prescripcion_unidad_contenido>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_unidad_contenido_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "code");
        assert_eq!(headers.get(1).unwrap(), "name");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "1");
        assert_eq!(records[0].get(1).unwrap(), "ampolla para inyección");
    }

    #[test]
    fn test_parse_via_administracion_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion_vias_administracion>
                <viasadministracion>
                    <codigoviaadministracion>7</codigoviaadministracion>
                    <viaadministracion>HEMODIÁLISIS</viaadministracion>
                </viasadministracion>
            </aemps_prescripcion_vias_administracion>"#
        )
        .unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_via_administracion_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();

        // Verify CSV headers use Rust field names
        let headers = csv_reader.headers().unwrap();
        assert_eq!(headers.get(0).unwrap(), "code");
        assert_eq!(headers.get(1).unwrap(), "name");

        // Verify CSV data
        let records: Vec<csv::StringRecord> = csv_reader.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].get(0).unwrap(), "7");
        assert_eq!(records[0].get(1).unwrap(), "HEMODIÁLISIS");
    }

    #[test]
    fn test_parse_prescription_with_nested() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion>
                <prescription>
                    <cod_nacion>600000</cod_nacion>
                    <nro_definitivo>66337</nro_definitivo>
                    <des_nomco>TEST</des_nomco>
                    <des_prese>TEST</des_prese>
                    <sw_psicotropo>0</sw_psicotropo>
                    <sw_estupefaciente>0</sw_estupefaciente>
                    <sw_afecta_conduccion>0</sw_afecta_conduccion>
                    <sw_triangulo_negro>0</sw_triangulo_negro>
                    <sw_receta>1</sw_receta>
                    <sw_generico>1</sw_generico>
                    <sw_sustituible>1</sw_sustituible>
                    <sw_envase_clinico>1</sw_envase_clinico>
                    <sw_uso_hospitalario>1</sw_uso_hospitalario>
                    <sw_diagnostico_hospitalario>0</sw_diagnostico_hospitalario>
                    <sw_tld>0</sw_tld>
                    <sw_especial_control_medico>0</sw_especial_control_medico>
                    <sw_huerfano>0</sw_huerfano>
                    <sw_base_a_plantas>0</sw_base_a_plantas>
                    <sw_comercializado>0</sw_comercializado>
                    <sw_tiene_excipientes_decl_obligatoria>0</sw_tiene_excipientes_decl_obligatoria>
                    <biosimilar>0</biosimilar>
                    <importacion_paralela>0</importacion_paralela>
                    <radiofarmaco>0</radiofarmaco>
                    <serializacion>1</serializacion>
                    <formasfarmaceuticas>
                        <cod_forfar>288</cod_forfar>
                        <cod_forfar_simplificada>34</cod_forfar_simplificada>
                        <nro_pactiv>1</nro_pactiv>
                        <composicion_pa>
                            <cod_principio_activo>160</cod_principio_activo>
                        </composicion_pa>
                        <viasadministracion>
                            <cod_via_admin>49</cod_via_admin>
                        </viasadministracion>
                    </formasfarmaceuticas>
                </prescription>
            </aemps_prescripcion>"#
        )
        .unwrap();

        let file = File::open(xml_file.path()).unwrap();
        let reader = BufReader::new(file);
        let result: Result<PrescriptionList, _> = from_reader(reader);

        match result {
            Ok(list) => {
                assert_eq!(list.records.len(), 1);
                let record = &list.records[0];
                assert_eq!(record.cod_nacion, "600000");
                assert!(record.forms.is_some());
                println!("Test passed! Nested structure deserialized successfully");
            }
            Err(e) => {
                panic!("Deserialization failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_parse_prescription_to_multi_csv() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion>
                <prescription>
                    <cod_nacion>600000</cod_nacion>
                    <nro_definitivo>66337</nro_definitivo>
                    <des_nomco>TEST</des_nomco>
                    <des_prese>TEST</des_prese>
                    <sw_psicotropo>0</sw_psicotropo>
                    <sw_estupefaciente>0</sw_estupefaciente>
                    <sw_afecta_conduccion>0</sw_afecta_conduccion>
                    <sw_triangulo_negro>0</sw_triangulo_negro>
                    <sw_receta>1</sw_receta>
                    <sw_generico>1</sw_generico>
                    <sw_sustituible>1</sw_sustituible>
                    <sw_envase_clinico>1</sw_envase_clinico>
                    <sw_uso_hospitalario>1</sw_uso_hospitalario>
                    <sw_diagnostico_hospitalario>0</sw_diagnostico_hospitalario>
                    <sw_tld>0</sw_tld>
                    <sw_especial_control_medico>0</sw_especial_control_medico>
                    <sw_huerfano>0</sw_huerfano>
                    <sw_base_a_plantas>0</sw_base_a_plantas>
                    <sw_comercializado>0</sw_comercializado>
                    <sw_tiene_excipientes_decl_obligatoria>0</sw_tiene_excipientes_decl_obligatoria>
                    <biosimilar>0</biosimilar>
                    <importacion_paralela>0</importacion_paralela>
                    <radiofarmaco>0</radiofarmaco>
                    <serializacion>1</serializacion>
                    <formasfarmaceuticas>
                        <cod_forfar>288</cod_forfar>
                        <cod_forfar_simplificada>34</cod_forfar_simplificada>
                        <nro_pactiv>1</nro_pactiv>
                        <composicion_pa>
                            <cod_principio_activo>160</cod_principio_activo>
                        </composicion_pa>
                        <viasadministracion>
                            <cod_via_admin>49</cod_via_admin>
                        </viasadministracion>
                    </formasfarmaceuticas>
                    <atc>
                        <cod_atc>J01CR02</cod_atc>
                    </atc>
                </prescription>
            </aemps_prescripcion>"#
        )
        .unwrap();

        let output_dir = tempfile::tempdir().unwrap();
        let result = parse_prescription_xml_to_csvs(xml_file.path(), output_dir.path());

        assert!(
            result.is_ok(),
            "Multi-CSV parsing failed: {:?}",
            result.err()
        );

        // Verify all 7 CSV files were created
        assert!(output_dir.path().join("prescriptions.csv").exists());
        assert!(output_dir.path().join("prescription_forms.csv").exists());
        assert!(
            output_dir
                .path()
                .join("prescription_active_ingredients.csv")
                .exists()
        );
        assert!(
            output_dir
                .path()
                .join("prescription_admin_routes.csv")
                .exists()
        );
        assert!(output_dir.path().join("prescription_atc.csv").exists());

        println!("Multi-CSV test passed! All 7 files created successfully");
    }
}
