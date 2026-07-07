use crate::error::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChapterState {
    Planned,
    SkeletonDrafting,
    SkeletonReview,
    ProseDrafting,
    ProseReview,
    Approved,
    Void,
}

impl ChapterState {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            ChapterState::Planned => "planned",
            ChapterState::SkeletonDrafting => "skeleton_drafting",
            ChapterState::SkeletonReview => "skeleton_review",
            ChapterState::ProseDrafting => "prose_drafting",
            ChapterState::ProseReview => "prose_review",
            ChapterState::Approved => "approved",
            ChapterState::Void => "void",
        }
    }
}

impl TryFrom<&str> for ChapterState {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let state = match value {
            "planned" => ChapterState::Planned,
            "skeleton_drafting" => ChapterState::SkeletonDrafting,
            "skeleton_review" => ChapterState::SkeletonReview,
            "prose_drafting" => ChapterState::ProseDrafting,
            "prose_review" => ChapterState::ProseReview,
            "approved" => ChapterState::Approved,
            "void" => ChapterState::Void,
            _ => {
                return Err(CoreError::UnknownDomainValue {
                    kind: "ChapterState",
                    value: value.to_string(),
                })
            }
        };
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [ChapterState; 7] = [
        ChapterState::Planned,
        ChapterState::SkeletonDrafting,
        ChapterState::SkeletonReview,
        ChapterState::ProseDrafting,
        ChapterState::ProseReview,
        ChapterState::Approved,
        ChapterState::Void,
    ];

    #[test]
    fn as_db_str_matches_mapping_table() {
        assert_eq!(ChapterState::Planned.as_db_str(), "planned");
        assert_eq!(ChapterState::SkeletonDrafting.as_db_str(), "skeleton_drafting");
        assert_eq!(ChapterState::SkeletonReview.as_db_str(), "skeleton_review");
        assert_eq!(ChapterState::ProseDrafting.as_db_str(), "prose_drafting");
        assert_eq!(ChapterState::ProseReview.as_db_str(), "prose_review");
        assert_eq!(ChapterState::Approved.as_db_str(), "approved");
        assert_eq!(ChapterState::Void.as_db_str(), "void");
    }

    #[test]
    fn roundtrip_every_variant() {
        for state in ALL {
            assert_eq!(ChapterState::try_from(state.as_db_str()).unwrap(), state);
        }
    }

    #[test]
    fn unknown_value_is_err() {
        let err = ChapterState::try_from("nope").unwrap_err();
        assert!(matches!(err, CoreError::UnknownDomainValue { kind: "ChapterState", .. }));
    }
}
