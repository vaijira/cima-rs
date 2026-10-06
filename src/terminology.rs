use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Virtual Therapeutic Moiety (VTM) / Denominación Común de Sustancia Activa (DCSA).
/// Highest abstract clinical level representing the active therapeutic substance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VtmRecord {
    /// Official VTM code or DCSA code
    pub code: String,
    /// Active substance name / clinical description
    pub name: String,
}

/// Virtual Medicinal Product (VMP) / Denominación Común del Principio activo (DCP).
/// Abstract drug entity: Active Substance + Strength/Dosage + Pharmaceutical Form (independent of brand).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VmpRecord {
    /// Official VMP code or DCP code
    pub code: String,
    /// Virtual medicinal product name (e.g. "Paracetamol 1 g comprimido")
    pub name: String,
    /// Parent VTM / DCSA code
    pub vtm_code: Option<String>,
}

/// Virtual Medicinal Product Pack (VMPP) / Denominación Común con Formato/Envase (DCPF).
/// Virtual pack representation: VMP + Pack size/count (e.g. "Paracetamol 1 g comprimido, 20 comprimidos").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VmppRecord {
    /// Official VMPP code or DCPF code
    pub code: String,
    /// Virtual pack description
    pub name: String,
    /// Parent VMP / DCP code
    pub vmp_code: Option<String>,
}

/// Actual Medicinal Product Pack (AMPP) / Presentación comercial autorizada.
/// Concrete commercial/generic package with assigned 6-digit Código Nacional (CN).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmppRecord {
    /// Official AMPP identifier or CN
    pub code: String,
    /// 6-digit National Code (Código Nacional)
    pub cn: String,
    /// Complete commercial presentation name
    pub name: String,
    /// Parent VMPP / DCPF code
    pub vmpp_code: Option<String>,
    /// Marketing authorization holder / laboratory
    pub laboratory: Option<String>,
    /// Whether the presentation is commercialized
    pub is_commercialized: bool,
}

/// Full hierarchical ancestry representation for a medicine presentation:
/// VTM -> VMP -> VMPP -> AMPP (CN).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DrugHierarchy {
    /// 6-digit National Code (Código Nacional)
    pub cn: String,
    /// Actual Medicinal Product Pack (AMPP) code
    pub ampp_code: String,
    /// Commercial presentation name
    pub ampp_name: String,
    /// Parent VMPP code
    pub vmpp_code: Option<String>,
    /// Parent VMPP description
    pub vmpp_name: Option<String>,
    /// Parent VMP code
    pub vmp_code: Option<String>,
    /// Parent VMP description
    pub vmp_name: Option<String>,
    /// Parent VTM code
    pub vtm_code: Option<String>,
    /// Parent VTM description
    pub vtm_name: Option<String>,
    /// Laboratory
    pub laboratory: Option<String>,
    /// Commercialization status
    pub is_commercialized: bool,
}

/// Summary of generated CSV files from terminology export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminologyExportSummary {
    pub total_vtms: usize,
    pub total_vmps: usize,
    pub total_vmpps: usize,
    pub total_ampps: usize,
    pub total_hierarchies: usize,
    pub vtm_csv: PathBuf,
    pub vmp_csv: PathBuf,
    pub vmpp_csv: PathBuf,
    pub ampp_csv: PathBuf,
    pub hierarchy_csv: PathBuf,
}

/// Bi-directional index providing O(1) traversal across the clinical hierarchy:
/// VTM -> VMP -> VMPP -> AMPP (CN).
#[derive(Debug, Clone, Default)]
pub struct TerminologyIndex {
    pub vtms: HashMap<String, VtmRecord>,
    pub vmps: HashMap<String, VmpRecord>,
    pub vmpps: HashMap<String, VmppRecord>,
    pub ampps: HashMap<String, AmppRecord>,
    pub cn_to_ampp: HashMap<String, String>,
    pub vmp_to_vmpps: HashMap<String, Vec<String>>,
    pub vmpp_to_ampps: HashMap<String, Vec<String>>,
    pub vtm_to_vmps: HashMap<String, Vec<String>>,
}

impl TerminologyIndex {
    /// Creates an empty TerminologyIndex.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a VTM record to the index.
    pub fn add_vtm(&mut self, record: VtmRecord) {
        self.vtms.insert(record.code.clone(), record);
    }

    /// Adds a VMP record to the index and updates parent VTM relationship.
    pub fn add_vmp(&mut self, record: VmpRecord) {
        if let Some(ref vtm_code) = record.vtm_code {
            self.vtm_to_vmps
                .entry(vtm_code.clone())
                .or_default()
                .push(record.code.clone());
        }
        self.vmps.insert(record.code.clone(), record);
    }

    /// Adds a VMPP record to the index and updates parent VMP relationship.
    pub fn add_vmpp(&mut self, record: VmppRecord) {
        if let Some(ref vmp_code) = record.vmp_code {
            self.vmp_to_vmpps
                .entry(vmp_code.clone())
                .or_default()
                .push(record.code.clone());
        }
        self.vmpps.insert(record.code.clone(), record);
    }

    /// Adds an AMPP record to the index and updates parent VMPP relationship and CN lookup.
    pub fn add_ampp(&mut self, record: AmppRecord) {
        self.cn_to_ampp
            .insert(record.cn.clone(), record.code.clone());
        if let Some(ref vmpp_code) = record.vmpp_code {
            self.vmpp_to_ampps
                .entry(vmpp_code.clone())
                .or_default()
                .push(record.code.clone());
        }
        self.ampps.insert(record.code.clone(), record);
    }

    /// Retrieves a VTM by its code.
    pub fn get_vtm(&self, code: &str) -> Option<&VtmRecord> {
        self.vtms.get(code)
    }

    /// Retrieves a VMP by its code.
    pub fn get_vmp(&self, code: &str) -> Option<&VmpRecord> {
        self.vmps.get(code)
    }

    /// Retrieves a VMPP by its code.
    pub fn get_vmpp(&self, code: &str) -> Option<&VmppRecord> {
        self.vmpps.get(code)
    }

    /// Retrieves an AMPP by its code.
    pub fn get_ampp(&self, code: &str) -> Option<&AmppRecord> {
        self.ampps.get(code)
    }

    /// Traverses upward from a Código Nacional (CN) to resolve the full hierarchy:
    /// AMPP (CN) -> VMPP -> VMP -> VTM.
    pub fn lookup_by_cn(&self, cn: &str) -> Option<DrugHierarchy> {
        let trimmed_cn = cn.trim();
        let ampp_code = self.cn_to_ampp.get(trimmed_cn)?;
        self.lookup_by_ampp_code(ampp_code)
    }

    /// Traverses upward from an AMPP code to resolve the full hierarchy.
    pub fn lookup_by_ampp_code(&self, ampp_code: &str) -> Option<DrugHierarchy> {
        let ampp = self.ampps.get(ampp_code)?;

        let (vmpp_code, vmpp_name, vmp_code, vmp_name, vtm_code, vtm_name) =
            if let Some(ref p_code) = ampp.vmpp_code {
                if let Some(vmpp) = self.vmpps.get(p_code) {
                    let vmp_info = if let Some(ref v_code) = vmpp.vmp_code {
                        if let Some(vmp) = self.vmps.get(v_code) {
                            let vtm_info = if let Some(ref t_code) = vmp.vtm_code {
                                if let Some(vtm) = self.vtms.get(t_code) {
                                    (Some(vtm.code.clone()), Some(vtm.name.clone()))
                                } else {
                                    (Some(t_code.clone()), None)
                                }
                            } else {
                                (None, None)
                            };
                            (
                                Some(vmp.code.clone()),
                                Some(vmp.name.clone()),
                                vtm_info.0,
                                vtm_info.1,
                            )
                        } else {
                            (Some(v_code.clone()), None, None, None)
                        }
                    } else {
                        (None, None, None, None)
                    };
                    (
                        Some(vmpp.code.clone()),
                        Some(vmpp.name.clone()),
                        vmp_info.0,
                        vmp_info.1,
                        vmp_info.2,
                        vmp_info.3,
                    )
                } else {
                    (Some(p_code.clone()), None, None, None, None, None)
                }
            } else {
                (None, None, None, None, None, None)
            };

        Some(DrugHierarchy {
            cn: ampp.cn.clone(),
            ampp_code: ampp.code.clone(),
            ampp_name: ampp.name.clone(),
            vmpp_code,
            vmpp_name,
            vmp_code,
            vmp_name,
            vtm_code,
            vtm_name,
            laboratory: ampp.laboratory.clone(),
            is_commercialized: ampp.is_commercialized,
        })
    }

    /// Finds all bioequivalent presentations (AMPPs) sharing the same VMP as the given CN.
    pub fn get_equivalents_by_cn(&self, cn: &str) -> Vec<AmppRecord> {
        if let Some(hierarchy) = self.lookup_by_cn(cn)
            && let Some(ref vmp_code) = hierarchy.vmp_code
        {
            return self.get_ampps_by_vmp(vmp_code);
        }
        Vec::new()
    }

    /// Finds all commercial presentations (AMPPs) grouped under a VMP code.
    pub fn get_ampps_by_vmp(&self, vmp_code: &str) -> Vec<AmppRecord> {
        let mut results = Vec::new();
        if let Some(vmpp_codes) = self.vmp_to_vmpps.get(vmp_code) {
            for vmpp_code in vmpp_codes {
                if let Some(ampp_codes) = self.vmpp_to_ampps.get(vmpp_code) {
                    for ampp_code in ampp_codes {
                        if let Some(ampp) = self.ampps.get(ampp_code) {
                            results.push(ampp.clone());
                        }
                    }
                }
            }
        }
        results.sort_by(|a, b| a.cn.cmp(&b.cn));
        results
    }

    /// Finds all VMPPs under a given VMP code.
    pub fn get_vmpps_by_vmp(&self, vmp_code: &str) -> Vec<VmppRecord> {
        let mut results = Vec::new();
        if let Some(vmpp_codes) = self.vmp_to_vmpps.get(vmp_code) {
            for vmpp_code in vmpp_codes {
                if let Some(vmpp) = self.vmpps.get(vmpp_code) {
                    results.push(vmpp.clone());
                }
            }
        }
        results.sort_by(|a, b| a.code.cmp(&b.code));
        results
    }

    /// Finds all VMPs associated with a given VTM code.
    pub fn get_vmps_by_vtm(&self, vtm_code: &str) -> Vec<VmpRecord> {
        let mut results = Vec::new();
        if let Some(vmp_codes) = self.vtm_to_vmps.get(vtm_code) {
            for vmp_code in vmp_codes {
                if let Some(vmp) = self.vmps.get(vmp_code) {
                    results.push(vmp.clone());
                }
            }
        }
        results.sort_by(|a, b| a.code.cmp(&b.code));
        results
    }

    /// Searches VTMs matching a case-insensitive query string.
    pub fn search_vtm(&self, query: &str) -> Vec<&VtmRecord> {
        let q = query.to_uppercase();
        let mut results: Vec<_> = self
            .vtms
            .values()
            .filter(|r| r.name.to_uppercase().contains(&q) || r.code.contains(&q))
            .collect();
        results.sort_by(|a, b| a.code.cmp(&b.code));
        results
    }

    /// Searches VMPs matching a case-insensitive query string.
    pub fn search_vmp(&self, query: &str) -> Vec<&VmpRecord> {
        let q = query.to_uppercase();
        let mut results: Vec<_> = self
            .vmps
            .values()
            .filter(|r| r.name.to_uppercase().contains(&q) || r.code.contains(&q))
            .collect();
        results.sort_by(|a, b| a.code.cmp(&b.code));
        results
    }

    /// Searches AMPPs matching a case-insensitive query string.
    pub fn search_ampp(&self, query: &str) -> Vec<&AmppRecord> {
        let q = query.to_uppercase();
        let mut results: Vec<_> = self
            .ampps
            .values()
            .filter(|r| r.name.to_uppercase().contains(&q) || r.cn.contains(&q))
            .collect();
        results.sort_by(|a, b| a.cn.cmp(&b.cn));
        results
    }

    /// Builds a list of all resolved `DrugHierarchy` representations across all AMPP items.
    pub fn build_all_hierarchies(&self) -> Vec<DrugHierarchy> {
        let mut list: Vec<_> = self
            .ampps
            .keys()
            .filter_map(|ampp_code| self.lookup_by_ampp_code(ampp_code))
            .collect();
        list.sort_by(|a, b| a.cn.cmp(&b.cn));
        list
    }
}

/// Helper function to build a `TerminologyIndex` from an extracted AEMPS CSV directory
/// containing `dcsa.csv`, `dcp.csv`, `dcpf.csv`, and `prescriptions.csv`.
pub fn build_from_aemps_csv_dir<P: AsRef<Path>>(dir: P) -> Result<TerminologyIndex> {
    let dir = dir.as_ref();
    let mut index = TerminologyIndex::new();

    // 1. Load DCSA (VTM)
    let dcsa_path = dir.join("dcsa.csv");
    if dcsa_path.exists() {
        let file = File::open(&dcsa_path)
            .with_context(|| format!("Failed to open dcsa.csv at {:?}", dcsa_path))?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);
        for result in rdr.records() {
            let record = result?;
            if record.len() >= 2 {
                index.add_vtm(VtmRecord {
                    code: record[0].trim().to_string(),
                    name: record[1].trim().to_string(),
                });
            }
        }
    }

    // 2. Load DCP (VMP)
    let dcp_path = dir.join("dcp.csv");
    if dcp_path.exists() {
        let file = File::open(&dcp_path)
            .with_context(|| format!("Failed to open dcp.csv at {:?}", dcp_path))?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);
        for result in rdr.records() {
            let record = result?;
            if record.len() >= 3 {
                index.add_vmp(VmpRecord {
                    code: record[0].trim().to_string(),
                    name: record[1].trim().to_string(),
                    vtm_code: if record[2].trim().is_empty() {
                        None
                    } else {
                        Some(record[2].trim().to_string())
                    },
                });
            }
        }
    }

    // 3. Load DCPF (VMPP)
    let dcpf_path = dir.join("dcpf.csv");
    if dcpf_path.exists() {
        let file = File::open(&dcpf_path)
            .with_context(|| format!("Failed to open dcpf.csv at {:?}", dcpf_path))?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);
        for result in rdr.records() {
            let record = result?;
            if record.len() >= 3 {
                index.add_vmpp(VmppRecord {
                    code: record[0].trim().to_string(),
                    name: record[1].trim().to_string(),
                    vmp_code: if record[2].trim().is_empty() {
                        None
                    } else {
                        Some(record[2].trim().to_string())
                    },
                });
            }
        }
    }

    // 4. Load Prescriptions (AMPP)
    let presc_path = dir.join("prescriptions.csv");
    if presc_path.exists() {
        let file = File::open(&presc_path)
            .with_context(|| format!("Failed to open prescriptions.csv at {:?}", presc_path))?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);
        let headers = rdr.headers()?.clone();

        let cn_idx = headers.iter().position(|h| h.trim() == "cod_nacion");
        let name_idx = headers.iter().position(|h| h.trim() == "des_prese");
        let nomco_idx = headers.iter().position(|h| h.trim() == "des_nomco");
        let dcpf_idx = headers.iter().position(|h| h.trim() == "cod_dcpf");
        let lab_idx = headers
            .iter()
            .position(|h| h.trim() == "laboratorio_titular");
        let comer_idx = headers.iter().position(|h| h.trim() == "sw_comercializado");

        if let (Some(c_i), Some(p_i)) = (cn_idx, name_idx) {
            for result in rdr.records() {
                let record = result?;
                let raw_cn = record.get(c_i).unwrap_or("").trim();
                if raw_cn.is_empty() {
                    continue;
                }
                let cn = if raw_cn.len() < 6 && raw_cn.chars().all(|c| c.is_ascii_digit()) {
                    format!("{:0>6}", raw_cn)
                } else {
                    raw_cn.to_string()
                };

                let desc_prese = record.get(p_i).unwrap_or("").trim();
                let des_nomco = nomco_idx.and_then(|i| record.get(i)).unwrap_or("").trim();
                let full_name = if !des_nomco.is_empty() && !desc_prese.is_empty() {
                    format!("{} - {}", des_nomco, desc_prese)
                } else if !desc_prese.is_empty() {
                    desc_prese.to_string()
                } else {
                    des_nomco.to_string()
                };

                let vmpp_code = dcpf_idx
                    .and_then(|i| record.get(i))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                let laboratory = lab_idx
                    .and_then(|i| record.get(i))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                let is_commercialized = comer_idx
                    .and_then(|i| record.get(i))
                    .map(|s| s.trim() == "1" || s.trim().eq_ignore_ascii_case("true"))
                    .unwrap_or(false);

                index.add_ampp(AmppRecord {
                    code: cn.clone(),
                    cn,
                    name: full_name,
                    vmpp_code,
                    laboratory,
                    is_commercialized,
                });
            }
        }
    }

    Ok(index)
}

/// Exports the clinical hierarchy from a `TerminologyIndex` into relational CSV files
/// ready for direct database importation.
pub fn export_terminology_to_csvs<P: AsRef<Path>>(
    index: &TerminologyIndex,
    output_dir: P,
) -> Result<TerminologyExportSummary> {
    let out_dir = output_dir.as_ref();
    std::fs::create_dir_all(out_dir).context("Failed to create terminology output directory")?;

    let vtm_path = out_dir.join("terminology_vtm.csv");
    let vmp_path = out_dir.join("terminology_vmp.csv");
    let vmpp_path = out_dir.join("terminology_vmpp.csv");
    let ampp_path = out_dir.join("terminology_ampp.csv");
    let hierarchy_path = out_dir.join("terminology_hierarchy.csv");

    // 1. Export VTM
    {
        let mut writer = csv::Writer::from_path(&vtm_path)
            .with_context(|| format!("Failed to create {:?}", vtm_path))?;
        let mut vtms: Vec<_> = index.vtms.values().collect();
        vtms.sort_by(|a, b| a.code.cmp(&b.code));
        for vtm in vtms {
            writer.serialize(vtm)?;
        }
        writer.flush()?;
    }

    // 2. Export VMP
    {
        let mut writer = csv::Writer::from_path(&vmp_path)
            .with_context(|| format!("Failed to create {:?}", vmp_path))?;
        let mut vmps: Vec<_> = index.vmps.values().collect();
        vmps.sort_by(|a, b| a.code.cmp(&b.code));
        for vmp in vmps {
            writer.serialize(vmp)?;
        }
        writer.flush()?;
    }

    // 3. Export VMPP
    {
        let mut writer = csv::Writer::from_path(&vmpp_path)
            .with_context(|| format!("Failed to create {:?}", vmpp_path))?;
        let mut vmpps: Vec<_> = index.vmpps.values().collect();
        vmpps.sort_by(|a, b| a.code.cmp(&b.code));
        for vmpp in vmpps {
            writer.serialize(vmpp)?;
        }
        writer.flush()?;
    }

    // 4. Export AMPP
    {
        let mut writer = csv::Writer::from_path(&ampp_path)
            .with_context(|| format!("Failed to create {:?}", ampp_path))?;
        let mut ampps: Vec<_> = index.ampps.values().collect();
        ampps.sort_by(|a, b| a.cn.cmp(&b.cn));
        for ampp in ampps {
            writer.serialize(ampp)?;
        }
        writer.flush()?;
    }

    // 5. Export Full Denormalized Hierarchy View
    let hierarchies = index.build_all_hierarchies();
    {
        let mut writer = csv::Writer::from_path(&hierarchy_path)
            .with_context(|| format!("Failed to create {:?}", hierarchy_path))?;
        for h in &hierarchies {
            writer.serialize(h)?;
        }
        writer.flush()?;
    }

    Ok(TerminologyExportSummary {
        total_vtms: index.vtms.len(),
        total_vmps: index.vmps.len(),
        total_vmpps: index.vmpps.len(),
        total_ampps: index.ampps.len(),
        total_hierarchies: hierarchies.len(),
        vtm_csv: vtm_path,
        vmp_csv: vmp_path,
        vmpp_csv: vmpp_path,
        ampp_csv: ampp_path,
        hierarchy_csv: hierarchy_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminology_index_hierarchy_traversal() {
        let mut index = TerminologyIndex::new();

        index.add_vtm(VtmRecord {
            code: "DCSA001".to_string(),
            name: "PARACETAMOL".to_string(),
        });

        index.add_vmp(VmpRecord {
            code: "DCP001".to_string(),
            name: "PARACETAMOL 1 G COMPRIMIDO".to_string(),
            vtm_code: Some("DCSA001".to_string()),
        });

        index.add_vmpp(VmppRecord {
            code: "DCPF001".to_string(),
            name: "PARACETAMOL 1 G 20 COMPRIMIDOS".to_string(),
            vmp_code: Some("DCP001".to_string()),
        });

        index.add_ampp(AmppRecord {
            code: "AMPP650123".to_string(),
            cn: "650123".to_string(),
            name: "PARACETAMOL CINFA 1 G 20 COMPRIMIDOS EFG".to_string(),
            vmpp_code: Some("DCPF001".to_string()),
            laboratory: Some("LABORATORIOS CINFA S.A.".to_string()),
            is_commercialized: true,
        });

        index.add_ampp(AmppRecord {
            code: "AMPP650124".to_string(),
            cn: "650124".to_string(),
            name: "PARACETAMOL STADA 1 G 20 COMPRIMIDOS EFG".to_string(),
            vmpp_code: Some("DCPF001".to_string()),
            laboratory: Some("STADA S.L.".to_string()),
            is_commercialized: true,
        });

        // 1. Upward traversal from CN -> VMPP -> VMP -> VTM
        let hierarchy = index.lookup_by_cn("650123").unwrap();
        assert_eq!(hierarchy.cn, "650123");
        assert_eq!(
            hierarchy.ampp_name,
            "PARACETAMOL CINFA 1 G 20 COMPRIMIDOS EFG"
        );
        assert_eq!(hierarchy.vmpp_code.as_deref(), Some("DCPF001"));
        assert_eq!(
            hierarchy.vmpp_name.as_deref(),
            Some("PARACETAMOL 1 G 20 COMPRIMIDOS")
        );
        assert_eq!(hierarchy.vmp_code.as_deref(), Some("DCP001"));
        assert_eq!(
            hierarchy.vmp_name.as_deref(),
            Some("PARACETAMOL 1 G COMPRIMIDO")
        );
        assert_eq!(hierarchy.vtm_code.as_deref(), Some("DCSA001"));
        assert_eq!(hierarchy.vtm_name.as_deref(), Some("PARACETAMOL"));
        assert!(hierarchy.is_commercialized);

        // 2. Downward traversal from VMP -> Equivalent AMPPs
        let equivalents = index.get_ampps_by_vmp("DCP001");
        assert_eq!(equivalents.len(), 2);
        assert_eq!(equivalents[0].cn, "650123");
        assert_eq!(equivalents[1].cn, "650124");

        // 3. Find equivalents by CN
        let cinfa_equivalents = index.get_equivalents_by_cn("650123");
        assert_eq!(cinfa_equivalents.len(), 2);

        // 4. Search
        let search_results = index.search_vmp("paracetamol");
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].code, "DCP001");
    }

    #[test]
    fn test_export_terminology_to_csvs() {
        let mut index = TerminologyIndex::new();

        index.add_vtm(VtmRecord {
            code: "DCSA001".to_string(),
            name: "IBUPROFENO".to_string(),
        });
        index.add_vmp(VmpRecord {
            code: "DCP001".to_string(),
            name: "IBUPROFENO 600 MG COMPRIMIDO".to_string(),
            vtm_code: Some("DCSA001".to_string()),
        });
        index.add_vmpp(VmppRecord {
            code: "DCPF001".to_string(),
            name: "IBUPROFENO 600 MG 40 COMPRIMIDOS".to_string(),
            vmp_code: Some("DCP001".to_string()),
        });
        index.add_ampp(AmppRecord {
            code: "600123".to_string(),
            cn: "600123".to_string(),
            name: "IBUPROFENO CINFA 600 MG 40 COMPRIMIDOS".to_string(),
            vmpp_code: Some("DCPF001".to_string()),
            laboratory: Some("CINFA".to_string()),
            is_commercialized: true,
        });

        let temp_dir = tempfile::tempdir().unwrap();
        let summary = export_terminology_to_csvs(&index, temp_dir.path()).unwrap();

        assert_eq!(summary.total_vtms, 1);
        assert_eq!(summary.total_vmps, 1);
        assert_eq!(summary.total_vmpps, 1);
        assert_eq!(summary.total_ampps, 1);
        assert_eq!(summary.total_hierarchies, 1);

        assert!(summary.vtm_csv.exists());
        assert!(summary.vmp_csv.exists());
        assert!(summary.vmpp_csv.exists());
        assert!(summary.ampp_csv.exists());
        assert!(summary.hierarchy_csv.exists());
    }

    #[test]
    fn test_build_from_aemps_csv_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path();

        // 1. Create mock dcsa.csv
        std::fs::write(
            path.join("dcsa.csv"),
            "cod_dcsa,des_dcsa\n101,AMOXICILINA\n",
        )
        .unwrap();

        // 2. Create mock dcp.csv
        std::fs::write(
            path.join("dcp.csv"),
            "cod_dcp,des_dcp,cod_dcsa\n201,AMOXICILINA 500 MG CAPSULA,101\n",
        )
        .unwrap();

        // 3. Create mock dcpf.csv
        std::fs::write(
            path.join("dcpf.csv"),
            "cod_dcpf,des_dcpf,cod_dcp\n301,AMOXICILINA 500 MG 24 CAPSULAS,201\n",
        )
        .unwrap();

        // 4. Create mock prescriptions.csv
        std::fs::write(
            path.join("prescriptions.csv"),
            "cod_nacion,des_nomco,des_prese,cod_dcpf,laboratorio_titular,sw_comercializado\n712345,CLAMOXIL,500 mg 24 capsulas,301,GLAXOSMITHKLINE,1\n",
        )
        .unwrap();

        let index = build_from_aemps_csv_dir(path).unwrap();
        assert_eq!(index.vtms.len(), 1);
        assert_eq!(index.vmps.len(), 1);
        assert_eq!(index.vmpps.len(), 1);
        assert_eq!(index.ampps.len(), 1);

        let hierarchy = index.lookup_by_cn("712345").unwrap();
        assert_eq!(hierarchy.cn, "712345");
        assert_eq!(hierarchy.vtm_code.as_deref(), Some("101"));
        assert_eq!(hierarchy.vtm_name.as_deref(), Some("AMOXICILINA"));
        assert_eq!(hierarchy.vmp_code.as_deref(), Some("201"));
        assert_eq!(
            hierarchy.vmp_name.as_deref(),
            Some("AMOXICILINA 500 MG CAPSULA")
        );
        assert_eq!(hierarchy.vmpp_code.as_deref(), Some("301"));
        assert_eq!(
            hierarchy.vmpp_name.as_deref(),
            Some("AMOXICILINA 500 MG 24 CAPSULAS")
        );
        assert_eq!(hierarchy.laboratory.as_deref(), Some("GLAXOSMITHKLINE"));
        assert!(hierarchy.is_commercialized);

        let search = index.search_ampp("clamoxil");
        assert_eq!(search.len(), 1);
        assert_eq!(search[0].cn, "712345");
    }
}
