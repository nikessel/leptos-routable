use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};
use std::str::FromStr;

/// Holds the parsed state of a route or query parameter.
/// It may be missing, unparseable, or a valid value of type [`T`].
#[derive(Debug, PartialEq, Clone, Eq)]
pub enum ParamValue<T>
where
    T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq,
{
    /// The parameter was not found or is an empty string.
    Missing,
    /// A value was present but failed to parse as [`T`]. The inner string is the raw value.
    ParseError(String),
    /// A successfully parsed [`T`].
    Value(T),
}

impl<T> ParamValue<T>
where
    T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq,
{
    /// Returns `Some(T)` if this is a valid parsed value, or [`None`] otherwise.
    pub fn ok(self) -> Option<T> {
        match self {
            Self::Value(v) => Some(v),
            Self::Missing | Self::ParseError(_) => None,
        }
    }

    /// Returns the parsed value if valid, falling back to [`default`] otherwise.
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Value(v) => v,
            Self::Missing | Self::ParseError(_) => default,
        }
    }
}

/// A minimal error type for reporting parameter-related issues:
/// either a missing parameter or a failed parse.
#[derive(Debug, thiserror::Error)]
pub enum ParamError {
    /// Indicates that a parameter with the given name was missing or empty.
    #[error("missing param: {0}")]
    Missing(&'static str),
    /// Indicates that a parameter could not be parsed successfully.
    #[error("failed to parse param: {0}")]
    ParseError(String),
}

/// Defines a common interface for typed parameters.
/// This can be implemented by route params, query params, cookies, etc.
pub trait TypedParam<T>
where
    T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq,
{
    /// Creates a new typed param with the specified key.
    fn new(key: &'static str) -> Self;

    /// Retrieves the current [`ParamValue<T>`].
    fn get(&self) -> ParamValue<T>;

    /// Returns a [`Memo<bool>`] that is true if the param is missing or empty.
    fn is_missing(&self) -> Memo<bool>;

    /// Returns a [`Memo<bool>`] that is true if the param failed to parse.
    fn is_parse_error(&self) -> Memo<bool>;

    /// Returns a [`Memo<bool>`] that is true if the param is a valid parsed value.
    fn is_value(&self) -> Memo<bool>;

    /// Returns a [`Memo<Option<T>>`] that is `Some(T)` when valid, or [`None`] otherwise.
    fn ok(&self) -> Memo<Option<T>>;

    /// Returns a [`Memo<T>`] that either holds the parsed value or a default if missing/invalid.
    fn unwrap_or(&self, default: T) -> Memo<T>;
}

/// Generates a struct that uses a reactive [`Memo`] to track and parse
/// a particular parameter key from either [`use_params_map`] or [`use_query_map`].
macro_rules! define_typed_param_type {
    (
        $type_name:ident,
        $map_fn:path
    ) => {
        /// A reactive parameter that automatically re-parses a specified key
        /// whenever the underlying data source changes.
        #[derive(Debug, PartialEq, Clone, Eq)]
        pub struct $type_name<T>
        where
            T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq,
        {
            key: &'static str,
            memo: Memo<ParamValue<T>>,
        }

        impl<T> $type_name<T>
        where
            T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq,
        {
            /// Creates a new instance linked to the specified parameter key.
            /// The param is parsed and stored in a [`Memo`] for reactive updates.
            pub fn new(key: &'static str) -> Self {
                let map_memo = $map_fn();
                let memo = Memo::new(move |_| {
                    let raw = map_memo
                        .get()
                        .get_str(key)
                        .map(|s| s.to_string());

                    match raw {
                        None => ParamValue::Missing,
                        Some(ref s) if s.is_empty() => ParamValue::Missing,
                        Some(s) => match s.parse::<T>() {
                            Ok(parsed) => ParamValue::Value(parsed),
                            Err(_) => ParamValue::ParseError(s),
                        },
                    }
                });
                Self { key, memo }
            }

            /// Returns the current [`ParamValue<T>`].
            pub fn get(&self) -> ParamValue<T> {
                self.memo.get()
            }

            /// Returns a [`Memo<bool>`] that is true if the param is missing or empty.
            pub fn is_missing(&self) -> Memo<bool> {
                let memo = self.memo.clone();
                Memo::new(move |_| matches!(memo.get(), ParamValue::Missing))
            }

            /// Returns a [`Memo<bool>`] that is true if the param failed to parse.
            pub fn is_parse_error(&self) -> Memo<bool> {
                let memo = self.memo.clone();
                Memo::new(move |_| matches!(memo.get(), ParamValue::ParseError(_)))
            }

            /// Returns a [`Memo<bool>`] that is true if the param was parsed successfully.
            pub fn is_value(&self) -> Memo<bool> {
                let memo = self.memo.clone();
                Memo::new(move |_| matches!(memo.get(), ParamValue::Value(_)))
            }

            /// Returns a [`Memo<Option<T>>`] that is `Some(T)` if parsed, or [`None`] otherwise.
            pub fn ok(&self) -> Memo<Option<T>> {
                let memo = self.memo.clone();
                Memo::new(move |_| memo.get().clone().ok())
            }

            /// Returns a [`Memo<T>`] that either holds the parsed value or a default.
            pub fn unwrap_or(&self, default: T) -> Memo<T> {
                let memo = self.memo.clone();
                Memo::new(move |_| memo.get().clone().unwrap_or(default.clone()))
            }
        }

        impl<T> TypedParam<T> for $type_name<T>
        where
            T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq,
        {
            fn new(key: &'static str) -> Self {
                Self::new(key)
            }

            fn get(&self) -> ParamValue<T> {
                self.get()
            }

            fn is_missing(&self) -> Memo<bool> {
                self.is_missing()
            }

            fn is_parse_error(&self) -> Memo<bool> {
                self.is_parse_error()
            }

            fn is_value(&self) -> Memo<bool> {
                self.is_value()
            }

            fn ok(&self) -> Memo<Option<T>> {
                self.ok()
            }

            fn unwrap_or(&self, default: T) -> Memo<T> {
                self.unwrap_or(default)
            }
        }

        impl<T> From<&'static str> for $type_name<T>
        where
            T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq,
        {
            /// Allows creating a typed param from a string literal key.
            fn from(key: &'static str) -> Self {
                Self::new(key)
            }
        }
    };
}

// Provides typed route params and query params using the macro.
define_typed_param_type!(MaybeParam, use_params_map);
define_typed_param_type!(MaybeQuery, use_query_map);
