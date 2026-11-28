//! Path-based thumbnail marks, independent of the selected viewer item.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub(super) struct Selection {
    anchor: Option<PathBuf>,
    marked: HashSet<PathBuf>,
}

impl Selection {
    pub(super) fn ordinary(&mut self, path: &Path) {
        self.clear();
        self.anchor = Some(path.to_owned());
    }

    pub(super) fn modify(
        &mut self,
        path: &Path,
        visible: &[PathBuf],
        active: Option<&Path>,
        control: bool,
        shift: bool,
    ) {
        let Some(target) = visible.iter().position(|item| item == path) else {
            return;
        };
        let anchor = self
            .anchor
            .as_deref()
            .and_then(|anchor| visible.iter().position(|item| item == anchor))
            .or_else(|| active.and_then(|active| visible.iter().position(|item| item == active)))
            .unwrap_or(target);
        if shift {
            if !control {
                self.marked.clear();
            }
            let range = anchor.min(target)..=anchor.max(target);
            self.marked.extend(visible[range].iter().cloned());
            self.anchor = Some(visible[anchor].clone());
        } else {
            if self.marked.is_empty() && anchor != target {
                self.marked.insert(visible[anchor].clone());
            }
            if !self.marked.remove(path) {
                self.marked.insert(path.to_owned());
            }
            self.anchor = Some(path.to_owned());
        }
    }

    pub(super) fn all(&mut self, visible: &[PathBuf]) {
        self.marked = visible.iter().cloned().collect();
        self.anchor = visible.first().cloned();
    }

    pub(super) fn reconcile(&mut self, catalog: &[(usize, PathBuf)]) {
        let paths: HashSet<_> = catalog.iter().map(|(_, path)| path.as_path()).collect();
        self.marked.retain(|path| paths.contains(path.as_path()));
        if self
            .anchor
            .as_deref()
            .is_some_and(|anchor| !paths.contains(anchor))
        {
            self.anchor = None;
        }
    }

    pub(super) fn contains(&self, path: &Path) -> bool {
        self.marked.contains(path)
    }

    pub(super) fn clear(&mut self) {
        self.marked.clear();
        self.anchor = None;
    }

    pub(super) fn has_marks(&self) -> bool {
        !self.marked.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marks_use_filtered_range_order_and_survive_reordering() {
        let paths: Vec<_> = ["a.jpg", "b.mp4", "c.png", "d.png"]
            .into_iter()
            .map(PathBuf::from)
            .collect();
        let mut selection = Selection::default();
        selection.ordinary(&paths[0]);
        selection.modify(&paths[2], &paths, Some(&paths[0]), true, false);
        assert!(selection.contains(&paths[0]) && selection.contains(&paths[2]));
        assert!(!selection.contains(&paths[1]));
        selection.modify(&paths[3], &paths, Some(&paths[0]), true, true);
        assert!(selection.contains(&paths[0]) && selection.contains(&paths[3]));
        let filtered = [paths[0].clone(), paths[3].clone()];
        selection.modify(&paths[0], &filtered, Some(&paths[3]), false, true);
        assert!(!selection.contains(&paths[1]) && !selection.contains(&paths[2]));
        selection.reconcile(&[(0, paths[3].clone()), (1, paths[1].clone())]);
        assert!(!selection.contains(&paths[0]) && selection.contains(&paths[3]));
        selection.all(&filtered);
        assert!(selection.contains(&paths[0]) && selection.contains(&paths[3]));
        selection.clear();
        assert!(!selection.has_marks());
    }
}
