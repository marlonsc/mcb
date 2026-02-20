//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use mcb_domain::utils::path as domain_path;
use normpath::PathExt;
use rocket::http::ContentType;

use super::embedded;
use super::engine::Engines;
use super::template::TemplateInfo;

pub(crate) type Callback =
    Box<dyn Fn(&mut Engines) -> Result<(), Box<dyn Error>> + Send + Sync + 'static>;

pub(crate) struct Context {
    pub root: PathBuf,
    pub templates: HashMap<String, TemplateInfo>,
    pub engines: Engines,
}

pub(crate) use self::manager::ContextManager;

impl Context {
    pub fn initialize(root: &Path, callback: &Callback) -> Option<Context> {
        fn is_file_with_ext(entry: &walkdir::DirEntry, ext: &str) -> bool {
            let is_file = entry.file_type().is_file();
            let has_ext = entry.path().extension().is_some_and(|e| e == ext);
            is_file && has_ext
        }

        let root = match root.normalize() {
            Ok(root) => root.into_path_buf(),
            Err(_) => {
                info!(
                    "Template directory '{}' not found on disk, using embedded templates.",
                    root.display()
                );
                return Self::initialize_embedded(callback);
            }
        };

        let mut templates: HashMap<String, TemplateInfo> = HashMap::new();
        for &ext in Engines::ENABLED_EXTENSIONS {
            for entry in walkdir::WalkDir::new(&root).follow_links(true) {
                let entry = match entry {
                    Ok(entry) if is_file_with_ext(&entry, ext) => entry,
                    Ok(_) | Err(_) => continue,
                };

                let (name, data_type_str) = match split_path(&root, entry.path()) {
                    Ok(parts) => parts,
                    Err(error) => {
                        warn_!(
                            "Failed to split template path '{}' against root '{}': {}",
                            entry.path().display(),
                            root.display(),
                            error
                        );
                        continue;
                    }
                };
                if let Some(info) = templates.get(&*name) {
                    warn_!("Template name '{}' does not have a unique source.", name);
                    match info.path {
                        Some(ref path) => info_!("Existing path: {:?}", path),
                        None => info_!("Existing Content-Type: {}", info.data_type),
                    }

                    info_!("Additional path: {:?}", entry.path());
                    warn_!("Keeping existing template '{}'.", name);
                    continue;
                }

                let data_type = data_type_str
                    .as_ref()
                    .and_then(|ext| ContentType::from_extension(ext))
                    .unwrap_or(ContentType::Text);

                templates.insert(
                    name,
                    TemplateInfo {
                        path: Some(entry.into_path()),
                        engine_ext: ext,
                        data_type,
                    },
                );
            }
        }

        let mut engines = Engines::init(&templates)?;
        if let Err(e) = callback(&mut engines) {
            error_!("Template customization callback failed.");
            error_!("{}", e);
            return None;
        }

        for (name, engine_ext) in engines.templates() {
            if !templates.contains_key(name) {
                let data_type = Path::new(name)
                    .extension()
                    .and_then(|osstr| osstr.to_str())
                    .and_then(ContentType::from_extension)
                    .unwrap_or(ContentType::Text);

                let info = TemplateInfo {
                    path: None,
                    engine_ext,
                    data_type,
                };
                templates.insert(name.to_owned(), info);
            }
        }

        Some(Context {
            root,
            templates,
            engines,
        })
    }

    fn initialize_embedded(callback: &Callback) -> Option<Context> {
        let mut engines = Engines::init_embedded()?;
        if let Err(e) = callback(&mut engines) {
            error_!("Template customization callback failed: {}", e);
            return None;
        }

        let mut templates = HashMap::new();
        for &(name, _) in embedded::EMBEDDED_TEMPLATES {
            let data_type = Path::new(name)
                .extension()
                .and_then(|osstr| osstr.to_str())
                .and_then(ContentType::from_extension)
                .unwrap_or(ContentType::HTML);

            templates.insert(
                name.to_owned(),
                TemplateInfo {
                    path: None,
                    engine_ext: "hbs",
                    data_type,
                },
            );
        }

        for (name, engine_ext) in engines.templates() {
            if !templates.contains_key(name) {
                let data_type = Path::new(name)
                    .extension()
                    .and_then(|osstr| osstr.to_str())
                    .and_then(ContentType::from_extension)
                    .unwrap_or(ContentType::Text);

                templates.insert(
                    name.to_owned(),
                    TemplateInfo {
                        path: None,
                        engine_ext,
                        data_type,
                    },
                );
            }
        }

        Some(Context {
            root: PathBuf::from("<embedded>"),
            templates,
            engines,
        })
    }
}

#[cfg(not(debug_assertions))]
mod manager {
    use std::ops::Deref;

    use super::Context;

    pub(crate) struct ContextManager(Context);

    impl ContextManager {
        pub fn new(ctxt: Context) -> ContextManager {
            ContextManager(ctxt)
        }

        pub fn context<'a>(&'a self) -> impl Deref<Target = Context> + 'a {
            &self.0
        }

        pub fn is_reloading(&self) -> bool {
            false
        }
    }
}

#[cfg(debug_assertions)]
mod manager {
    use std::ops::{Deref, DerefMut};
    use std::sync::mpsc::{Receiver, channel};
    use std::sync::{Mutex, RwLock};

    use notify::{Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};

    use super::{Callback, Context};

    pub(crate) struct ContextManager {
        context: RwLock<Context>,
        watcher: Option<(RecommendedWatcher, Mutex<Receiver<NotifyResult<Event>>>)>,
    }

    impl ContextManager {
        pub fn new(ctxt: Context) -> ContextManager {
            let (tx, rx) = channel();
            let watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
                let _ = tx.send(res);
            })
            .and_then(|mut watcher| {
                watcher.watch(&ctxt.root.canonicalize()?, RecursiveMode::Recursive)?;
                Ok(watcher)
            });

            let watcher = match watcher {
                Ok(watcher) => Some((watcher, Mutex::new(rx))),
                Err(e) => {
                    warn!("Failed to enable live template reloading: {}", e);
                    debug_!("Reload error: {:?}", e);
                    warn_!("Live template reloading is unavailable.");
                    None
                }
            };

            ContextManager {
                watcher,
                context: RwLock::new(ctxt),
            }
        }

        pub fn context(&self) -> impl Deref<Target = Context> + '_ {
            self.context.read().unwrap_or_else(|poisoned| {
                warn_!(
                    "Template context RwLock poisoned while reading; continuing with inner value."
                );
                poisoned.into_inner()
            })
        }

        pub fn is_reloading(&self) -> bool {
            self.watcher.is_some()
        }

        fn context_mut(&self) -> impl DerefMut<Target = Context> + '_ {
            self.context.write().unwrap_or_else(|poisoned| {
                warn_!(
                    "Template context RwLock poisoned while writing; continuing with inner value."
                );
                poisoned.into_inner()
            })
        }

        pub fn reload_if_needed(&self, callback: &Callback) {
            let templates_changes = self.watcher.as_ref().map(|(_, rx)| {
                rx.lock()
                    .unwrap_or_else(|poisoned| {
                        warn_!("Template watcher mutex poisoned; continuing with inner receiver.");
                        poisoned.into_inner()
                    })
                    .try_iter()
                    .count()
                    > 0
            });

            if let Some(true) = templates_changes {
                info_!("Change detected: reloading templates.");
                let root = self.context().root.clone();
                if let Some(new_ctxt) = Context::initialize(&root, callback) {
                    *self.context_mut() = new_ctxt;
                } else {
                    warn_!("An error occurred while reloading templates.");
                    warn_!("Existing templates will remain active.");
                };
            }
        }
    }
}

fn remove_extension(path: &Path) -> PathBuf {
    let stem = match path.file_stem() {
        Some(stem) => stem,
        None => return path.to_path_buf(),
    };

    match path.parent() {
        Some(parent) => parent.join(stem),
        None => PathBuf::from(stem),
    }
}

fn split_path(root: &Path, path: &Path) -> mcb_domain::error::Result<(String, Option<String>)> {
    let rel_path = domain_path::strict_strip_prefix(path, root)?;
    let path_no_ext = remove_extension(&rel_path);
    let data_type = path_no_ext.extension();
    let mut name = domain_path::path_to_utf8_string(&remove_extension(&path_no_ext))?;

    if cfg!(windows) {
        name = name.replace("\\", "/");
    }

    let data_type = data_type
        .map(|d| domain_path::path_to_utf8_string(Path::new(d)))
        .transpose()?;

    Ok((name, data_type))
}
