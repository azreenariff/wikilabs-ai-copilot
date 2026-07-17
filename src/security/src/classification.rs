//! Data classification — compile-time types for data sensitivity.

pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

impl DataClassification {
    pub fn requires_encryption(&self) -> bool {
        matches!(self, DataClassification::Confidential | DataClassification::Restricted)
    }

    pub fn requires_audit(&self) -> bool {
        matches!(self, DataClassification::Restricted)
    }
}