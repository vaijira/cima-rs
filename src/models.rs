use serde::{Deserialize, Serialize};

/// Wrapper for paginated API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    #[serde(rename = "totalFilas")]
    pub total_rows: u32,
    #[serde(rename = "pagina")]
    pub page: u32,
    #[serde(rename = "tamanioPagina")]
    pub page_size: u32,
    #[serde(rename = "resultados")]
    pub results: Vec<T>,
}

/// Authorization status of a medication or presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationStatus {
    /// Authorization date (Unix Epoch GMT+2:00)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aut: Option<i64>,
    /// Suspension date (Unix Epoch GMT+2:00)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub susp: Option<i64>,
    /// Revocation date (Unix Epoch GMT+2:00)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<i64>,
}

/// Generic item used in master data catalogs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterItem {
    /// Numeric identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Alphanumeric identifier
    #[serde(rename = "codigo", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Name
    #[serde(rename = "nombre")]
    pub name: String,
}

/// Supply problem for a presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyProblem {
    /// National code
    pub cn: String,
    /// Presentation name
    #[serde(rename = "nombre")]
    pub name: String,
    /// Start date (Unix Epoch GMT+2:00)
    pub fini: i64,
    /// Expected end date or resolution date (Unix Epoch GMT+2:00)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ffin: Option<i64>,
    /// Observations
    #[serde(rename = "observ", skip_serializing_if = "Option::is_none")]
    pub observations: Option<String>,
    /// Indicates if still active
    #[serde(rename = "activo")]
    pub active: bool,
}

/// Document section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    /// Section number (up to 3 levels: "N.N.N")
    #[serde(rename = "seccion")]
    pub section: String,
    /// Section title
    #[serde(rename = "titulo")]
    pub title: String,
    /// Section order
    #[serde(rename = "orden")]
    pub order: i32,
    /// Content in HTML format
    #[serde(rename = "contenido", skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Document type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum DocumentType {
    /// Technical data sheet
    #[serde(rename = "FichaTecnica")]
    TechnicalSheet = 1,
    /// Package leaflet
    #[serde(rename = "Prospecto")]
    PackageLeaflet = 2,
    /// Public evaluation report
    #[serde(rename = "InformePublico")]
    PublicReport = 3,
    /// Risk management plan
    #[serde(rename = "PlanGestionRiesgos")]
    RiskManagementPlan = 4,
}

/// Document associated with a medication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Document type
    #[serde(rename = "tipo")]
    pub doc_type: u8,
    /// URL to access the document
    pub url: String,
    /// Indicates if available in HTML sections
    #[serde(rename = "secc")]
    pub has_sections: bool,
    /// URL in HTML format (only if has_sections = true)
    #[serde(rename = "urlHtml", skip_serializing_if = "Option::is_none")]
    pub url_html: Option<String>,
    /// Modification date (Unix Epoch GMT+2:00)
    #[serde(rename = "fecha", skip_serializing_if = "Option::is_none")]
    pub date: Option<i64>,
}

/// Safety or informative note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyNote {
    /// Note type (1: Safety Note)
    #[serde(rename = "tipo")]
    pub note_type: u8,
    /// Note number
    pub num: String,
    /// Associated reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    /// Subject
    #[serde(rename = "asunto")]
    pub subject: String,
    /// Publication date (Unix Epoch GMT+2:00)
    #[serde(rename = "fecha")]
    pub date: i64,
    /// URL to access the note
    pub url: String,
}

/// Informative material document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDocument {
    /// Document title
    #[serde(rename = "nombre")]
    pub name: String,
    /// Access URL
    pub url: String,
    /// Update date (Unix Epoch GMT+2:00)
    #[serde(rename = "fecha")]
    pub date: i64,
}

/// Safety informative material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyMaterial {
    /// List of documents for healthcare professionals
    #[serde(rename = "listaDocsProfesional", default)]
    pub professional_docs: Vec<MaterialDocument>,
}

/// Clinical description (VMP/VMPP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalDescription {
    /// VMP code
    pub vmp: String,
    /// VMP name
    #[serde(rename = "vmpDesc")]
    pub vmp_desc: String,
    /// VMPP code
    pub vmpp: String,
    /// VMPP name
    #[serde(rename = "vmppDesc")]
    pub vmpp_desc: String,
    /// Number of commercialized presentations
    #[serde(rename = "presComerc")]
    pub commercialized_presentations: i32,
}

/// ATC code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtcCode {
    /// ATC code
    #[serde(rename = "codigo")]
    pub code: String,
    /// Descriptive name
    #[serde(rename = "nombre")]
    pub name: String,
    /// ATC code level
    #[serde(rename = "nivel")]
    pub level: i32,
}

/// Active ingredient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveIngredient {
    /// Active ingredient ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Identification code
    #[serde(rename = "codigo", skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Name
    #[serde(rename = "nombre")]
    pub name: String,
    /// Amount
    #[serde(rename = "cantidad", skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    /// Unit
    #[serde(rename = "unidad", skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Display order
    #[serde(rename = "orden", skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,
}

/// Excipient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Excipient {
    /// Excipient ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Name
    #[serde(rename = "nombre")]
    pub name: String,
    /// Amount
    #[serde(rename = "cantidad", skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    /// Unit
    #[serde(rename = "unidad", skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Display order
    #[serde(rename = "orden", skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,
}

/// Medication photo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Photo {
    /// Type: "materialas" (packaging material) or "formafarmac" (pharmaceutical form)
    #[serde(rename = "tipo")]
    pub photo_type: String,
    /// Image URL
    pub url: String,
    /// Update date (Unix Epoch GMT+2:00)
    #[serde(rename = "fecha", skip_serializing_if = "Option::is_none")]
    pub date: Option<i64>,
}

/// Presentation of a medication (simplified view for listings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationSummary {
    /// National code
    pub cn: String,
    /// Presentation name
    #[serde(rename = "nombre")]
    pub name: String,
    /// Registration status
    #[serde(rename = "estado")]
    pub status: AuthorizationStatus,
    /// Indicates if commercialized
    #[serde(rename = "comerc")]
    pub commercialized: bool,
    /// Indicates if has supply problems
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psum: Option<bool>,
}

/// Complete presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    /// National code
    pub cn: String,
    /// Name
    #[serde(rename = "nombre")]
    pub name: String,
    /// Status
    #[serde(rename = "estado")]
    pub status: AuthorizationStatus,
    /// Indicates if commercialized
    #[serde(rename = "comerc")]
    pub commercialized: bool,
    /// Indicates if has supply problems
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psum: Option<bool>,
}

/// Medication (simplified view for listings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicationSummary {
    /// Registration number
    pub nregistro: String,
    /// Medication name
    #[serde(rename = "nombre")]
    pub name: String,
    /// Holder laboratory
    pub labtitular: String,
    /// Registration status
    #[serde(rename = "estado")]
    pub status: AuthorizationStatus,
    /// Prescription conditions
    pub cpresc: String,
    /// Indicates if has any commercialized presentation
    #[serde(rename = "comerc", skip_serializing_if = "Option::is_none")]
    pub commercialized: Option<bool>,
    /// Indicates if requires prescription
    #[serde(rename = "receta", skip_serializing_if = "Option::is_none")]
    pub prescription_required: Option<bool>,
    /// Indicates if affects driving
    #[serde(rename = "conduc", skip_serializing_if = "Option::is_none")]
    pub affects_driving: Option<bool>,
    /// Indicates if has black triangle
    #[serde(rename = "triangulo", skip_serializing_if = "Option::is_none")]
    pub black_triangle: Option<bool>,
    /// Indicates if orphan drug
    #[serde(rename = "huerfano", skip_serializing_if = "Option::is_none")]
    pub orphan: Option<bool>,
    /// Indicates if biosimilar
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biosimilar: Option<bool>,
    /// Non-substitutable type
    #[serde(rename = "nosustituible", skip_serializing_if = "Option::is_none")]
    pub non_substitutable: Option<MasterItem>,
    /// Indicates if has supply problems
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psum: Option<bool>,
    /// Indicates if registered by EMA
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema: Option<bool>,
    /// Indicates if has notes
    #[serde(rename = "notas", skip_serializing_if = "Option::is_none")]
    pub has_notes: Option<bool>,
    /// Indicates if has informative materials
    #[serde(rename = "materialesInf", skip_serializing_if = "Option::is_none")]
    pub has_materials: Option<bool>,
    /// Associated documents
    #[serde(default)]
    pub docs: Vec<Document>,
    /// Associated photos
    #[serde(rename = "fotos", default)]
    pub photos: Vec<Photo>,
    /// Administration routes
    #[serde(rename = "viasAdministracion", default)]
    pub administration_routes: Vec<MasterItem>,
    /// Pharmaceutical form
    #[serde(rename = "formaFarmaceutica", skip_serializing_if = "Option::is_none")]
    pub pharmaceutical_form: Option<MasterItem>,
    /// Simplified pharmaceutical form
    #[serde(
        rename = "formaFarmaceuticaSimplificada",
        skip_serializing_if = "Option::is_none"
    )]
    pub simplified_pharmaceutical_form: Option<MasterItem>,
    /// Active ingredient dose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosis: Option<String>,
}

/// Complete medication with all details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    /// Registration number
    pub nregistro: String,
    /// Medication name
    #[serde(rename = "nombre")]
    pub name: String,
    /// Comma-separated list of active ingredients
    pub pactivos: String,
    /// Holder laboratory
    pub labtitular: String,
    /// Registration status
    #[serde(rename = "estado")]
    pub status: AuthorizationStatus,
    /// Prescription conditions
    pub cpresc: String,
    /// Indicates if has any commercialized presentation
    #[serde(rename = "comerc", skip_serializing_if = "Option::is_none")]
    pub commercialized: Option<bool>,
    /// Indicates if requires prescription
    #[serde(rename = "receta", skip_serializing_if = "Option::is_none")]
    pub prescription_required: Option<bool>,
    /// Indicates if affects driving
    #[serde(rename = "conduc", skip_serializing_if = "Option::is_none")]
    pub affects_driving: Option<bool>,
    /// Indicates if has black triangle
    #[serde(rename = "triangulo", skip_serializing_if = "Option::is_none")]
    pub black_triangle: Option<bool>,
    /// Indicates if orphan drug
    #[serde(rename = "huerfano", skip_serializing_if = "Option::is_none")]
    pub orphan: Option<bool>,
    /// Indicates if biosimilar
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biosimilar: Option<bool>,
    /// Indicates if registered by EMA
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema: Option<bool>,
    /// Indicates if has supply problems
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psum: Option<bool>,
    /// Associated documents
    #[serde(default)]
    pub docs: Vec<Document>,
    /// Associated photos
    #[serde(rename = "fotos", default)]
    pub photos: Vec<Photo>,
    /// Indicates if has notes
    #[serde(rename = "notas", skip_serializing_if = "Option::is_none")]
    pub has_notes: Option<bool>,
    /// Indicates if has informative materials
    #[serde(rename = "materialesInf", skip_serializing_if = "Option::is_none")]
    pub has_materials: Option<bool>,
    /// ATC codes
    #[serde(default)]
    pub atcs: Vec<AtcCode>,
    /// Active ingredients
    #[serde(rename = "principiosActivos", default)]
    pub active_ingredients: Vec<ActiveIngredient>,
    /// Excipients
    #[serde(rename = "excipientes", default)]
    pub excipients: Vec<Excipient>,
    /// Administration routes
    #[serde(rename = "viasAdministracion", default)]
    pub administration_routes: Vec<MasterItem>,
    /// Non-substitutable type
    #[serde(rename = "nosustituible", skip_serializing_if = "Option::is_none")]
    pub non_substitutable: Option<MasterItem>,
    /// Presentations
    #[serde(rename = "presentaciones", default)]
    pub presentations: Vec<PresentationSummary>,
    /// Pharmaceutical form
    #[serde(rename = "formaFarmaceutica", skip_serializing_if = "Option::is_none")]
    pub pharmaceutical_form: Option<MasterItem>,
    /// Simplified pharmaceutical form
    #[serde(
        rename = "formaFarmaceuticaSimplificada",
        skip_serializing_if = "Option::is_none"
    )]
    pub simplified_pharmaceutical_form: Option<MasterItem>,
    /// Active ingredient dose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosis: Option<String>,
}

/// Change log record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRecord {
    /// Medication registration number
    pub nregistro: String,
    /// Change date (Unix Epoch GMT+2:00)
    #[serde(rename = "fecha")]
    pub date: i64,
    /// Change type: 1=New, 2=Deleted, 3=Modified
    #[serde(rename = "tipoCambio")]
    pub change_type: u8,
    /// List of changes: "estado", "comerc", "prosp", "ft", "psum", "notasSeguridad", "matinf", "otros"
    #[serde(rename = "cambios", default)]
    pub changes: Vec<String>,
}

/// Master data type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MasterDataType {
    ActiveIngredients = 1,
    PharmaceuticalForms = 3,
    AdministrationRoutes = 4,
    Laboratories = 6,
    AtcCodes = 7,
    ActiveIngredientsSNOMED = 11,
    SimplifiedPharmaceuticalFormsSNOMED = 13,
    AdministrationRoutesSNOMED = 14,
    Medications = 15,
    CommercializedMedicationsSNOMED = 16,
}

impl MasterDataType {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}
