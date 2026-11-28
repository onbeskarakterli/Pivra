//! Same-directory batch plans, built on identity-pinned rename tickets.
use super::{Ticket, validate_name};
use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct RenameStep {
    ticket: Arc<Ticket>,
    from: PathBuf,
    to: PathBuf,
}

impl RenameStep {
    pub fn new(ticket: Arc<Ticket>, name: &str) -> io::Result<Self> {
        validate_name(name).map_err(io::Error::other)?;
        Ok(Self {
            from: ticket.parent.join(&ticket.name),
            to: ticket.parent.join(name),
            ticket,
        })
    }

    pub fn paths(&self) -> (&Path, &Path) {
        (&self.from, &self.to)
    }

    fn invert(mut self) -> Self {
        std::mem::swap(&mut self.from, &mut self.to);
        self
    }
}

pub struct BatchReversal(Mutex<Vec<RenameStep>>);

impl BatchReversal {
    pub fn new(completed: Vec<RenameStep>) -> Self {
        Self(Mutex::new(
            completed.into_iter().map(RenameStep::invert).collect(),
        ))
    }

    pub fn undo(&self) -> Result<RenameResult, String> {
        let mut remaining = self.0.lock().map_err(|_| "Rename history unavailable")?;
        let mut result = apply(std::mem::take(&mut *remaining));
        *remaining = std::mem::take(&mut result.retry);
        Ok(result)
    }
}

pub struct RenameResult {
    pub completed: Vec<RenameStep>,
    pub failed: usize,
    pub error: Option<String>,
    retry: Vec<RenameStep>,
}

pub fn is_temporary(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .and_then(|name| name.strip_prefix(".pivra-rename-"))
        .is_some_and(|name| {
            name.len() == 36
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
        })
}

pub fn apply(steps: Vec<RenameStep>) -> RenameResult {
    run(steps, || {})
}

struct Staged {
    step: RenameStep,
    temporary: String,
    fields: Vec<(String, String)>,
}

fn run(steps: Vec<RenameStep>, before_publish: impl FnOnce()) -> RenameResult {
    let mut result = RenameResult {
        completed: Vec::new(),
        failed: 0,
        error: None,
        retry: Vec::new(),
    };
    let mut staged = Vec::new();
    // Stage all inputs to free every original name, including cycles. Each
    // stage and publication is no-replace and checks its pinned source.
    for step in steps {
        if step.from == step.to {
            continue;
        }
        let name = format!(".pivra-rename-{}", gtk::glib::uuid_string_random());
        let fields = crate::media_information::lookup(&step.from);
        match step
            .ticket
            .rename_entry(step.from.file_name().unwrap(), OsStr::new(&name))
        {
            Ok(()) => staged.push(Staged {
                step,
                temporary: name,
                fields,
            }),
            Err(error) => fail(&mut result, step, error.to_string()),
        }
    }
    before_publish();
    for stage in staged {
        let mut step = stage.step;
        let temporary = OsStr::new(&stage.temporary);
        match step
            .ticket
            .rename_entry(temporary, step.to.file_name().unwrap())
        {
            Ok(()) => {
                if let Ok(key) = crate::image_cache::FileKey::read(&step.to) {
                    crate::media_information::remember(key, stage.fields);
                }
                result.completed.push(step);
            }
            Err(publish) => {
                let message = match step
                    .ticket
                    .rename_entry(temporary, step.from.file_name().unwrap())
                {
                    Ok(()) => {
                        if let Ok(key) = crate::image_cache::FileKey::read(&step.from) {
                            crate::media_information::remember(key, stage.fields);
                        }
                        publish.to_string()
                    }
                    Err(rollback) => {
                        step.from = step.ticket.parent.join(&stage.temporary);
                        format!(
                            "{publish}. Rollback: {rollback}. Original staged at {}",
                            step.from.display()
                        )
                    }
                };
                fail(&mut result, step, message);
            }
        }
    }
    result
}

fn fail(result: &mut RenameResult, step: RenameStep, message: String) {
    result.failed += 1;
    // Always surface a stranded temporary path, even after another item fails.
    if is_temporary(&step.from) {
        eprintln!("Pivra batch rename recovery: {message}");
    }
    if result.error.is_none() || is_temporary(&step.from) {
        result.error = Some(message);
    }
    result.retry.push(step);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn step(path: PathBuf, target: &str) -> RenameStep {
        RenameStep::new(Arc::new(Ticket::capture(&path).unwrap()), target).unwrap()
    }

    #[test]
    fn batch_rename_cycles_and_inverse_keep_exact_files() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.png");
        let b = dir.path().join("b.png");
        fs::write(&a, b"A").unwrap();
        fs::write(&b, b"B").unwrap();
        let result = apply(vec![step(a.clone(), "b.png"), step(b.clone(), "a.png")]);
        assert_eq!((result.completed.len(), result.failed), (2, 0));
        assert_eq!(fs::read(&a).unwrap(), b"B");
        assert_eq!(fs::read(&b).unwrap(), b"A");
        let undone = BatchReversal::new(result.completed).undo().unwrap();
        assert_eq!((undone.completed.len(), undone.failed), (2, 0));
        assert_eq!(fs::read(&a).unwrap(), b"A");
        assert_eq!(fs::read(&b).unwrap(), b"B");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }

    #[test]
    fn batch_rename_partial_publish_collision_and_retry_preserve_sources() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.jpg");
        let b = dir.path().join("b.png");
        let target = dir.path().join("target.png");
        fs::write(&a, b"A").unwrap();
        fs::write(&b, b"B").unwrap();
        let result = run(
            vec![step(a.clone(), "done.jpg"), step(b.clone(), "target.png")],
            || {
                fs::write(&target, b"unrelated").unwrap();
            },
        );
        assert_eq!((result.completed.len(), result.failed), (1, 1));
        assert_eq!(fs::read(&target).unwrap(), b"unrelated");
        assert_eq!(fs::read(&b).unwrap(), b"B");

        let reversal = BatchReversal::new(result.completed);
        fs::write(&a, b"retry obstruction").unwrap();
        assert_eq!(reversal.undo().unwrap().failed, 1);
        assert_eq!(fs::read(&a).unwrap(), b"retry obstruction");
        fs::remove_file(&a).unwrap();
        assert_eq!(reversal.undo().unwrap().completed.len(), 1);
        assert_eq!(fs::read(&a).unwrap(), b"A");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 3);
    }
}
