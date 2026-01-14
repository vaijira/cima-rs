#![doc = include_str!("../README.md")]

pub mod api_client;
pub mod downloader;
pub mod endpoints;
pub mod models;
pub mod parser;

// Re-export main types for convenience
pub use api_client::CimaClient;
pub use endpoints::{
    MasterDataParams, SearchClinicalDescriptionParams, SearchMedicationsParams,
    SearchPresentationsParams, TechnicalSheetQuery,
};
pub use models::{
    ActiveIngredient, AtcCode, AuthorizationStatus, ChangeRecord, ClinicalDescription, Document,
    DocumentType, Excipient, MasterDataType, MasterItem, MaterialDocument, Medication,
    MedicationSummary, PaginatedResponse, Photo, Presentation, PresentationSummary, SafetyMaterial,
    SafetyNote, Section, SupplyProblem,
};
