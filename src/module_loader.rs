use crate::transpiler;
use deno_core::{
    anyhow::{self, anyhow},
    futures::{self, FutureExt},
    ModuleCodeBytes, ModuleLoadResponse, ModuleLoader, ModuleSource, ModuleSourceCode,
    ModuleSourceFuture, ModuleSpecifier, ModuleType,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::Mutex,
};

/// Module cache provider trait
/// Implement this trait to provide a custom module cache
/// You will need to use interior due to the deno's loader trait
/// Default cache for the loader is in-memory
pub trait ModuleCacheProvider {
    fn set(&self, specifier: &ModuleSpecifier, source: ModuleSource);
    fn get(&self, specifier: &ModuleSpecifier) -> Option<ModuleSource>;

    fn clone_source(&self, specifier: &ModuleSpecifier, source: &ModuleSource) -> ModuleSource {
        ModuleSource::new(
            source.module_type.clone(),
            match &source.code {
                ModuleSourceCode::String(s) => ModuleSourceCode::String(s.to_string().into()),
                ModuleSourceCode::Bytes(b) => {
                    ModuleSourceCode::Bytes(ModuleCodeBytes::Boxed(b.to_vec().into()))
                }
            },
            specifier,
            source.code_cache.clone(),
        )
    }
}

/// Default in-memory module cache provider
#[derive(Default)]
pub struct DefaultModuleCacheProvider(RefCell<HashMap<ModuleSpecifier, ModuleSource>>);
impl ModuleCacheProvider for DefaultModuleCacheProvider {
    fn set(&self, specifier: &ModuleSpecifier, source: ModuleSource) {
        self.0.borrow_mut().insert(specifier.clone(), source);
    }

    fn get(&self, specifier: &ModuleSpecifier) -> Option<ModuleSource> {
        let cache = self.0.borrow();
        let source = cache.get(specifier)?;
        Some(Self::clone_source(&self, specifier, source))
    }
}

#[cfg(feature = "url_import")]
async fn load_from_url(
    module_specifier: &ModuleSpecifier,
    cache_provider: &Option<Box<dyn ModuleCacheProvider>>,
) -> Result<ModuleSource, deno_core::error::AnyError> {
    match cache_provider.as_ref().map(|p| p.get(&module_specifier)) {
        Some(Some(source)) => return Ok(source),
        _ => {
            let module_type = if module_specifier.path().ends_with(".json") {
                ModuleType::Json
            } else {
                ModuleType::JavaScript
            };

            let response = reqwest::get(module_specifier.as_str()).await?;
            let code = response.text().await?;
            let code = transpiler::transpile(&module_specifier, &code)?;

            Ok(ModuleSource::new(
                module_type,
                ModuleSourceCode::String(code.into()),
                &module_specifier,
                None,
            ))
        }
    }
}

async fn load_from_file(
    module_specifier: &ModuleSpecifier,
    cache_provider: &Option<Box<dyn ModuleCacheProvider>>,
) -> Result<ModuleSource, deno_core::error::AnyError> {
    match cache_provider.as_ref().map(|p| p.get(&module_specifier)) {
        Some(Some(source)) => return Ok(source),
        _ => {
            let module_type = if module_specifier.path().ends_with(".json") {
                ModuleType::Json
            } else {
                ModuleType::JavaScript
            };

            let path = module_specifier.to_file_path().map_err(|_| {
                anyhow!("Provided module specifier \"{module_specifier}\" is not a file URL.")
            })?;
            let code = std::fs::read_to_string(path)?;
            let code = transpiler::transpile(&module_specifier, &code)?;

            Ok(ModuleSource::new(
                module_type,
                ModuleSourceCode::String(code.into()),
                &module_specifier,
                None,
            ))
        }
    }
}

pub struct RustyLoader {
    fs_whlist: Mutex<HashSet<String>>,
    cache_provider: Option<Box<dyn ModuleCacheProvider>>,
}
#[allow(unreachable_code)]
impl ModuleLoader for RustyLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: deno_core::ResolutionKind,
    ) -> Result<ModuleSpecifier, anyhow::Error> {
        let url = deno_core::resolve_import(specifier, &referrer)?;
        if referrer == "." {
            self.whitelist_add(url.as_str());
        }

        // We check permissions first
        match url.scheme() {
            // Remote fetch imports
            "https" | "http" => {
                #[cfg(not(feature = "url_import"))]
                return Err(anyhow!("web imports are not allowed here: {specifier}"));
            }

            // Dynamic FS imports
            "file" =>
            {
                #[cfg(not(feature = "fs_import"))]
                if !self.whitelist_has(url.as_str()) {
                    return Err(anyhow!("requested module is not loaded: {specifier}"));
                }
            }

            _ if specifier.starts_with("ext:") => {
                // Extension import - allow
            }

            _ => {
                return Err(anyhow!(
                    "unrecognized schema for module import: {specifier}"
                ));
            }
        }

        Ok(url)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleSpecifier>,
        _is_dyn_import: bool,
        _requested_module_type: deno_core::RequestedModuleType,
    ) -> deno_core::ModuleLoadResponse {
        // We check permissions first
        match module_specifier.scheme() {
            // Remote fetch imports
            #[cfg(feature = "url_import")]
            "https" | "http" => {
                let future = load_from_url(&module_specifier, &self.cache_provider);
                ModuleLoadResponse::Async(Box::pin(future))
            }

            // FS imports
            "file" => {
                let future = load_from_file(&module_specifier, &self.cache_provider);
                ModuleLoadResponse::Async(Box::pin(future))
            }

            _ => ModuleLoadResponse::Sync(Err(anyhow!(
                "{} imports are not allowed here: {}",
                module_specifier.scheme(),
                module_specifier.as_str()
            ))),
        }
    }
}

#[allow(dead_code)]
impl RustyLoader {
    pub fn new(cache_provider: Option<Box<dyn ModuleCacheProvider>>) -> Self {
        Self {
            fs_whlist: Mutex::new(Default::default()),
            cache_provider,
        }
    }

    pub fn whitelist_add(&self, specifier: &str) {
        if let Ok(mut whitelist) = self.fs_whlist.lock() {
            whitelist.insert(specifier.to_string());
        }
    }

    pub fn whitelist_has(&self, specifier: &str) -> bool {
        if let Ok(whitelist) = self.fs_whlist.lock() {
            whitelist.contains(specifier)
        } else {
            false
        }
    }
}
