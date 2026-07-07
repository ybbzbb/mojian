use crate::error::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SopPhase {
    StyleSampling,
    StyleExtracting,
    BriefDrafting,
    VisionDrafting,
    BibleBuilding,
    BibleCheck,
    BibleVerify,
    OutlineExpand,
    OutlineVerify,
    Writing,
}

impl SopPhase {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            SopPhase::StyleSampling => "style_sampling",
            SopPhase::StyleExtracting => "style_extracting",
            SopPhase::BriefDrafting => "brief_drafting",
            SopPhase::VisionDrafting => "vision_drafting",
            SopPhase::BibleBuilding => "bible_building",
            SopPhase::BibleCheck => "bible_check",
            SopPhase::BibleVerify => "bible_verify",
            SopPhase::OutlineExpand => "outline_expand",
            SopPhase::OutlineVerify => "outline_verify",
            SopPhase::Writing => "writing",
        }
    }
}

impl TryFrom<&str> for SopPhase {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let phase = match value {
            "style_sampling" => SopPhase::StyleSampling,
            "style_extracting" => SopPhase::StyleExtracting,
            "brief_drafting" => SopPhase::BriefDrafting,
            "vision_drafting" => SopPhase::VisionDrafting,
            "bible_building" => SopPhase::BibleBuilding,
            "bible_check" => SopPhase::BibleCheck,
            "bible_verify" => SopPhase::BibleVerify,
            "outline_expand" => SopPhase::OutlineExpand,
            "outline_verify" => SopPhase::OutlineVerify,
            "writing" => SopPhase::Writing,
            _ => {
                return Err(CoreError::UnknownDomainValue {
                    kind: "SopPhase",
                    value: value.to_string(),
                })
            }
        };
        Ok(phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [SopPhase; 10] = [
        SopPhase::StyleSampling,
        SopPhase::StyleExtracting,
        SopPhase::BriefDrafting,
        SopPhase::VisionDrafting,
        SopPhase::BibleBuilding,
        SopPhase::BibleCheck,
        SopPhase::BibleVerify,
        SopPhase::OutlineExpand,
        SopPhase::OutlineVerify,
        SopPhase::Writing,
    ];

    #[test]
    fn as_db_str_matches_mapping_table() {
        assert_eq!(SopPhase::StyleSampling.as_db_str(), "style_sampling");
        assert_eq!(SopPhase::StyleExtracting.as_db_str(), "style_extracting");
        assert_eq!(SopPhase::BriefDrafting.as_db_str(), "brief_drafting");
        assert_eq!(SopPhase::VisionDrafting.as_db_str(), "vision_drafting");
        assert_eq!(SopPhase::BibleBuilding.as_db_str(), "bible_building");
        assert_eq!(SopPhase::BibleCheck.as_db_str(), "bible_check");
        assert_eq!(SopPhase::BibleVerify.as_db_str(), "bible_verify");
        assert_eq!(SopPhase::OutlineExpand.as_db_str(), "outline_expand");
        assert_eq!(SopPhase::OutlineVerify.as_db_str(), "outline_verify");
        assert_eq!(SopPhase::Writing.as_db_str(), "writing");
    }

    #[test]
    fn roundtrip_every_variant() {
        for phase in ALL {
            assert_eq!(SopPhase::try_from(phase.as_db_str()).unwrap(), phase);
        }
    }

    #[test]
    fn unknown_value_is_err() {
        let err = SopPhase::try_from("nope").unwrap_err();
        assert!(matches!(err, CoreError::UnknownDomainValue { kind: "SopPhase", .. }));
    }
}
