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

/// Estado de autorización de un medicamento o presentación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estado {
    /// Fecha de autorización (Unix Epoch GMT+2:00)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aut: Option<i64>,
    /// Fecha de suspensión (Unix Epoch GMT+2:00)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub susp: Option<i64>,
    /// Fecha de revocación (Unix Epoch GMT+2:00)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<i64>,
}

/// Item genérico usado en maestras
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Identificador numérico
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Identificador alfanumérico
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo: Option<String>,
    /// Nombre del elemento
    pub nombre: String,
}

/// Problema de suministro de una presentación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemaSuministro {
    /// Código nacional
    pub cn: String,
    /// Nombre de la presentación
    pub nombre: String,
    /// Fecha de inicio (Unix Epoch GMT+2:00)
    pub fini: i64,
    /// Fecha prevista de fin o fecha de solución (Unix Epoch GMT+2:00)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ffin: Option<i64>,
    /// Observaciones
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observ: Option<String>,
    /// Indica si sigue activo
    pub activo: bool,
}

/// Sección de un documento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seccion {
    /// Número de sección (hasta 3 niveles: "N.N.N")
    pub seccion: String,
    /// Título de la sección
    pub titulo: String,
    /// Orden de la sección
    pub orden: i32,
    /// Contenido en formato HTML
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contenido: Option<String>,
}

/// Tipo de documento
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TipoDocumento {
    /// Ficha Técnica
    FichaTecnica = 1,
    /// Prospecto
    Prospecto = 2,
    /// Informe Público Evaluación
    InformePublico = 3,
    /// Plan de gestión de riesgos
    PlanGestionRiesgos = 4,
}

/// Documento asociado a un medicamento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documento {
    /// Tipo de documento
    pub tipo: u8,
    /// URL para acceder al documento
    pub url: String,
    /// Indica si está disponible en HTML por secciones
    pub secc: bool,
    /// URL en formato HTML (sólo si secc = true)
    #[serde(rename = "urlHtml", skip_serializing_if = "Option::is_none")]
    pub url_html: Option<String>,
    /// Fecha de modificación (Unix Epoch GMT+2:00) - optional as some documents may not have a date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fecha: Option<i64>,
}

/// Nota de seguridad o informativa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nota {
    /// Tipo de nota (1: Nota Seguridad)
    pub tipo: u8,
    /// Número de la nota
    pub num: String,
    /// Referencia asociada
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    /// Asunto
    pub asunto: String,
    /// Fecha de publicación (Unix Epoch GMT+2:00)
    pub fecha: i64,
    /// URL para acceder a la nota
    pub url: String,
}

/// Documento de material informativo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentoMaterial {
    /// Título del documento
    pub nombre: String,
    /// URL para acceder
    pub url: String,
    /// Fecha de actualización (Unix Epoch GMT+2:00)
    pub fecha: i64,
}

/// Material informativo sobre seguridad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    /// Lista de documentos para profesionales sanitarios
    #[serde(rename = "listaDocsProfesional", default)]
    pub lista_docs_profesional: Vec<DocumentoMaterial>,
}

/// Descripción clínica (VMP/VMPP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescripcionClinica {
    /// Código de VMP
    pub vmp: String,
    /// Nombre del VMP
    #[serde(rename = "vmpDesc")]
    pub vmp_desc: String,
    /// Código de VMPP
    pub vmpp: String,
    /// Nombre del VMPP
    #[serde(rename = "vmppDesc")]
    pub vmpp_desc: String,
    /// Número de presentaciones comercializadas
    #[serde(rename = "presComerc")]
    pub pres_comerc: i32,
}

/// Código ATC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atc {
    /// Código ATC
    pub codigo: String,
    /// Nombre descriptivo
    pub nombre: String,
    /// Nivel del código ATC
    pub nivel: i32,
}

/// Principio activo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrincipioActivo {
    /// ID del principio activo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Código identificativo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codigo: Option<String>,
    /// Nombre
    pub nombre: String,
    /// Cantidad
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cantidad: Option<String>,
    /// Unidad
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unidad: Option<String>,
    /// Orden en la lista
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orden: Option<i32>,
}

/// Excipiente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Excipiente {
    /// ID del excipiente
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Nombre
    pub nombre: String,
    /// Cantidad
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cantidad: Option<String>,
    /// Unidad
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unidad: Option<String>,
    /// Orden en la lista
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orden: Option<i32>,
}

/// Foto de un medicamento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Foto {
    /// Tipo: "materialas" (material acondicionamiento) o "formafarmac" (forma farmacéutica)
    pub tipo: String,
    /// URL de la imagen
    pub url: String,
    /// Fecha de actualización (Unix Epoch GMT+2:00) - optional as some photos may not have a date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fecha: Option<i64>,
}

/// Presentación de un medicamento (vista simplificada para listados)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentacionList {
    /// Código nacional de la presentación
    pub cn: String,
    /// Nombre de la presentación
    pub nombre: String,
    /// Estado de registro
    pub estado: Estado,
    /// Indica si está comercializada
    pub comerc: bool,
    /// Indica si tiene problemas de suministro
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psum: Option<bool>,
}

/// Presentación completa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentacion {
    /// Código nacional
    pub cn: String,
    /// Nombre
    pub nombre: String,
    /// Estado
    pub estado: Estado,
    /// Indica si está comercializada
    pub comerc: bool,
    /// Indica si tiene problemas de suministro
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psum: Option<bool>,
}

/// Medicamento (vista simplificada para listados)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicamentoList {
    /// Nº de registro
    pub nregistro: String,
    /// Nombre del medicamento
    pub nombre: String,
    /// Laboratorio titular
    pub labtitular: String,
    /// Estado de registro
    pub estado: Estado,
    /// Condiciones de prescripción
    pub cpresc: String,
    /// Indica si tiene alguna presentación comercializada
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comerc: Option<bool>,
    /// Indica si necesita receta médica
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receta: Option<bool>,
    /// Indica si afecta a la conducción
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conduc: Option<bool>,
    /// Indica si tiene triángulo negro
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triangulo: Option<bool>,
    /// Indica si es huérfano
    #[serde(skip_serializing_if = "Option::is_none")]
    pub huerfano: Option<bool>,
    /// Indica si es biosimilar
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biosimilar: Option<bool>,
    /// Tipo de no sustituible
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nosustituible: Option<Item>,
    /// Indica si tiene problemas de suministro
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psum: Option<bool>,
    /// Indica si registrado por EMA
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema: Option<bool>,
    /// Indica si tiene notas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notas: Option<bool>,
    /// Indica si tiene materiales informativos
    #[serde(rename = "materialesInf", skip_serializing_if = "Option::is_none")]
    pub materiales_inf: Option<bool>,
    /// Documentos asociados
    #[serde(default)]
    pub docs: Vec<Documento>,
    /// Fotos asociadas
    #[serde(default)]
    pub fotos: Vec<Foto>,
    /// Vías de administración
    #[serde(rename = "viasAdministracion", default)]
    pub vias_administracion: Vec<Item>,
    /// Forma farmacéutica
    #[serde(rename = "formaFarmaceutica", skip_serializing_if = "Option::is_none")]
    pub forma_farmaceutica: Option<Item>,
    /// Forma farmacéutica simplificada
    #[serde(
        rename = "formaFarmaceuticaSimplificada",
        skip_serializing_if = "Option::is_none"
    )]
    pub forma_farmaceutica_simplificada: Option<Item>,
    /// Dosis de principios activos
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosis: Option<String>,
}

/// Medicamento completo con todos los detalles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medicamento {
    /// Nº de registro
    pub nregistro: String,
    /// Nombre del medicamento
    pub nombre: String,
    /// Lista de principios activos separados por comas
    pub pactivos: String,
    /// Laboratorio titular
    pub labtitular: String,
    /// Estado de registro
    pub estado: Estado,
    /// Condiciones de prescripción
    pub cpresc: String,
    /// Indica si tiene alguna presentación comercializada
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comerc: Option<bool>,
    /// Indica si necesita receta médica
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receta: Option<bool>,
    /// Indica si afecta a la conducción
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conduc: Option<bool>,
    /// Indica si tiene triángulo negro
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triangulo: Option<bool>,
    /// Indica si es huérfano
    #[serde(skip_serializing_if = "Option::is_none")]
    pub huerfano: Option<bool>,
    /// Indica si es biosimilar
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biosimilar: Option<bool>,
    /// Indica si registrado por EMA
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ema: Option<bool>,
    /// Indica si tiene problemas de suministro
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psum: Option<bool>,
    /// Documentos asociados
    #[serde(default)]
    pub docs: Vec<Documento>,
    /// Fotos asociadas
    #[serde(default)]
    pub fotos: Vec<Foto>,
    /// Indica si tiene notas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notas: Option<bool>,
    /// Indica si tiene materiales informativos
    #[serde(rename = "materialesInf", skip_serializing_if = "Option::is_none")]
    pub materiales_inf: Option<bool>,
    /// Códigos ATC
    #[serde(default)]
    pub atcs: Vec<Atc>,
    /// Principios activos
    #[serde(rename = "principiosActivos", default)]
    pub principios_activos: Vec<PrincipioActivo>,
    /// Excipientes
    #[serde(default)]
    pub excipientes: Vec<Excipiente>,
    /// Vías de administración
    #[serde(rename = "viasAdministracion", default)]
    pub vias_administracion: Vec<Item>,
    /// Tipo de no sustituible
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nosustituible: Option<Item>,
    /// Presentaciones
    #[serde(default)]
    pub presentaciones: Vec<PresentacionList>,
    /// Forma farmacéutica
    #[serde(rename = "formaFarmaceutica", skip_serializing_if = "Option::is_none")]
    pub forma_farmaceutica: Option<Item>,
    /// Forma farmacéutica simplificada
    #[serde(
        rename = "formaFarmaceuticaSimplificada",
        skip_serializing_if = "Option::is_none"
    )]
    pub forma_farmaceutica_simplificada: Option<Item>,
    /// Dosis de principios activos
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dosis: Option<String>,
}

/// Registro de cambios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistroCambios {
    /// Nº de registro del medicamento
    pub nregistro: String,
    /// Fecha del cambio (Unix Epoch GMT+2:00)
    pub fecha: i64,
    /// Tipo de cambio: 1=Nuevo, 2=Baja, 3=Modificado
    #[serde(rename = "tipoCambio")]
    pub tipo_cambio: u8,
    /// Lista de cambios: "estado", "comerc", "prosp", "ft", "psum", "notasSeguridad", "matinf", "otros"
    #[serde(default)]
    pub cambios: Vec<String>,
}

/// Tipo de maestra
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TipoMaestra {
    PrincipiosActivos = 1,
    FormasFarmaceuticas = 3,
    ViasAdministracion = 4,
    Laboratorios = 6,
    CodigosATC = 7,
    PrincipiosActivosSnomed = 11,
    FormasFarmaceuticasSimplificadasSnomed = 13,
    ViasAdministracionSimplificadasSnomed = 14,
    Medicamentos = 15,
    MedicamentosComercializadosSnomed = 16,
}

impl TipoMaestra {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}
