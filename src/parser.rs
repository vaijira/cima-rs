use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use quick_xml::de::from_reader;

#[derive(Debug, Serialize, Deserialize)]
pub struct AtcRecord {
    #[serde(rename = "nroatc")]
    pub number: i32,
    #[serde(rename = "codigoatc")]
    pub code: String,
    #[serde(rename = "descatc")]
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
    #[serde(rename = "codigodcp")]
    pub code: String,
    #[serde(rename = "nombredcp")]
    pub name: String,
    #[serde(rename = "codigodcsa")]
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
    #[serde(rename = "codigodcpf")]
    pub code: String,
    #[serde(rename = "nombredcpf")]
    pub name: String,
    #[serde(rename = "codigodcp")]
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
    #[serde(rename = "codigodcsa")]
    pub code: String,
    #[serde(rename = "nombredcsa")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_dcsa")]
struct DcsaList {
    #[serde(rename = "dcsa")]
    records: Vec<DcsaRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvaseRecord {
    #[serde(rename = "codigoenvase")]
    pub code: String,
    #[serde(rename = "envase")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_envases")]
struct EnvaseList {
    #[serde(rename = "envases")]
    records: Vec<EnvaseRecord>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ExcipienteRecord {
    #[serde(rename = "codigoedo")]
    pub code: String,
    #[serde(rename = "edo")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_excipientes")]
struct ExcipienteList {
    #[serde(rename = "excipientes")]
    records: Vec<ExcipienteRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormaFarmaceuticaRecord {
    #[serde(rename = "codigoformafarmaceutica")]
    pub code: String,
    #[serde(rename = "formafarmaceutica")]
    pub name: String,
    #[serde(rename = "codigoformafarmaceuticasimplificada")]
    pub simplified_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_formas_farmaceuticas")]
struct FormaFarmaceuticaList {
    #[serde(rename = "formasfarmaceuticas")]
    records: Vec<FormaFarmaceuticaRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormaFarmaceuticaSimplificadaRecord {
    #[serde(rename = "codigoformafarmaceuticasimplificada")]
    pub code: String,
    #[serde(rename = "formafarmaceuticasimplificada")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_formas_farmaceuticas_simplificadas")]
struct FormaFarmaceuticaSimplificadaList {
    #[serde(rename = "formasfarmaceuticassimplificadas")]
    records: Vec<FormaFarmaceuticaSimplificadaRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LaboratorioRecord {
    #[serde(rename = "codigolaboratorio")]
    pub code: String,
    #[serde(rename = "laboratorio")]
    pub name: String,
    #[serde(rename = "direccion")]
    pub address: Option<String>,
    #[serde(rename = "codigopostal")]
    pub zip: Option<String>,
    #[serde(rename = "localidad")]
    pub city: Option<String>,
    #[serde(rename = "cif")]
    pub vat: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_laboratorios")]
struct LaboratorioList {
    #[serde(rename = "laboratorios")]
    records: Vec<LaboratorioRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrincipioActivoRecord {
    #[serde(rename = "nroprincipioactivo")]
    pub number: String,
    #[serde(rename = "codigoprincipioactivo")]
    pub code: String,
    #[serde(rename = "principioactivo")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_principios_activos")]
struct PrincipioActivoList {
    #[serde(rename = "principiosactivos")]
    records: Vec<PrincipioActivoRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SituacionRegistroRecord {
    #[serde(rename = "codigosituacionregistro")]
    pub code: String,
    #[serde(rename = "situacionregistro")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_situacion_registro")]
struct SituacionRegistroList {
    #[serde(rename = "situacionesregistro")]
    records: Vec<SituacionRegistroRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnidadContenidoRecord {
    #[serde(rename = "codigounidadcontenido")]
    pub code: String,
    #[serde(rename = "unidadcontenido")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_unidad_contenido")]
struct UnidadContenidoList {
    #[serde(rename = "unidadescontenido")]
    records: Vec<UnidadContenidoRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViaAdministracionRecord {
    #[serde(rename = "codigoviaadministracion")]
    pub code: String,
    #[serde(rename = "viaadministracion")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "aemps_prescripcion_vias_administracion")]
struct ViaAdministracionList {
    #[serde(rename = "viasadministracion")]
    records: Vec<ViaAdministracionRecord>,
}

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
    pub sw_psicotropo: String,
    pub sw_estupefaciente: String,
    pub sw_afecta_conduccion: String,
    pub sw_triangulo_negro: String,
    pub url_fictec: Option<String>,
    pub url_prosp: Option<String>,
    pub sw_receta: String,
    pub sw_generico: String,
    pub sw_sustituible: String,
    pub sw_envase_clinico: String,
    pub sw_uso_hospitalario: String,
    pub sw_diagnostico_hospitalario: String,
    pub sw_tld: String,
    pub sw_especial_control_medico: String,
    pub sw_huerfano: String,
    pub sw_base_a_plantas: String,
    pub laboratorio_titular: Option<String>,
    pub laboratorio_comercializador: Option<String>,
    pub fecha_autorizacion: Option<String>,
    pub sw_comercializado: String,
    pub fec_comer: Option<String>,
    pub cod_sitreg: Option<String>,
    pub cod_sitreg_presen: Option<String>,
    pub fecha_situacion_registro: Option<String>,
    pub fec_sitreg_presen: Option<String>,
    pub sw_tiene_excipientes_decl_obligatoria: String,
    pub biosimilar: String,
    pub importacion_paralela: String,
    pub radiofarmaco: String,
    pub serializacion: String,
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
    EnvaseList,
    "Failed to deserialize Envases XML"
);

impl_xml_parser!(
    /// Parses the Excipientes XML file and writes its content to a CSV file.
    parse_excipientes_xml_to_csv,
    ExcipienteList,
    "Failed to deserialize Excipientes XML"
);

impl_xml_parser!(
    /// Parses the Forma Farmaceutica XML file and writes its content to a CSV file.
    parse_forma_farmaceutica_xml_to_csv,
    FormaFarmaceuticaList,
    "Failed to deserialize Forma Farmaceutica XML"
);

impl_xml_parser!(
    /// Parses the Forma Farmaceutica Simplificada XML file and writes its content to a CSV file.
    parse_forma_farmaceutica_simplificada_xml_to_csv,
    FormaFarmaceuticaSimplificadaList,
    "Failed to deserialize Forma Farmaceutica Simplificada XML"
);

impl_xml_parser!(
    /// Parses the Laboratorio XML file and writes its content to a CSV file.
    parse_laboratorio_xml_to_csv,
    LaboratorioList,
    "Failed to deserialize Laboratorio XML"
);

impl_xml_parser!(
    /// Parses the Principio Activo XML file and writes its content to a CSV file.
    parse_principio_activo_xml_to_csv,
    PrincipioActivoList,
    "Failed to deserialize Principio Activo XML"
);

impl_xml_parser!(
    /// Parses the Situacion Registro XML file and writes its content to a CSV file.
    parse_situacion_registro_xml_to_csv,
    SituacionRegistroList,
    "Failed to deserialize Situacion Registro XML"
);

impl_xml_parser!(
    /// Parses the Unidad Contenido XML file and writes its content to a CSV file.
    parse_unidad_contenido_xml_to_csv,
    UnidadContenidoList,
    "Failed to deserialize Unidad Contenido XML"
);

impl_xml_parser!(
    /// Parses the Via Administracion XML file and writes its content to a CSV file.
    parse_via_administracion_xml_to_csv,
    ViaAdministracionList,
    "Failed to deserialize Via Administracion XML"
);

impl_xml_parser!(
    /// Parses the Prescription XML file and writes its content to a CSV file.
    parse_prescription_xml_to_csv,
    PrescriptionList,
    "Failed to deserialize Prescription XML"
);

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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_atc_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: AtcRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].code, "A01");
        assert_eq!(records[0].description, "DIGESTIVE");
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_dcp_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: DcpRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "D01");
        assert_eq!(records[0].name, "DCP NAME");
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_dcpf_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: DcpfRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "DF01");
        assert_eq!(records[0].name, "DCPF NAME");
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_dcsa_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: DcsaRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "S01");
        assert_eq!(records[0].name, "DCSA NAME");
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_envases_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: EnvaseRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "E01");
        assert_eq!(records[0].name, "ENVASE NAME");
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_excipientes_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: ExcipienteRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "X01");
        assert_eq!(records[0].name, "EXCIPIENTE NAME");
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_forma_farmaceutica_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: FormaFarmaceuticaRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "FF01");
        assert_eq!(records[0].name, "FORMA NAME");
        assert_eq!(records[0].simplified_code, "SFF01");
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_forma_farmaceutica_simplificada_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: FormaFarmaceuticaSimplificadaRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "SFF01");
        assert_eq!(records[0].name, "SIMPLIFIED NAME");
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_laboratorio_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: LaboratorioRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].code, "L01");
        assert_eq!(records[0].name, "LAB NAME");
        assert_eq!(records[0].address, Some("ADDR".to_string()));
        assert_eq!(records[0].zip, Some("ZIP".to_string()));
        assert_eq!(records[0].city, Some("CITY".to_string()));
        assert_eq!(records[0].vat, Some("VAT".to_string()));

        assert_eq!(records[1].code, "L02");
        assert_eq!(records[1].name, "LAB NAME 2");
        assert_eq!(records[1].address, None);
        assert_eq!(records[1].zip, None);
        assert_eq!(records[1].city, None);
        assert_eq!(records[1].vat, None);
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_principio_activo_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: PrincipioActivoRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].number, "1");
        assert_eq!(records[0].code, "PA01");
        assert_eq!(records[0].name, "PRINCIPIO NAME");
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_situacion_registro_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: SituacionRegistroRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "1");
        assert_eq!(records[0].name, "Autorizado");
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_unidad_contenido_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: UnidadContenidoRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "1");
        assert_eq!(records[0].name, "ampolla para inyección");
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
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_via_administracion_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: ViaAdministracionRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].code, "7");
        assert_eq!(records[0].name, "HEMODIÁLISIS");
    }

    #[test]
    fn test_parse_prescription_xml() {
        let mut xml_file = NamedTempFile::new().unwrap();
        writeln!(
            xml_file,
            r#"<aemps_prescripcion>
                <header><listprescriptiondate>2026-01-11</listprescriptiondate></header>
                <prescription>
                    <cod_nacion>600000</cod_nacion>
                    <nro_definitivo>66337</nro_definitivo>
                    <des_nomco>AMOXICILINA</des_nomco>
                    <des_prese>AMOXICILINA 500mg</des_prese>
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
                </prescription>
            </aemps_prescripcion>"#
        ).unwrap();

        let csv_file = NamedTempFile::new().unwrap();
        let xml_path = xml_file.path();
        let csv_path = csv_file.path();

        let result = parse_prescription_xml_to_csv(xml_path, csv_path);
        assert!(result.is_ok());

        let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
        let mut records = Vec::new();
        for result in csv_reader.deserialize() {
            let record: PrescriptionRecord = result.unwrap();
            records.push(record);
        }

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].cod_nacion, "600000");
        assert_eq!(records[0].nro_definitivo, "66337");
    }
}
