#![doc = include_str!("../README.md")]

pub mod api_client;
pub mod downloader;
pub mod endpoints;
pub mod models;
pub mod parser;

// Re-export main types for convenience
pub use api_client::CimaClient;
pub use endpoints::{
    FichaTecnicaQuery, MaestraParams, SearchMedicamentosParams, SearchPresentacionesParams,
    SearchVmppParams,
};
pub use models::{
    Atc, DescripcionClinica, Documento, DocumentoMaterial, Estado, Excipiente, Foto, Item,
    Material, Medicamento, MedicamentoList, Nota, PaginatedResponse, Presentacion,
    PresentacionList, PrincipioActivo, ProblemaSuministro, RegistroCambios, Seccion, TipoDocumento,
    TipoMaestra,
};
