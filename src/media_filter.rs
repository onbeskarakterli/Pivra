//! Window-local thumbnail projection; catalog order/selection remain untouched.
use crate::media::MediaKind;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MediaFilter {
    #[default]
    All,
    Photos,
    Videos,
}
impl MediaFilter {
    pub fn includes(self, kind: Option<MediaKind>) -> bool {
        match self {
            Self::All => true,
            Self::Photos => kind == Some(MediaKind::Image),
            Self::Videos => kind == Some(MediaKind::Video),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filtering_active_photo_or_video_never_changes_viewer_selection() {
        use crate::{catalog::MediaCollection, media::MediaItem};
        let mut c = MediaCollection::new(vec![
            MediaItem::new("1.png", MediaKind::Image),
            MediaItem::new("2.mp4", MediaKind::Video),
            MediaItem::new("10.jpg", MediaKind::Image),
        ]);
        c.select(1);
        let visible = |f: MediaFilter, c: &MediaCollection| {
            c.items()
                .iter()
                .enumerate()
                .filter(|(_, i)| f.includes(Some(i.kind())))
                .map(|(n, _)| n)
                .collect::<Vec<_>>()
        };
        assert_eq!(visible(MediaFilter::Photos, &c), vec![0, 2]);
        assert!(!visible(MediaFilter::Photos, &c).contains(&c.selected_index().unwrap()));
        assert_eq!(c.current().unwrap().path(), std::path::Path::new("2.mp4"));
        assert_eq!(visible(MediaFilter::Videos, &c), vec![1]);
        c.select(0);
        assert_eq!(visible(MediaFilter::Videos, &c), vec![1]);
        assert!(!visible(MediaFilter::Videos, &c).contains(&c.selected_index().unwrap()));
        assert_eq!(c.current().unwrap().path(), std::path::Path::new("1.png"));
        assert_eq!(visible(MediaFilter::Photos, &c), vec![0, 2]);
        assert_eq!(visible(MediaFilter::All, &c), vec![0, 1, 2]);
        assert_eq!(c.selected_index(), Some(0));
        c.renamed("1.png".as_ref(), "20.png".as_ref());
        assert_eq!(visible(MediaFilter::Videos, &c), vec![0]);
        assert_eq!(c.current().unwrap().path(), std::path::Path::new("20.png"));
        let empty = MediaCollection::new(vec![]);
        assert!(visible(MediaFilter::All, &empty).is_empty());
    }
}
