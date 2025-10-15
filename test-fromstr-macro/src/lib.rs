use leptos_routable::prelude::Routable;
use std::str::FromStr;

#[derive(Routable, PartialEq, Debug)]
#[routes(view_prefix = "", view_suffix = "View", transition = false)]
pub enum TestRoutes {
    #[route(path = "/")]
    Home,

    #[route(path = "/about")]
    About,

    #[route(path = "/user/:id")]
    User { id: u64 },

    #[route(path = "/post/:id")]
    Post {
        id: u64,
        comment: Option<String>,
    },

    #[fallback]
    #[route(path = "/404")]
    NotFound,
}

// Stub view functions - these won't actually be called in tests
fn HomeView() -> &'static str { "home" }
fn AboutView() -> &'static str { "about" }
fn UserView() -> &'static str { "user" }
fn PostView() -> &'static str { "post" }
fn NotFoundView() -> &'static str { "notfound" }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_home() {
        let route = TestRoutes::from_str("/").unwrap();
        assert_eq!(route, TestRoutes::Home);
    }

    #[test]
    fn test_from_str_about() {
        let route = TestRoutes::from_str("/about").unwrap();
        assert_eq!(route, TestRoutes::About);
    }

    #[test]
    fn test_from_str_user() {
        let route = TestRoutes::from_str("/user/42").unwrap();
        assert_eq!(route, TestRoutes::User { id: 42 });
    }

    #[test]
    fn test_from_str_post_no_query() {
        let route = TestRoutes::from_str("/post/123").unwrap();
        assert_eq!(route, TestRoutes::Post { id: 123, comment: None });
    }

    #[test]
    fn test_from_str_post_with_query() {
        let route = TestRoutes::from_str("/post/456?comment=hello").unwrap();
        assert_eq!(route, TestRoutes::Post { id: 456, comment: Some("hello".to_string()) });
    }

    #[test]
    fn test_from_str_unknown_fails() {
        let result = TestRoutes::from_str("/unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_with_fallback() {
        let route: TestRoutes = "/unknown/path".into();
        assert_eq!(route, TestRoutes::NotFound);
    }

    #[test]
    fn test_from_valid_route() {
        let route: TestRoutes = "/about".into();
        assert_eq!(route, TestRoutes::About);
    }
}
