#![doc = include_str!("../README.md")]

pub mod api_client;
pub mod billing;
pub mod downloader;
pub mod endpoints;
pub mod models;
pub mod parser;
pub mod terminology;

// Re-export main types for convenience
pub use api_client::CimaClient;
pub use billing::{
    BillingExportSummary, BillingProduct, GroupPresentationMapping, HomogeneousGroup,
    ParallelImportRecord, compute_homogeneous_groups, export_billing_data_to_csvs,
    extract_parallel_imports, generate_group_presentation_mappings,
    load_parallel_import_cns_from_prescriptions_csv, parse_billing_csv, parse_billing_csv_reader,
    tag_parallel_imports_from_prescriptions,
};
pub use downloader::{
    BILLING_NOMENCLATOR_EXPORT_URL, download_and_extract_nomenclator, download_billing_nomenclator,
};
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
pub use terminology::{
    AmppRecord, DrugHierarchy, TerminologyExportSummary, TerminologyIndex, VmpRecord, VmppRecord,
    VtmRecord, build_from_aemps_csv_dir, export_terminology_to_csvs,
};
