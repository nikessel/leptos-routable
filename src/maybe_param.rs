use leptos::prelude::*;
use std::str::FromStr;
use leptos_router::hooks::{use_params_map, use_query_map};

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum ParamValue<T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq> {
    Missing,
    ParseError(String),
    Value(T),
}

impl<T> ParamValue<T>
where
    T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq,
{
    pub fn ok(self) -> Option<T> {
        match self {
            ParamValue::Value(v) => Some(v),
            ParamValue::Missing | ParamValue::ParseError(_) => None,
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            ParamValue::Value(v) => v,
            ParamValue::Missing | ParamValue::ParseError(_) => default,
        }
    }
}

macro_rules! define_typed_param_type {
    (
        $type_name:ident,
        $map_fn:path
    ) => {
        #[derive(Debug, PartialEq, Clone, Eq)]
        pub struct $type_name<T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq> {
            key: &'static str,
            memo: Memo<ParamValue<T>>,
        }

        impl<T> $type_name<T>
        where
            T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq,
        {
            pub fn new(key: &'static str) -> Self {
                let map_memo = $map_fn();

                let memo = Memo::new(move |_| {

                    let raw: Option<String> = map_memo
                        .get()
                        .get_str(key)
                        .map(|s| s.to_string());

                    match raw {
                        None => ParamValue::Missing,
                        Some(ref s) if s.is_empty() => ParamValue::Missing,
                        Some(s) => match s.parse::<T>() {
                            Ok(parsed) => ParamValue::Value(parsed),
                            Err(_) => ParamValue::ParseError(s),
                        }
                    }
                });

                Self { key, memo }
            }

            pub fn get(&self) -> ParamValue<T> {
                self.memo.get()
            }

            pub fn is_missing(&self) -> Memo<bool> {
                let memo = self.memo.clone();
                Memo::new(move |_| matches!(memo.get(), ParamValue::Missing))
            }

            pub fn is_parse_error(&self) -> Memo<bool> {
                let memo = self.memo.clone();
                Memo::new(move |_| matches!(memo.get(), ParamValue::ParseError(_)))
            }

            pub fn is_value(&self) -> Memo<bool> {
                let memo = self.memo.clone();
                Memo::new(move |_| matches!(memo.get(), ParamValue::Value(_)))
            }

            pub fn ok(&self) -> Memo<Option<T>> {
                let memo = self.memo.clone();
                Memo::new(move |_| memo.get().clone().ok())
            }

            pub fn unwrap_or(&self, default: T) -> Memo<T> {
                let memo = self.memo.clone();
                Memo::new(move |_| memo.get().clone().unwrap_or(default.clone()))
            }
        }

        impl<T> From<&'static str> for $type_name<T>
        where
            T: FromStr + Send + Clone + Sync + 'static + PartialEq + Eq,
        {
            fn from(key: &'static str) -> Self {
                $type_name::new(key)
            }
        }
    };
}

define_typed_param_type!(MaybeParam, use_params_map);
define_typed_param_type!(MaybeQuery, use_query_map);
