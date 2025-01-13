use leptos_router::{hooks::{
    use_navigate,
    // query_signal, query_signal_with_options,
    // use_location, use_params,
    // , use_params_map, use_query_map, use_query,
    // use_matched,
}, params::{ParamsError}, location::Location, NavigateOptions};
use std::str::FromStr;
use leptos::prelude::{Get, Memo, Set, With};
use leptos::reactive::wrappers::write::SignalSetter;

pub trait IntoPath {
    fn into_path(self) -> String;
}

pub trait NavigateExt: Fn(&str, NavigateOptions) {
    fn navigate<P: IntoPath>(&self, pathlike: P) {
        (self)(&pathlike.into_path(), NavigateOptions::default());
    }

    fn navigate_with_options<P: IntoPath>(&self, pathlike: P, options: NavigateOptions) {
        (self)(&pathlike.into_path(), options);
    }
}

impl<F> NavigateExt for F
where
    F: Fn(&str, NavigateOptions),
{}

pub trait QuerySignalSetterExt<T> {
    fn set_path<P: IntoPath>(&self, pathlike: P);
}

impl<T> QuerySignalSetterExt<T> for SignalSetter<Option<T>>
where
    T: FromStr + ToString + PartialEq + Send + Sync + 'static,
{
    fn set_path<P: IntoPath>(&self, pathlike: P) {
        let path_str = pathlike.into_path();
        if let Ok(converted) = path_str.parse::<T>() {
            self.set(Some(converted));
        } else {
            self.set(None);
        }
    }
}

pub trait LocationExt {
    fn current_path(&self) -> String;
    fn set_path<P: IntoPath>(&self, pathlike: P);
}

impl LocationExt for Location {
    fn current_path(&self) -> String {
        self.pathname.get()
    }

    fn set_path<P: IntoPath>(&self, pathlike: P) {
        let navigate = use_navigate();
        navigate(&pathlike.into_path(), NavigateOptions::default());
    }
}

pub trait ParamsMemoExt<T: Send + Sync + 'static> {
    fn unwrap_params(self) -> Memo<Option<T>>;
}

impl<T> ParamsMemoExt<T> for Memo<Result<T, ParamsError>>
where
    T: Clone + 'static + Send + Sync + PartialEq,
{
    fn unwrap_params(self) -> Memo<Option<T>> {
        Memo::new(move |_| self.with(|r| r.clone().ok()))
    }
}

pub trait MatchedMemoExt {
    fn matched_path(self) -> Memo<String>;
}

impl MatchedMemoExt for Memo<String> {
    fn matched_path(self) -> Memo<String> {
        self
    }
}
