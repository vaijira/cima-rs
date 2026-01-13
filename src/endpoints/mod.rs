pub mod cambios;
pub mod documentos;
pub mod maestras;
pub mod materiales;
pub mod medicamentos;
pub mod notas;
pub mod presentaciones;
pub mod suministro;
pub mod vmpp;

// Re-export commonly used types
pub use maestras::MaestraParams;
pub use medicamentos::{FichaTecnicaQuery, SearchMedicamentosParams};
pub use presentaciones::SearchPresentacionesParams;
pub use vmpp::SearchVmppParams;
