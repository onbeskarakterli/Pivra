//! Quality lifecycle. Production currently has no certified early-preview provider.
//! File identity/cancellation belong to LoadSlot; this model never authorizes a token.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Requested,
    PreviewLoading,
    PreviewReady,
    FullLoading,
    FullReady,
    Failed,
    Cancelled,
}
impl Phase {
    /// Full quality is terminal: a late preview must never downgrade it.
    pub fn preview(self, success: bool) -> Self {
        match self {
            Self::FullReady | Self::Cancelled | Self::Failed => self,
            _ if success => Self::PreviewReady,
            _ => Self::FullLoading,
        }
    }
    /// Retain a usable preview on upgrade failure, rather than blanking it.
    pub fn full(self, success: bool, has_preview: bool) -> Self {
        if self == Self::Cancelled {
            self
        } else if success {
            Self::FullReady
        } else if has_preview {
            Self::PreviewReady
        } else {
            Self::Failed
        }
    }
}
#[cfg(test)]
mod tests {
    use super::Phase::*;
    #[test]
    fn direct_full_and_preview_fallbacks() {
        assert_eq!(FullLoading.full(true, false), FullReady);
        assert_eq!(PreviewLoading.preview(true).full(true, true), FullReady);
        assert_eq!(PreviewLoading.preview(true).full(false, true), PreviewReady);
        assert_eq!(PreviewLoading.preview(false).full(true, false), FullReady);
        assert_eq!(PreviewLoading.preview(false).full(false, false), Failed);
    }
    #[test]
    fn late_preview_and_cancelled_stages_cannot_replace_full_or_cancel() {
        assert_eq!(FullReady.preview(true), FullReady);
        assert_eq!(FullReady.preview(false), FullReady);
        assert_eq!(Cancelled.preview(true), Cancelled);
        assert_eq!(Cancelled.full(true, true), Cancelled);
        assert_eq!(Failed.preview(true), Failed);
        assert_eq!(Requested, Default::default());
    }
}
