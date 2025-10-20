use leptos_routable::prelude::*;

pub mod routes;

#[derive(Routable, Clone, PartialEq, Eq, Debug)]
#[routes(
    state_suffix = "State",
    module_organization = "routes",
    transition = false
)]
pub enum AppRoutes {
    #[route(path = "/")]
    Index,

    #[parent_route(path = "/dashboard")]
    Dashboard(DashboardRoutes),

    #[fallback]
    #[route(path = "/404")]
    NotFound,
}

#[derive(Routable, Clone, PartialEq, Eq, Debug)]
#[routes(
    module_organization = "routes/dashboard/sub_routes",
    transition = false
)]
pub enum DashboardRoutes {
    #[route(path = "/settings")]
    Settings,
}
