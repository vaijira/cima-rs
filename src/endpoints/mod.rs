pub mod changes;
pub mod clinical_descriptions;
pub mod documents;
pub mod master_data;
pub mod materials;
pub mod medications;
pub mod presentations;
pub mod safety_notes;
pub mod supply_problems;

// Re-export commonly used types
pub use clinical_descriptions::SearchClinicalDescriptionParams;
pub use master_data::MasterDataParams;
pub use medications::{SearchMedicationsParams, TechnicalSheetQuery};
pub use presentations::SearchPresentationsParams;
