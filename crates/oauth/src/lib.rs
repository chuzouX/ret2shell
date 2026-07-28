pub mod phira;
pub mod traits;
mod utility;
use std::collections::HashMap;

use r2s_config::auth::Config;
use r2s_engine::{DiagnosticMarker, Engine, EngineError};
use rune::{Any, ContextError, Module, Value, runtime::Object};
pub use traits::OAuthError;

#[derive(Clone, Debug, Any)]
#[rune(item = ::ret2shell::oauth)]
pub struct RuneMap(pub HashMap<String, String>);

impl RuneMap {
  #[rune::function(path = Self::get)]
  pub fn get(&self, key: &str) -> Option<String> {
    self.0.get(key).cloned()
  }
}

#[rune::module(::ret2shell::oauth)]
pub fn module(_stdio: bool) -> Result<Module, ContextError> {
  let mut module = Module::from_meta(self::module_meta)?;
  module.ty::<RuneMap>()?;
  module.function_meta(RuneMap::get)?;
  Ok(module)
}

#[derive(Debug, Clone, Default)]
pub struct OAuth;

impl OAuth {
  fn default_modules() -> Vec<fn(bool) -> Result<rune::Module, rune::ContextError>> {
    vec![
      rune_modules::http::module,
      rune_modules::json::module,
      rune_modules::toml::module,
      rune_modules::process::module,
      utility::xml::module,
      module,
    ]
  }

  pub async fn expire(&self, engine: &Engine, key: impl AsRef<str>) {
    engine.expire(format!("oauth-{}", key.as_ref())).await;
  }

  pub async fn preload(
    &self, engine: &Engine, key: impl AsRef<str>, script: impl AsRef<str>,
  ) -> Result<(), EngineError> {
    let key = format!("oauth-{}", key.as_ref());
    engine
      .preload(Self::default_modules(), key, script, None)
      .await
  }

  pub async fn lint(&self, script: impl AsRef<str>) -> Result<Vec<DiagnosticMarker>, EngineError> {
    Engine::lint(Self::default_modules(), script, &["login", "bind"]).await
  }

  pub async fn login(
    &self, engine: &Engine, key: impl AsRef<str>, params: &HashMap<String, String>,
  ) -> Result<HashMap<String, String>, OAuthError> {
    let key = key.as_ref();
    let key = format!("oauth-{}", key);
    let params_object = RuneMap(params.clone());
    let result = engine.execute(key, "login", (params_object,)).await?;
    let output: Result<Object, Value> = rune::from_value(result).map_err(EngineError::from)?;
    if let Ok(object) = output {
      let _ = object
        .get("auth_key")
        .ok_or_else(|| OAuthError::MissingField("auth_key".to_owned()))?;
      let mut data: HashMap<String, String> = HashMap::new();
      for (key, value) in object.iter() {
        data.insert(
          key.to_string(),
          rune::from_value(value.clone()).map_err(EngineError::from)?,
        );
      }
      Ok(data)
    } else {
      Err(OAuthError::ScriptError(
        "unexpected value in oauth script".to_owned(),
      ))
    }
  }

  pub async fn bind(
    &self, engine: &Engine, key: impl AsRef<str>, params: &HashMap<String, String>,
    user: &HashMap<String, String>,
  ) -> Result<HashMap<String, String>, OAuthError> {
    let key = key.as_ref();
    let key = format!("oauth-{}", key);
    let params_object = RuneMap(params.clone());
    let user_object = RuneMap(user.clone());
    let result = engine
      .execute(key, "bind", (params_object, user_object))
      .await?;
    let output: Result<Object, Value> = rune::from_value(result).map_err(EngineError::from)?;
    if let Ok(object) = output {
      let _ = object
        .get("auth_key")
        .ok_or_else(|| OAuthError::MissingField("auth_key".to_owned()))?;
      let mut data: HashMap<String, String> = HashMap::new();
      for (key, value) in object.iter() {
        data.insert(
          key.to_string(),
          rune::from_value(value.clone()).map_err(EngineError::from)?,
        );
      }
      Ok(data)
    } else {
      Err(OAuthError::ScriptError(
        "unexpected value in oauth script".to_owned(),
      ))
    }
  }
}

pub async fn initialize(_config: &Option<Config>) -> OAuth {
  OAuth
}
