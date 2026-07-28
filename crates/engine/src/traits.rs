use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
  #[error("io error: {0}")]
  IoError(#[from] std::io::Error),
  #[error("missing script: {0}")]
  MissingScript(String),
  #[error("rune context error: {0}")]
  RuneError(#[from] rune::ContextError),
  #[error("rune runtime error: {0}")]
  RuneRuntimeError(#[from] rune::runtime::RuntimeError),
  #[error("can not load script source: {0}")]
  SourceError(#[from] rune::source::FromPathError),
  #[error("can not build script unit: {0}")]
  BuildError(#[from] rune::BuildError),
  #[error("can not alloc script engine runtime: {0}")]
  AllocError(#[from] rune::alloc::Error),
  #[error("executed script error: {0}")]
  ExecError(#[from] rune::runtime::VmError),
  #[error("missing fields from script result: {0}")]
  MissingResultField(String),
  #[error("script error: {0}")]
  ScriptError(String),
  #[error("compile error: {0}")]
  CompileError(String),
  #[error("missing function: {0}")]
  MissingFunction(String),
  #[error("string UTF-8 decode error: {0}")]
  FromUtf8Error(#[from] std::string::FromUtf8Error),
}

/// DiagnosticMarker for rune scripts
/// Adapted from the upstream cdsctf Rune implementation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DiagnosticMarker {
  pub kind: DiagnosticKind,
  pub message: String,
  pub start_line: usize,
  pub start_column: usize,
  pub end_line: usize,
  pub end_column: usize,
}

/// DiagnosticKind for rune scripts
/// Adapted from the upstream cdsctf Rune implementation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
  Error,
  Warning,
}
