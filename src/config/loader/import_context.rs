use crate::error::{DeclarchError, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Track import chain for circular import detection.
#[derive(Debug, Default)]
pub(super) struct ImportContext {
    /// Stack of files currently being loaded (for cycle detection).
    stack: Vec<PathBuf>,
    /// Set of all visited files.
    visited: HashSet<PathBuf>,
}

impl ImportContext {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn push(&mut self, path: PathBuf) -> Result<()> {
        if let Some(pos) = self.stack.iter().position(|p| p == &path) {
            let cycle: Vec<String> = self.stack[pos..]
                .iter()
                .chain(std::iter::once(&path))
                .map(|p| p.display().to_string())
                .collect();
            return Err(DeclarchError::ConfigError(format!(
                "Circular import detected:\n  {}",
                cycle.join("\n  -> ")
            )));
        }

        self.stack.push(path.clone());
        self.visited.insert(path);
        Ok(())
    }

    pub(super) fn pop(&mut self) {
        self.stack.pop();
    }

    pub(super) fn contains(&self, path: &Path) -> bool {
        self.visited.contains(path)
    }
}

#[cfg(test)]
mod tests {
    use super::ImportContext;
    use std::path::PathBuf;

    #[test]
    fn rejects_circular_import_path() {
        let mut ctx = ImportContext::new();
        let a = PathBuf::from("/tmp/a.kdl");
        let b = PathBuf::from("/tmp/b.kdl");

        ctx.push(a.clone()).expect("first push should succeed");
        ctx.push(b).expect("second push should succeed");
        let err = ctx
            .push(a)
            .expect_err("re-pushing stack member should produce cycle error");
        let msg = err.to_string();
        assert!(msg.contains("Circular import detected"));
    }

    #[test]
    fn tracks_visited_paths_after_pop() {
        let mut ctx = ImportContext::new();
        let a = PathBuf::from("/tmp/a.kdl");

        ctx.push(a.clone()).expect("push should succeed");
        ctx.pop();

        assert!(ctx.contains(&a));
    }
}
