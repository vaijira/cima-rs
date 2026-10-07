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
    BillingExportSummary, BillingProduct, DEFAULT_KNOWN_IMPORTERS, EmaParallelDistributionIndex,
    EmaParallelDistributionRecord, GroupPresentationMapping, HomogeneousGroup,
    ParallelImportDetection, ParallelImportDetector, ParallelImportRecord, ParallelImporterCatalog,
    builtin_ema_spain_records, compute_homogeneous_groups, detect_parallel_import_syntax,
    export_billing_data_to_csvs, extract_parallel_imports, generate_group_presentation_mappings,
    is_destination_spain, is_status_active, load_ema_register_csv,
    load_parallel_import_cns_from_prescriptions_csv, merge_ema_records, normalize_company_name,
    parse_billing_csv, parse_billing_csv_reader, parse_ema_register_csv,
    tag_parallel_imports_from_ema, tag_parallel_imports_from_prescriptions,
    tag_parallel_imports_with_detector,
};
pub use downloader::{
    BILLING_NOMENCLATOR_EXPORT_URL, EMA_IRIS_REGISTER_URL, download_and_extract_nomenclator,
    download_billing_nomenclator, download_ema_parallel_distribution_register,
};
pub use endpoints::{
    MasterDataParams, SearchClinicalDescriptionParams, SearchMedicationsParams,
    SearchPresentationsParams, TechnicalSheetQuery,
};
pub use models::{
    ActiveIngredient, AtcCode, AuthorizationStatus, ChangeRecord, ClinicalDescription, Document,
    DocumentType, Excipient, MasterDataType, MasterItem, MaterialDocument, Medication,
    MedicationSummary, PaginatedResponse, ParallelImportDossierInfo, Photo, Presentation,
    PresentationSummary, SafetyMaterial, SafetyNote, Section, SupplyProblem,
};
pub use terminology::{
    AmppRecord, DrugHierarchy, TerminologyExportSummary, TerminologyIndex, VmpRecord, VmppRecord,
    VtmRecord, build_from_aemps_csv_dir, export_terminology_to_csvs,
};
