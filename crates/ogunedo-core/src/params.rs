#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterStatus {
    DevelopmentOnly,
    CryptanalysisRequired,
    ProductionApproved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parameters {
    pub id: u32,
    pub name: &'static str,
    pub q: u32,
    pub ring_degree: usize,
    pub rows: usize,
    pub columns: usize,
    pub coefficient_bound: i32,
    pub l2_bound_squared: u64,
    pub primitive_root: u32,
    pub status: ParameterStatus,
}

pub const DEV_PARAMETERS_ID: u32 = 0x4f47_0001;
pub const DRAFT_PARAMETERS_ID: u32 = 0x4f47_0101;

const DEV_PARAMETERS: Parameters = Parameters {
    id: DEV_PARAMETERS_ID,
    name: "OGUNEDO-DEV-N8-Q97-K4-V1",
    q: 97,
    ring_degree: 8,
    rows: 1,
    columns: 4,
    coefficient_bound: 2,
    l2_bound_squared: 128,
    primitive_root: 5,
    status: ParameterStatus::DevelopmentOnly,
};

const DRAFT_PARAMETERS: Parameters = Parameters {
    id: DRAFT_PARAMETERS_ID,
    name: "OGUNEDO-DRAFT-N256-Q12289-K18-V1",
    q: 12_289,
    ring_degree: 256,
    rows: 1,
    columns: 18,
    coefficient_bound: 2,
    l2_bound_squared: 10_240,
    primitive_root: 11,
    status: ParameterStatus::CryptanalysisRequired,
};

pub fn parameters(id: u32) -> Option<Parameters> {
    match id {
        DEV_PARAMETERS_ID => Some(DEV_PARAMETERS),
        DRAFT_PARAMETERS_ID => Some(DRAFT_PARAMETERS),
        _ => None,
    }
}
