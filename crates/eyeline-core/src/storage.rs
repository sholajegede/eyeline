use serde::Serialize;
use std::{
    fs, io,
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

const MAX_DEPTH: usize = 4;
const MAX_SCRIPT_BYTES: usize = 2 * 1024 * 1024;

const WELCOME: &str = "## Welcome\n\
This is your first script. Press Start to open the prompter next to your camera.\n\n\
## How cues work\n\
Every heading starts a new cue. Jump between cues with the hotkeys or by clicking a word.\n\n\
## Stage directions\n\
Anything in square brackets is shown but never read aloud [pause for a breath] so it will not confuse voice follow.\n";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ScriptMeta {
    pub id: String,
    pub title: String,
    pub modified: u64,
}

#[derive(Debug, Clone)]
pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn open(dir: PathBuf) -> io::Result<Self> {
        let existed = dir.exists();
        fs::create_dir_all(&dir)?;
        let store = Self { dir };
        if !existed {
            store.write("Welcome.md", WELCOME)?;
        }
        Ok(store)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn resolve(&self, id: &str) -> io::Result<PathBuf> {
        let path = Path::new(id);
        let safe = !id.is_empty()
            && id.ends_with(".md")
            && path.components().all(|c| matches!(c, Component::Normal(_)));
        if !safe {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid script id"));
        }
        Ok(self.dir.join(path))
    }

    pub fn list(&self) -> io::Result<Vec<ScriptMeta>> {
        let mut found = Vec::new();
        self.walk(&self.dir, 0, &mut found)?;
        found.sort_by(|a, b| b.modified.cmp(&a.modified).then_with(|| a.id.cmp(&b.id)));
        Ok(found)
    }

    fn walk(&self, dir: &Path, depth: usize, out: &mut Vec<ScriptMeta>) -> io::Result<()> {
        if depth > MAX_DEPTH {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_dir() {
                self.walk(&path, depth + 1, out)?;
            } else if kind.is_file() && name.ends_with(".md") {
                let rel = path
                    .strip_prefix(&self.dir)
                    .map_err(io::Error::other)?
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                let modified = entry
                    .metadata()?
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map_or(0, |d| d.as_secs());
                let title = title_of(&fs::read_to_string(&path).unwrap_or_default(), &name);
                out.push(ScriptMeta { id: rel, title, modified });
            }
        }
        Ok(())
    }

    pub fn read(&self, id: &str) -> io::Result<String> {
        fs::read_to_string(self.resolve(id)?)
    }

    pub fn write(&self, id: &str, content: &str) -> io::Result<()> {
        if content.len() > MAX_SCRIPT_BYTES {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "script is too large"));
        }
        let path = self.resolve(id)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("md.tmp");
        fs::write(&tmp, content)?;
        fs::rename(tmp, path)
    }

    pub fn create(&self, title: &str) -> io::Result<String> {
        let base = slug(title);
        let mut id = format!("{base}.md");
        let mut n = 2;
        while self.resolve(&id)?.exists() {
            id = format!("{base}-{n}.md");
            n += 1;
        }
        self.write(&id, &format!("## {}\n\n", title.trim()))?;
        Ok(id)
    }

    pub fn delete(&self, id: &str) -> io::Result<()> {
        fs::remove_file(self.resolve(id)?)
    }
}

fn title_of(content: &str, file_name: &str) -> String {
    content
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with('#'))
        .map(|l| l.trim_start_matches('#').trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| file_name.trim_end_matches(".md").to_string())
}

fn slug(title: &str) -> String {
    let cleaned: String = title
        .trim()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let collapsed = cleaned
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if collapsed.is_empty() {
        "Untitled".into()
    } else {
        collapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(tag: &str) -> (Store, PathBuf) {
        let dir = std::env::temp_dir().join(format!("eyeline-store-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        (Store::open(dir.clone()).unwrap(), dir)
    }

    #[test]
    fn seeds_a_welcome_script_on_first_run_only() {
        let (store, dir) = temp_store("seed");
        assert_eq!(store.list().unwrap().len(), 1);
        store.delete("Welcome.md").unwrap();
        let again = Store::open(dir.clone()).unwrap();
        assert!(again.list().unwrap().is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_path_traversal_and_non_markdown() {
        let (store, dir) = temp_store("traversal");
        for bad in ["../secret.md", "/etc/passwd.md", "a/../../b.md", "notes.txt", "", "..", "x/../y.md"] {
            assert!(store.read(bad).is_err(), "{bad} should be rejected");
            assert!(store.write(bad, "x").is_err(), "{bad} should be rejected");
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn creates_unique_ids_and_reads_back() {
        let (store, dir) = temp_store("create");
        let a = store.create("Launch Video").unwrap();
        let b = store.create("Launch Video").unwrap();
        assert_eq!(a, "Launch-Video.md");
        assert_eq!(b, "Launch-Video-2.md");
        assert!(store.read(&a).unwrap().starts_with("## Launch Video"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn lists_nested_scripts_with_titles() {
        let (store, dir) = temp_store("nested");
        store.write("clients/acme/pitch.md", "# Acme Pitch\nHello").unwrap();
        store.write("plain.md", "no heading here").unwrap();
        let listed = store.list().unwrap();
        let acme = listed.iter().find(|m| m.id == "clients/acme/pitch.md").unwrap();
        assert_eq!(acme.title, "Acme Pitch");
        let plain = listed.iter().find(|m| m.id == "plain.md").unwrap();
        assert_eq!(plain.title, "plain");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn refuses_oversized_scripts() {
        let (store, dir) = temp_store("big");
        let huge = "a".repeat(MAX_SCRIPT_BYTES + 1);
        assert!(store.write("big.md", &huge).is_err());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn slug_handles_symbols_and_empty_titles() {
        assert_eq!(slug("  Q3 / Board: update!  "), "Q3-Board-update");
        assert_eq!(slug("!!!"), "Untitled");
    }
}
