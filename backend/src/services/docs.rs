use html_escape::decode_html_entities;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::{AppError, AppResult};
use crate::models::DocPageSummary;

/// A single documentation page loaded from Sphinx HTML output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocPage {
    pub slug: String,
    pub title: String,
    pub lang: String,
    pub html: String,
}

/// In-memory index of all documentation pages.
#[derive(Debug, Default)]
pub struct DocsIndex {
    pages: Mutex<HashMap<String, DocPage>>,
    toc: Mutex<HashMap<String, Vec<DocPageSummary>>>,
}

impl DocsIndex {
    /// Scans the given HTML output directory and builds the index.
    pub fn load(html_dir: &str) -> AppResult<Self> {
        let dir = Path::new(html_dir);
        if !dir.exists() {
            tracing::warn!("docs html dir {} does not exist, starting with empty index", html_dir);
            return Ok(Self::default());
        }

        let mut pages = HashMap::new();
        let mut toc: HashMap<String, Vec<DocPageSummary>> = HashMap::new();

        for lang in ["zh", "en"] {
            let lang_dir = dir.join(lang);
            if !lang_dir.exists() {
                continue;
            }
            let mut summaries = Vec::new();
            Self::collect_html_files(&lang_dir, &lang_dir, lang, &mut pages, &mut summaries)?;
            Self::sort_by_toc(&lang_dir, &mut summaries);
            toc.insert(lang.to_string(), summaries);
        }

        Ok(Self {
            pages: Mutex::new(pages),
            toc: Mutex::new(toc),
        })
    }

    /// Sorts summaries according to the language-specific toc.json if present.
    fn sort_by_toc(lang_dir: &Path, summaries: &mut Vec<DocPageSummary>) {
        let toc_path = lang_dir.join("toc.json");
        if !toc_path.exists() {
            return;
        }

        let raw = match fs::read_to_string(&toc_path) {
            Ok(raw) => raw,
            Err(e) => {
                tracing::warn!("failed to read {}: {}", toc_path.display(), e);
                return;
            }
        };

        let toc: Vec<String> = match serde_json::from_str(&raw) {
            Ok(toc) => toc,
            Err(e) => {
                tracing::warn!("failed to parse {}: {}", toc_path.display(), e);
                return;
            }
        };

        let mut ordered = Vec::with_capacity(summaries.len());
        for slug in toc {
            if let Some(pos) = summaries.iter().position(|s| s.slug == slug) {
                ordered.push(summaries.remove(pos));
            } else {
                tracing::debug!("toc entry '{}' not found in built docs", slug);
            }
        }
        // Append any remaining pages in alphabetical order.
        summaries.sort_by(|a, b| a.slug.cmp(&b.slug));
        ordered.append(summaries);
        *summaries = ordered;
    }

    fn collect_html_files(
        base: &Path,
        current: &Path,
        lang: &str,
        pages: &mut HashMap<String, DocPage>,
        summaries: &mut Vec<DocPageSummary>,
    ) -> AppResult<()> {
        let title_re = Regex::new(r"<title>(.*?)</title>").unwrap();

        for entry in fs::read_dir(current).map_err(|e| AppError::Internal(e.to_string()))? {
            let entry = entry.map_err(|e| AppError::Internal(e.to_string()))?;
            let path = entry.path();
            if path.is_dir() {
                Self::collect_html_files(base, &path, lang, pages, summaries)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("html") {
                let html = fs::read_to_string(&path).map_err(|e| AppError::Internal(e.to_string()))?;
                let title = title_re
                    .captures(&html)
                    .and_then(|c| c.get(1))
                    .map(|m| decode_html_entities(m.as_str()).to_string())
                    .unwrap_or_else(|| "Untitled".to_string());

                let rel = path.strip_prefix(base).unwrap_or(&path);
                let mut slug = rel.to_string_lossy().to_string();
                if let Some(stem) = PathBuf::from(&slug).file_stem() {
                    let stem_str = stem.to_string_lossy().to_string();
                    let parent = rel.parent().and_then(|p| {
                        let s = p.to_string_lossy().to_string();
                        if s.is_empty() || s == "." {
                            None
                        } else {
                            Some(s)
                        }
                    });
                    slug = parent.map(|p| format!("{}/{}", p, stem_str)).unwrap_or(stem_str);
                }
                slug = slug.replace("\\", "/");

                if slug == "genindex" || slug == "search" {
                    continue;
                }

                let key = format!("{}/{}", lang, slug);
                summaries.push(DocPageSummary {
                    slug: slug.clone(),
                    title: title.clone(),
                    lang: lang.to_string(),
                });
                pages.insert(key, DocPage {
                    slug,
                    title,
                    lang: lang.to_string(),
                    html,
                });
            }
        }

        Ok(())
    }

    /// Retrieves a single documentation page by language and slug.
    pub fn get(&self, lang: &str, slug: &str) -> Option<DocPage> {
        let key = format!("{}/{}", lang, slug);
        self.pages.lock().ok()?.get(&key).cloned()
    }

    /// Lists all documentation pages for a language.
    pub fn list(&self, lang: &str) -> Vec<DocPageSummary> {
        self.toc
            .lock()
            .ok()
            .and_then(|toc| toc.get(lang).cloned())
            .unwrap_or_default()
    }

    /// Returns all languages present in the index.
    #[allow(dead_code)]
    pub fn languages(&self) -> Vec<String> {
        self.toc
            .lock()
            .ok()
            .map(|toc| toc.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Replaces the index contents by rescanning the given HTML directory.
    pub fn reload(&self, html_dir: &str) -> AppResult<()> {
        let new = Self::load(html_dir)?;
        let mut pages = self.pages.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let mut toc = self.toc.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        *pages = new.pages.into_inner().map_err(|e| AppError::Internal(e.to_string()))?;
        *toc = new.toc.into_inner().map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }
}

/// Recursively copies a directory tree from `src` to `dst`.
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> AppResult<()> {
    fs::create_dir_all(&dst).map_err(|e| AppError::Internal(e.to_string()))?;
    for entry in fs::read_dir(src).map_err(|e| AppError::Internal(e.to_string()))? {
        let entry = entry.map_err(|e| AppError::Internal(e.to_string()))?;
        let path = entry.path();
        let dest = dst.as_ref().join(entry.file_name());
        if path.is_dir() {
            copy_dir_all(&path, &dest)?;
        } else {
            fs::copy(&path, &dest).map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }
    Ok(())
}

/// Pulls pre-built documentation from a remote Git repository and reloads the index.
///
/// This function expects the remote repository to have a `gh-pages` branch whose root
/// contains `en/` and `zh/` directories with the built Sphinx HTML output.
pub async fn pull_docs_from_git(
    git_url: &str,
    html_dir: &str,
    index: &std::sync::RwLock<DocsIndex>,
) -> AppResult<()> {
    let tmp_dir = tempfile::tempdir().map_err(|e| AppError::Internal(e.to_string()))?;
    let tmp_path = tmp_dir.path().to_path_buf();

    tracing::info!("pulling docs from {}", git_url);
    let output = tokio::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            "gh-pages",
            git_url,
            tmp_path.to_string_lossy().as_ref(),
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("failed to run git: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::External(format!("git clone failed: {}", stderr)));
    }

    fs::create_dir_all(html_dir).map_err(|e| AppError::Internal(e.to_string()))?;

    for lang in ["en", "zh"] {
        let src = tmp_path.join(lang);
        if src.exists() {
            let dst = Path::new(html_dir).join(lang);
            if dst.exists() {
                fs::remove_dir_all(&dst).map_err(|e| AppError::Internal(e.to_string()))?;
            }
            copy_dir_all(&src, &dst)?;
            tracing::info!("updated docs language '{}'", lang);
        } else {
            tracing::warn!("docs language '{}' not found in remote", lang);
        }
    }

    let docs = index.write().map_err(|e| AppError::Internal(e.to_string()))?;
    docs.reload(html_dir)?;
    drop(docs);

    tracing::info!("docs index reloaded successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_doc(dir: &Path, slug: &str, title: &str) {
        let path = dir.join(format!("{}.html", slug));
        let mut file = fs::File::create(path).unwrap();
        write!(
            file,
            "<!DOCTYPE html><html><head><title>{}</title></head><body><h1>{}</h1></body></html>",
            title, title
        )
        .unwrap();
    }

    #[test]
    fn loads_docs_index() {
        let tmp = tempfile::tempdir().unwrap();
        let zh_dir = tmp.path().join("zh");
        fs::create_dir_all(&zh_dir).unwrap();
        create_test_doc(&zh_dir, "quick_start", "Quick Start");
        create_test_doc(&zh_dir, "basic", "Basic Features");

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        let zh = index.list("zh");
        assert_eq!(zh.len(), 2);
        assert!(zh.iter().any(|p| p.slug == "quick_start"));

        let page = index.get("zh", "quick_start").unwrap();
        assert_eq!(page.title, "Quick Start");
        assert!(page.html.contains("<h1>Quick Start"));
    }

    #[test]
    fn skips_genindex_and_search() {
        let tmp = tempfile::tempdir().unwrap();
        let zh_dir = tmp.path().join("zh");
        fs::create_dir_all(&zh_dir).unwrap();
        create_test_doc(&zh_dir, "quick_start", "Quick Start");
        create_test_doc(&zh_dir, "genindex", "Index");
        create_test_doc(&zh_dir, "search", "Search");

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(index.list("zh").len(), 1);
    }

    #[test]
    fn missing_dir_returns_empty_index() {
        let index = DocsIndex::load("/nonexistent/docs/path").unwrap();
        assert!(index.list("zh").is_empty());
        assert!(index.list("en").is_empty());
    }

    #[test]
    fn loads_nested_docs_and_lists_languages() {
        let tmp = tempfile::tempdir().unwrap();
        let en_dir = tmp.path().join("en").join("guide");
        fs::create_dir_all(&en_dir).unwrap();
        create_test_doc(&en_dir, "intro", "Intro");

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        let en = index.list("en");
        assert!(en.iter().any(|p| p.slug == "guide/intro"));
        let langs = index.languages();
        assert!(langs.contains(&"en".to_string()));
    }

    #[test]
    fn sorts_docs_by_toctree_and_appends_unknown_alphabetically() {
        let tmp = tempfile::tempdir().unwrap();
        let en_dir = tmp.path().join("en");
        fs::create_dir_all(&en_dir).unwrap();
        create_test_doc(&en_dir, "basic", "Basic");
        create_test_doc(&en_dir, "quick_start", "Quick Start");
        create_test_doc(&en_dir, "advanced", "Advanced");
        fs::write(
            en_dir.join("toc.json"),
            r#"["quick_start", "basic"]"#,
        )
        .unwrap();

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        let en = index.list("en");
        let slugs: Vec<_> = en.iter().map(|p| p.slug.as_str()).collect();
        assert_eq!(slugs, vec!["quick_start", "basic", "advanced"]);
    }

    #[test]
    fn falls_back_to_alphabetical_when_toc_json_is_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let zh_dir = tmp.path().join("zh");
        fs::create_dir_all(&zh_dir).unwrap();
        create_test_doc(&zh_dir, "zzz", "Zzz");
        create_test_doc(&zh_dir, "aaa", "Aaa");

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        let zh = index.list("zh");
        let slugs: Vec<_> = zh.iter().map(|p| p.slug.as_str()).collect();
        assert_eq!(slugs, vec!["aaa", "zzz"]);
    }

    fn run_git(args: &[&str], cwd: &Path) {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .status()
            .expect("git should be installed");
        assert!(status.success(), "git {:?} failed", args);
    }

    fn init_local_docs_repo() -> (tempfile::TempDir, String) {
        let tmp = tempfile::tempdir().unwrap();
        run_git(&["init"], tmp.path());
        run_git(&["config", "user.email", "test@example.com"], tmp.path());
        run_git(&["config", "user.name", "Test"], tmp.path());

        // Create the default branch with a dummy file so we can create gh-pages.
        fs::write(tmp.path().join("README.md"), "docs").unwrap();
        run_git(&["add", "README.md"], tmp.path());
        run_git(&["commit", "-m", "init"], tmp.path());

        // Build the gh-pages branch content.
        let pages = tmp.path().join("pages");
        let en_dir = pages.join("en");
        let zh_dir = pages.join("zh");
        fs::create_dir_all(&en_dir).unwrap();
        fs::create_dir_all(&zh_dir).unwrap();
        create_test_doc(&en_dir, "intro", "Intro");
        create_test_doc(&zh_dir, "start", "Start");
        fs::write(en_dir.join("toc.json"), r#"["intro"]"#).unwrap();

        run_git(&["checkout", "-b", "gh-pages"], tmp.path());
        // Move the built docs to the branch root and remove the default branch files.
        for entry in fs::read_dir(&pages).unwrap() {
            let entry = entry.unwrap();
            std::fs::rename(entry.path(), tmp.path().join(entry.file_name())).unwrap();
        }
        fs::remove_dir_all(&pages).unwrap();
        fs::remove_file(tmp.path().join("README.md")).unwrap();
        run_git(&["add", "."], tmp.path());
        run_git(&["commit", "-m", "docs"], tmp.path());

        let path = tmp.path().to_str().unwrap().to_string();
        (tmp, path)
    }

    #[tokio::test]
    async fn pull_docs_from_git_clones_and_reloads_index() {
        let (_repo, url) = init_local_docs_repo();
        let html_dir = tempfile::tempdir().unwrap();
        let index = std::sync::Arc::new(std::sync::RwLock::new(DocsIndex::default()));

        pull_docs_from_git(&url, html_dir.path().to_str().unwrap(), &index)
            .await
            .expect("pull should succeed");

        let docs = index.read().unwrap();
        let en = docs.list("en");
        assert!(en.iter().any(|p| p.slug == "intro"));
        let zh = docs.list("zh");
        assert!(zh.iter().any(|p| p.slug == "start"));
    }

    #[tokio::test]
    async fn pull_docs_from_git_fails_for_missing_branch() {
        let tmp = tempfile::tempdir().unwrap();
        run_git(&["init"], tmp.path());
        run_git(&["config", "user.email", "test@example.com"], tmp.path());
        run_git(&["config", "user.name", "Test"], tmp.path());
        fs::write(tmp.path().join("README.md"), "x").unwrap();
        run_git(&["add", "README.md"], tmp.path());
        run_git(&["commit", "-m", "init"], tmp.path());

        let html_dir = tempfile::tempdir().unwrap();
        let index = std::sync::Arc::new(std::sync::RwLock::new(DocsIndex::default()));
        let result = pull_docs_from_git(
            tmp.path().to_str().unwrap(),
            html_dir.path().to_str().unwrap(),
            &index,
        )
        .await;
        assert!(result.is_err());
    }
}
