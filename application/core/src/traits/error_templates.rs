//! Source of API error message templates for the error-localization middleware.

use async_trait::async_trait;
use std::collections::HashMap;

/// The templates could not be loaded (unknown language, store down ...)
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct ErrorTemplateError(pub String);

/// Provides error templates for the error-localization middleware.
#[async_trait]
pub trait ErrorTemplateSource: Send + Sync {
    /// Templates of one language (`th`, `en`) as `"<API error code>"` -> `"<template>"`.
    /// A template may contain `{detail}`, replaced by the original (English) error message.
    async fn error_templates(
        &self,
        language_code: &str,
    ) -> Result<HashMap<String, String>, ErrorTemplateError>;
}
