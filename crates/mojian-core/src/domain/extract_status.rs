use crate::error::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtractStatus {
    Pending,
    Extracting,
    Extracted,
}

impl ExtractStatus {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            ExtractStatus::Pending => "pending",
            ExtractStatus::Extracting => "extracting",
            ExtractStatus::Extracted => "extracted",
        }
    }
}

impl TryFrom<&str> for ExtractStatus {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let status = match value {
            "pending" => ExtractStatus::Pending,
            "extracting" => ExtractStatus::Extracting,
            "extracted" => ExtractStatus::Extracted,
            _ => {
                return Err(CoreError::UnknownDomainValue {
                    kind: "ExtractStatus",
                    value: value.to_string(),
                })
            }
        };
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [ExtractStatus; 3] = [
        ExtractStatus::Pending,
        ExtractStatus::Extracting,
        ExtractStatus::Extracted,
    ];

    #[test]
    fn as_db_str_matches_mapping_table() {
        assert_eq!(ExtractStatus::Pending.as_db_str(), "pending");
        assert_eq!(ExtractStatus::Extracting.as_db_str(), "extracting");
        assert_eq!(ExtractStatus::Extracted.as_db_str(), "extracted");
    }

    #[test]
    fn roundtrip_every_variant() {
        for status in ALL {
            assert_eq!(ExtractStatus::try_from(status.as_db_str()).unwrap(), status);
        }
    }

    #[test]
    fn unknown_value_is_err() {
        let err = ExtractStatus::try_from("nope").unwrap_err();
        assert!(matches!(err, CoreError::UnknownDomainValue { kind: "ExtractStatus", .. }));
    }
}
