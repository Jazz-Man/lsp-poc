//! Cross-file index: resolves link targets to workspace files.
//!
//! Targets resolve against open documents first (the closure the server
//! wires to its document store), then against the disk through a
//! mtime+size-stamped cache. The workspace root for wikilinks is the
//! nearest ancestor holding `.git` (Zed worktrees are git repos),
//! discovered once and reset by `reset_root` on folder changes.

#![expect(
    dead_code,
    reason = "model lands before its consumers; wiring lands in cycle-1 task 4"
)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::SystemTime;

use async_language_server::lsp_types::Url;
use tree_sitter_md::MarkdownParser;

use crate::links::{self, Target};

/// (modification time, size in bytes) — any doubt re-reads.
type FileStamp = (SystemTime, u64);

#[derive(Debug)]
struct Entry {
    stamp: Option<FileStamp>,
    index: Arc<links::MdIndex>,
}

/// What a resolved target turned out to be.
#[derive(Debug)]
pub enum Resolved {
    /// The target file exists (or is open); its index is attached.
    Found(Arc<links::MdIndex>),
    /// No candidate path exists.
    Missing,
}

/// Cache of parsed workspace files plus the discovered workspace root.
///
/// The index owns its own parser so the server's parser and the index's
/// never contend for one Mutex.
pub struct Index {
    parser: Mutex<MarkdownParser>,
    cache: Mutex<HashMap<Url, Entry>>,
    root: Mutex<Option<PathBuf>>,
}

// MarkdownParser has no Debug impl, so the derive cannot cover the parser
// field; Debug reports the cache and root and elides the rest.
impl std::fmt::Debug for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Index")
            .field("cache", &self.cache)
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

impl Index {
    #[must_use]
    pub fn new() -> Self {
        Self {
            parser: Mutex::new(MarkdownParser::default()),
            cache: Mutex::new(HashMap::new()),
            root: Mutex::new(None),
        }
    }

    /// Forgets the discovered workspace root (folder changes).
    pub fn reset_root(&self) {
        *self.root.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }

    /// Resolves `target` to a file. Returns `None` only for
    /// [`Target::Fragment`] — same-document targets are the caller's job.
    /// Open documents (via `open`) win over the disk.
    pub fn resolve(
        &self,
        open: &dyn Fn(&Url) -> Option<Arc<links::MdIndex>>,
        self_url: &Url,
        target: &Target,
    ) -> Option<Resolved> {
        match target {
            Target::Fragment(_) => None,
            Target::Doc { path, .. } => Some(self.load(open, &self.doc_candidates(self_url, path))),
            Target::Wiki { path, .. } => {
                Some(self.load(open, &self.wiki_candidates(self_url, path)))
            }
        }
    }

    fn load(
        &self,
        open: &dyn Fn(&Url) -> Option<Arc<links::MdIndex>>,
        candidates: &[PathBuf],
    ) -> Resolved {
        for path in candidates {
            let Ok(url) = Url::from_file_path(path) else {
                continue;
            };
            if let Some(index) = open(&url) {
                return Resolved::Found(index);
            }
            if let Some(index) = self.load_from_disk(&url, path) {
                return Resolved::Found(index);
            }
        }
        Resolved::Missing
    }

    fn load_from_disk(&self, url: &Url, path: &Path) -> Option<Arc<links::MdIndex>> {
        let stamp = file_stamp(path)?;
        {
            let cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some(entry) = cache.get(url)
                && entry.stamp == Some(stamp)
            {
                return Some(Arc::clone(&entry.index));
            }
        }
        // arch-lint: allow(no-sync-io) reason="link targets load on demand inside synchronous notification hooks, which the protocol keeps sync"
        let text = std::fs::read_to_string(path).ok()?;
        let index = {
            let mut parser = self.parser.lock().unwrap_or_else(PoisonError::into_inner);
            links::build(&mut parser, &text)?
        };
        let index = Arc::new(index);
        self.cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(
                url.clone(),
                Entry {
                    stamp: Some(stamp),
                    index: Arc::clone(&index),
                },
            );
        Some(index)
    }

    fn doc_candidates(&self, self_url: &Url, path: &str) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        if let Ok(self_path) = self_url.to_file_path()
            && let Some(dir) = self_path.parent()
        {
            candidates.push(dir.join(path));
        }
        if let Some(root) = self.root_for(self_url) {
            candidates.push(root.join(path));
        }
        candidates
    }

    fn wiki_candidates(&self, self_url: &Url, path: &str) -> Vec<PathBuf> {
        let Some(root) = self.root_for(self_url) else {
            return Vec::new();
        };
        let base = root.join(path);
        if Path::new(path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            vec![base]
        } else {
            vec![base, root.join(format!("{path}.md"))]
        }
    }

    fn root_for(&self, self_url: &Url) -> Option<PathBuf> {
        {
            let root = self.root.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some(root) = &*root {
                return Some(root.clone());
            }
        }
        let discovered = git_root(&self_url.to_file_path().ok()?)?;
        *self.root.lock().unwrap_or_else(PoisonError::into_inner) = Some(discovered.clone());
        Some(discovered)
    }
}

/// Nearest ancestor directory holding `.git` (a dir or a worktree file).
fn git_root(from: &Path) -> Option<PathBuf> {
    from.ancestors()
        // arch-lint: allow(no-sync-io) reason="root discovery probes ancestor directories inside synchronous notification hooks, which the protocol keeps sync"
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
}

fn file_stamp(path: &Path) -> Option<FileStamp> {
    // arch-lint: allow(no-sync-io) reason="stamp probes ride the same synchronous hook context as the reads they gate"
    let metadata = std::fs::metadata(path).ok()?;
    Some((metadata.modified().ok()?, metadata.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::links::{Target, parse_destination};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lsp-poc-ws-{name}-{}", std::process::id()));
        // arch-lint: allow(no-sync-io) reason="temp-dir fixtures run under #[cfg(test)], never on the async runtime"
        let _ = std::fs::remove_dir_all(&dir);
        // arch-lint: allow(no-sync-io) reason="temp-dir fixtures run under #[cfg(test)], never on the async runtime"
        std::fs::create_dir_all(&dir).expect("temp dir creates");
        dir
    }

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            // arch-lint: allow(no-sync-io) reason="temp-dir fixtures run under #[cfg(test)], never on the async runtime"
            std::fs::create_dir_all(parent).expect("parent dir creates");
        }
        // arch-lint: allow(no-sync-io) reason="temp-dir fixtures run under #[cfg(test)], never on the async runtime"
        std::fs::write(path, contents).expect("fixture writes");
    }

    fn parse(text: &str) -> Arc<links::MdIndex> {
        let mut parser = MarkdownParser::default();
        Arc::new(links::build(&mut parser, text).expect("fixture parses"))
    }

    #[test]
    fn resolve_follows_doc_relative_and_wiki_paths() {
        let root = temp_dir("resolve");
        // arch-lint: allow(no-sync-io) reason="temp-dir fixtures run under #[cfg(test)], never on the async runtime"
        std::fs::create_dir_all(root.join(".git")).expect(".git dir creates");
        write(&root.join("a/doc.md"), "# Doc\n");
        write(&root.join("a/b.md"), "# B\n");
        write(&root.join("c.md"), "# C\n");

        let index = Index::new();
        let doc_url = Url::from_file_path(root.join("a/doc.md")).expect("doc url");
        let no_open = |_: &Url| None;

        let relative = index.resolve(
            &no_open,
            &doc_url,
            &parse_destination("b.md").expect("target"),
        );
        assert!(matches!(relative, Some(Resolved::Found(_))));

        assert!(matches!(
            index.resolve(
                &no_open,
                &doc_url,
                &parse_destination("nope.md").expect("target")
            ),
            Some(Resolved::Missing),
        ));
        assert!(
            index
                .resolve(
                    &no_open,
                    &doc_url,
                    &parse_destination("#doc").expect("target")
                )
                .is_none(),
            "fragments are the caller's job",
        );
        assert!(matches!(
            index.resolve(
                &no_open,
                &doc_url,
                &Target::Wiki {
                    path: "c".to_owned(),
                    fragment: None
                },
            ),
            Some(Resolved::Found(_)),
        ));
        assert!(matches!(
            index.resolve(
                &no_open,
                &doc_url,
                &Target::Wiki {
                    path: "ghost".to_owned(),
                    fragment: None
                },
            ),
            Some(Resolved::Missing),
        ));

        // arch-lint: allow(no-sync-io) reason="temp-dir fixtures run under #[cfg(test)], never on the async runtime"
        std::fs::remove_dir_all(&root).expect("cleanup removes");
    }

    #[test]
    fn resolve_prefers_open_documents_over_disk() {
        let root = temp_dir("open-preference");
        // arch-lint: allow(no-sync-io) reason="temp-dir fixtures run under #[cfg(test)], never on the async runtime"
        std::fs::create_dir_all(root.join(".git")).expect(".git dir creates");
        write(&root.join("doc.md"), "# Disk\n");
        let doc_url = Url::from_file_path(root.join("doc.md")).expect("doc url");
        let open_index = parse("# Open only\n");
        let url_for_open = doc_url.clone();
        let open = move |url: &Url| {
            (url == &url_for_open).then(|| Arc::<links::MdIndex>::clone(&open_index))
        };

        let index = Index::new();
        let Some(Resolved::Found(found)) = index.resolve(
            &open,
            &doc_url,
            &parse_destination("doc.md#open-only").expect("target"),
        ) else {
            panic!("the open document must resolve");
        };
        assert!(found.has_heading_slug("open-only"));

        // arch-lint: allow(no-sync-io) reason="temp-dir fixtures run under #[cfg(test)], never on the async runtime"
        std::fs::remove_dir_all(&root).expect("cleanup removes");
    }

    #[test]
    fn wikilinks_miss_without_a_git_root() {
        let root = temp_dir("no-root");
        write(&root.join("doc.md"), "# Doc\n");
        let doc_url = Url::from_file_path(root.join("doc.md")).expect("doc url");
        let index = Index::new();
        let no_open = |_: &Url| None;
        assert!(matches!(
            index.resolve(
                &no_open,
                &doc_url,
                &Target::Wiki {
                    path: "x".to_owned(),
                    fragment: None
                },
            ),
            Some(Resolved::Missing),
        ));
        // arch-lint: allow(no-sync-io) reason="temp-dir fixtures run under #[cfg(test)], never on the async runtime"
        std::fs::remove_dir_all(&root).expect("cleanup removes");
    }
}
