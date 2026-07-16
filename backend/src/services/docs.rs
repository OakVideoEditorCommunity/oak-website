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
    pub version: String,
    pub html: String,
}

/// In-memory index of all documentation pages.
///
/// Pages are keyed by `"{version}/{lang}/{slug}"` and per-language TOCs by
/// `"{version}/{lang}"`. `versions` lists every known version ordered with the
/// default version first.
#[derive(Debug, Default)]
pub struct DocsIndex {
    pages: Mutex<HashMap<String, DocPage>>,
    toc: Mutex<HashMap<String, Vec<DocPageSummary>>>,
    versions: Mutex<Vec<String>>,
}

/// Manifest written by the docs builder at the root of the HTML output
/// directory when multiple documentation versions are present.
#[derive(Debug, Deserialize)]
struct VersionsManifest {
    #[serde(default)]
    versions: Vec<String>,
    #[serde(default)]
    latest: Option<String>,
}

impl DocsIndex {
    /// Scans the given HTML output directory and builds the index.
    ///
    /// When `{html_dir}/versions.json` exists and parses, each listed version is
    /// loaded from `{version}/{lang}/`. Otherwise the legacy layout is assumed:
    /// `zh/` and `en/` directly under `html_dir`, loaded as a single implicit
    /// version named `latest`.
    pub fn load(html_dir: &str) -> AppResult<Self> {
        let dir = Path::new(html_dir);
        if !dir.exists() {
            tracing::warn!(
                "docs html dir {} does not exist, starting with empty index",
                html_dir
            );
            return Ok(Self::default());
        }

        let mut pages = HashMap::new();
        let mut toc: HashMap<String, Vec<DocPageSummary>> = HashMap::new();

        let versions = match Self::read_versions_manifest(dir) {
            Some(versions) => {
                for version in &versions {
                    Self::scan_version(&dir.join(version), version, &mut pages, &mut toc)?;
                }
                versions
            }
            None => {
                // Legacy layout: language directories directly under html_dir.
                let version = "latest".to_string();
                Self::scan_version(dir, &version, &mut pages, &mut toc)?;
                vec![version]
            }
        };

        Ok(Self {
            pages: Mutex::new(pages),
            toc: Mutex::new(toc),
            versions: Mutex::new(versions),
        })
    }

    /// Reads and parses `{dir}/versions.json`. Returns `None` when the file is
    /// missing or unreadable, in which case the caller falls back to the legacy
    /// layout. The returned list has the default version first.
    fn read_versions_manifest(dir: &Path) -> Option<Vec<String>> {
        let path = dir.join("versions.json");
        if !path.exists() {
            return None;
        }

        let raw = match fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(e) => {
                tracing::warn!("failed to read {}: {}", path.display(), e);
                return None;
            }
        };

        let manifest: VersionsManifest = match serde_json::from_str(&raw) {
            Ok(manifest) => manifest,
            Err(e) => {
                tracing::warn!("failed to parse {}: {}", path.display(), e);
                return None;
            }
        };

        let mut versions: Vec<String> = manifest
            .versions
            .into_iter()
            .filter(|v| !v.is_empty())
            .collect();

        // The default version is kept first in the list.
        if let Some(latest) = manifest.latest {
            if let Some(pos) = versions.iter().position(|v| *v == latest) {
                let latest = versions.remove(pos);
                versions.insert(0, latest);
            }
        }

        Some(versions)
    }

    /// Scans `base/{lang}` for each supported language under the given version.
    fn scan_version(
        base: &Path,
        version: &str,
        pages: &mut HashMap<String, DocPage>,
        toc: &mut HashMap<String, Vec<DocPageSummary>>,
    ) -> AppResult<()> {
        for lang in ["zh", "en"] {
            let lang_dir = base.join(lang);
            if !lang_dir.exists() {
                continue;
            }
            let mut summaries = Vec::new();
            Self::collect_html_files(&lang_dir, &lang_dir, version, lang, pages, &mut summaries)?;
            Self::sort_by_toc(&lang_dir, &mut summaries);
            toc.insert(format!("{}/{}", version, lang), summaries);
        }
        Ok(())
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
        version: &str,
        lang: &str,
        pages: &mut HashMap<String, DocPage>,
        summaries: &mut Vec<DocPageSummary>,
    ) -> AppResult<()> {
        let title_re = Regex::new(r"<title>(.*?)</title>").unwrap();

        for entry in fs::read_dir(current).map_err(|e| AppError::Internal(e.to_string()))? {
            let entry = entry.map_err(|e| AppError::Internal(e.to_string()))?;
            let path = entry.path();
            if path.is_dir() {
                Self::collect_html_files(base, &path, version, lang, pages, summaries)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("html") {
                let html =
                    fs::read_to_string(&path).map_err(|e| AppError::Internal(e.to_string()))?;
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
                    slug = parent
                        .map(|p| format!("{}/{}", p, stem_str))
                        .unwrap_or(stem_str);
                }
                slug = slug.replace("\\", "/");

                if slug == "genindex" || slug == "search" {
                    continue;
                }

                let key = format!("{}/{}/{}", version, lang, slug);
                summaries.push(DocPageSummary {
                    slug: slug.clone(),
                    title: title.clone(),
                    lang: lang.to_string(),
                });
                pages.insert(
                    key,
                    DocPage {
                        slug,
                        title,
                        lang: lang.to_string(),
                        version: version.to_string(),
                        html,
                    },
                );
            }
        }

        Ok(())
    }

    /// Returns the default version (the first entry of the versions list).
    pub fn default_version(&self) -> Option<String> {
        self.versions.lock().ok()?.first().cloned()
    }

    /// Returns all known versions, ordered with the default version first.
    pub fn versions(&self) -> Vec<String> {
        self.versions.lock().map(|v| v.clone()).unwrap_or_default()
    }

    /// Returns whether the given version exists in the index.
    pub fn has_version(&self, version: &str) -> bool {
        self.versions
            .lock()
            .map(|v| v.iter().any(|x| x == version))
            .unwrap_or(false)
    }

    /// Retrieves a single documentation page by version, language and slug.
    pub fn get(&self, version: &str, lang: &str, slug: &str) -> Option<DocPage> {
        let key = format!("{}/{}/{}", version, lang, slug);
        self.pages.lock().ok()?.get(&key).cloned()
    }

    /// Lists all documentation pages for a version and language.
    pub fn list(&self, version: &str, lang: &str) -> Vec<DocPageSummary> {
        let key = format!("{}/{}", version, lang);
        self.toc
            .lock()
            .ok()
            .and_then(|toc| toc.get(&key).cloned())
            .unwrap_or_default()
    }

    /// Replaces the index contents by rescanning the given HTML directory.
    pub fn reload(&self, html_dir: &str) -> AppResult<()> {
        let new = Self::load(html_dir)?;
        let mut pages = self
            .pages
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut toc = self
            .toc
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let mut versions = self
            .versions
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *pages = new
            .pages
            .into_inner()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *toc = new
            .toc
            .into_inner()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        *versions = new
            .versions
            .into_inner()
            .map_err(|e| AppError::Internal(e.to_string()))?;
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
/// This function expects the remote repository to have a `gh-pages` branch whose
/// root holds either the versioned layout — a `versions.json` manifest plus one
/// `{version}/{lang}/` directory tree per documented version — or the legacy
/// layout with `en/` and `zh/` directories directly at the branch root. The
/// local HTML directory is synced to mirror the branch root: entries missing
/// from the branch are removed and same-named entries are replaced.
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

    // Sanity-check the branch root before touching the local copy.
    let has_manifest = tmp_path.join("versions.json").exists();
    let has_legacy_layout = tmp_path.join("en").exists() || tmp_path.join("zh").exists();
    if !has_manifest && !has_legacy_layout {
        return Err(AppError::External(format!(
            "gh-pages branch of {} contains neither versions.json nor language directories",
            git_url
        )));
    }

    let html_path = Path::new(html_dir);
    fs::create_dir_all(html_path).map_err(|e| AppError::Internal(e.to_string()))?;

    // Remove local top-level entries that no longer exist on the branch.
    for entry in fs::read_dir(html_path).map_err(|e| AppError::Internal(e.to_string()))? {
        let entry = entry.map_err(|e| AppError::Internal(e.to_string()))?;
        if tmp_path.join(entry.file_name()).exists() {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(|e| AppError::Internal(e.to_string()))?;
        } else {
            fs::remove_file(&path).map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }

    // Copy every entry from the branch root, replacing same-named local ones.
    for entry in fs::read_dir(&tmp_path).map_err(|e| AppError::Internal(e.to_string()))? {
        let entry = entry.map_err(|e| AppError::Internal(e.to_string()))?;
        if entry.file_name() == ".git" {
            continue;
        }
        let src = entry.path();
        let dst = html_path.join(entry.file_name());
        if dst.exists() {
            if dst.is_dir() {
                fs::remove_dir_all(&dst).map_err(|e| AppError::Internal(e.to_string()))?;
            } else {
                fs::remove_file(&dst).map_err(|e| AppError::Internal(e.to_string()))?;
            }
        }
        if src.is_dir() {
            copy_dir_all(&src, &dst)?;
        } else {
            fs::copy(&src, &dst).map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }

    let docs = index
        .write()
        .map_err(|e| AppError::Internal(e.to_string()))?;
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

    fn create_versioned_fixture(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        fs::write(
            dir.join("versions.json"),
            r#"{"versions": ["0.2.0", "0.1.0"], "latest": "0.2.0"}"#,
        )
        .unwrap();
        for (version, title) in [("0.2.0", "Intro v2"), ("0.1.0", "Intro v1")] {
            let en_dir = dir.join(version).join("en");
            fs::create_dir_all(&en_dir).unwrap();
            create_test_doc(&en_dir, "intro", title);
        }
    }

    #[test]
    fn loads_docs_index() {
        let tmp = tempfile::tempdir().unwrap();
        let zh_dir = tmp.path().join("zh");
        fs::create_dir_all(&zh_dir).unwrap();
        create_test_doc(&zh_dir, "quick_start", "Quick Start");
        create_test_doc(&zh_dir, "basic", "Basic Features");

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        let zh = index.list("latest", "zh");
        assert_eq!(zh.len(), 2);
        assert!(zh.iter().any(|p| p.slug == "quick_start"));

        let page = index.get("latest", "zh", "quick_start").unwrap();
        assert_eq!(page.title, "Quick Start");
        assert_eq!(page.version, "latest");
        assert!(page.html.contains("<h1>Quick Start"));
    }

    #[test]
    fn legacy_layout_is_loaded_as_latest_version() {
        let tmp = tempfile::tempdir().unwrap();
        let zh_dir = tmp.path().join("zh");
        fs::create_dir_all(&zh_dir).unwrap();
        create_test_doc(&zh_dir, "quick_start", "Quick Start");

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(index.versions(), vec!["latest".to_string()]);
        assert_eq!(index.default_version().as_deref(), Some("latest"));
        assert!(index.has_version("latest"));
        assert!(!index.has_version("0.1.0"));
    }

    #[test]
    fn loads_versioned_docs_index() {
        let tmp = tempfile::tempdir().unwrap();
        create_versioned_fixture(tmp.path());

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(
            index.versions(),
            vec!["0.2.0".to_string(), "0.1.0".to_string()]
        );
        assert_eq!(index.default_version().as_deref(), Some("0.2.0"));
        assert!(index.has_version("0.1.0"));
        assert!(!index.has_version("9.9.9"));

        let v2 = index.get("0.2.0", "en", "intro").unwrap();
        assert_eq!(v2.title, "Intro v2");
        assert_eq!(v2.version, "0.2.0");
        let v1 = index.get("0.1.0", "en", "intro").unwrap();
        assert_eq!(v1.title, "Intro v1");
        assert_eq!(v1.version, "0.1.0");

        assert!(index.get("9.9.9", "en", "intro").is_none());
        assert_eq!(index.list("0.1.0", "en").len(), 1);
        assert!(index.list("9.9.9", "en").is_empty());
    }

    #[test]
    fn versions_manifest_latest_becomes_default() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("versions.json"),
            r#"{"versions": ["0.2.0", "0.1.0"], "latest": "0.1.0"}"#,
        )
        .unwrap();
        let en_dir = tmp.path().join("0.1.0").join("en");
        fs::create_dir_all(&en_dir).unwrap();
        create_test_doc(&en_dir, "intro", "Intro");

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(index.default_version().as_deref(), Some("0.1.0"));
        assert_eq!(
            index.versions(),
            vec!["0.1.0".to_string(), "0.2.0".to_string()]
        );
    }

    #[test]
    fn versions_manifest_without_latest_defaults_to_first_entry() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("versions.json"),
            r#"{"versions": ["0.2.0", "0.1.0"]}"#,
        )
        .unwrap();

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(index.default_version().as_deref(), Some("0.2.0"));
    }

    #[test]
    fn invalid_versions_json_falls_back_to_legacy_layout() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("versions.json"), "not json").unwrap();
        let en_dir = tmp.path().join("en");
        fs::create_dir_all(&en_dir).unwrap();
        create_test_doc(&en_dir, "intro", "Intro");

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(index.versions(), vec!["latest".to_string()]);
        assert!(index.get("latest", "en", "intro").is_some());
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
        assert_eq!(index.list("latest", "zh").len(), 1);
    }

    #[test]
    fn missing_dir_returns_empty_index() {
        let index = DocsIndex::load("/nonexistent/docs/path").unwrap();
        assert!(index.list("latest", "zh").is_empty());
        assert!(index.list("latest", "en").is_empty());
        assert!(index.versions().is_empty());
        assert_eq!(index.default_version(), None);
    }

    #[test]
    fn loads_nested_docs() {
        let tmp = tempfile::tempdir().unwrap();
        let en_dir = tmp.path().join("en").join("guide");
        fs::create_dir_all(&en_dir).unwrap();
        create_test_doc(&en_dir, "intro", "Intro");

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        let en = index.list("latest", "en");
        assert!(en.iter().any(|p| p.slug == "guide/intro"));
        assert!(index.get("latest", "en", "guide/intro").is_some());
    }

    #[test]
    fn sorts_docs_by_toctree_and_appends_unknown_alphabetically() {
        let tmp = tempfile::tempdir().unwrap();
        let en_dir = tmp.path().join("en");
        fs::create_dir_all(&en_dir).unwrap();
        create_test_doc(&en_dir, "basic", "Basic");
        create_test_doc(&en_dir, "quick_start", "Quick Start");
        create_test_doc(&en_dir, "advanced", "Advanced");
        fs::write(en_dir.join("toc.json"), r#"["quick_start", "basic"]"#).unwrap();

        let index = DocsIndex::load(tmp.path().to_str().unwrap()).unwrap();
        let en = index.list("latest", "en");
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
        let zh = index.list("latest", "zh");
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

    fn init_local_versioned_docs_repo() -> (tempfile::TempDir, String) {
        let tmp = tempfile::tempdir().unwrap();
        run_git(&["init"], tmp.path());
        run_git(&["config", "user.email", "test@example.com"], tmp.path());
        run_git(&["config", "user.name", "Test"], tmp.path());

        // Create the default branch with a dummy file so we can create gh-pages.
        fs::write(tmp.path().join("README.md"), "docs").unwrap();
        run_git(&["add", "README.md"], tmp.path());
        run_git(&["commit", "-m", "init"], tmp.path());

        // Build the gh-pages branch content with a versioned layout.
        let pages = tmp.path().join("pages");
        create_versioned_fixture(&pages);

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

        // The .git directory of the clone must not leak into the html dir.
        assert!(!html_dir.path().join(".git").exists());

        let docs = index.read().unwrap();
        assert_eq!(docs.versions(), vec!["latest".to_string()]);
        let en = docs.list("latest", "en");
        assert!(en.iter().any(|p| p.slug == "intro"));
        let zh = docs.list("latest", "zh");
        assert!(zh.iter().any(|p| p.slug == "start"));
    }

    #[tokio::test]
    async fn pull_docs_from_git_syncs_versioned_tree() {
        let (_repo, url) = init_local_versioned_docs_repo();
        let html_dir = tempfile::tempdir().unwrap();
        // A stale file from a previous sync must be removed.
        fs::write(html_dir.path().join("stale.html"), "stale").unwrap();
        let index = std::sync::Arc::new(std::sync::RwLock::new(DocsIndex::default()));

        pull_docs_from_git(&url, html_dir.path().to_str().unwrap(), &index)
            .await
            .expect("pull should succeed");

        assert!(!html_dir.path().join("stale.html").exists());
        assert!(html_dir.path().join("versions.json").exists());
        assert!(!html_dir.path().join(".git").exists());

        let docs = index.read().unwrap();
        assert_eq!(
            docs.versions(),
            vec!["0.2.0".to_string(), "0.1.0".to_string()]
        );
        assert_eq!(docs.default_version().as_deref(), Some("0.2.0"));
        assert_eq!(docs.get("0.2.0", "en", "intro").unwrap().title, "Intro v2");
        assert_eq!(docs.get("0.1.0", "en", "intro").unwrap().title, "Intro v1");
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
