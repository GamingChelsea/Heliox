use nsh_core::prelude::*;
use std::collections::VecDeque;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Clone, Copy, Debug)]
pub struct RouteMeta {
    pub name: &'static str,
    pub no_input: bool,
}

#[derive(Clone, PartialEq, Debug, EnumIter)]
pub enum NavigationRoute {
    Home,
    Search,
    LocalLibrary,
    TestPage { message: String },
    Settings,
}

impl NavigationRoute {
    pub fn get_clean_routes() -> Vec<NavigationRoute> {
        Self::iter().filter(|route| route.meta().no_input).collect()
    }

    pub fn meta(&self) -> RouteMeta {
        match self {
            NavigationRoute::Home => RouteMeta {
                name: "Home",
                no_input: true,
            },
            NavigationRoute::Search => RouteMeta {
                name: "Search",
                no_input: true,
            },
            NavigationRoute::LocalLibrary => RouteMeta {
                name: "Local Library",
                no_input: true,
            },
            NavigationRoute::TestPage { .. } => RouteMeta {
                name: "Test Page",
                no_input: false,
            },
            NavigationRoute::Settings => RouteMeta {
                name: "Settings",
                no_input: true,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavigationHistory {
    pub history: VecDeque<NavigationRoute>,
    pub current: NavigationRoute,
    pub future: VecDeque<NavigationRoute>,
}

impl Default for NavigationHistory {
    fn default() -> Self {
        Self {
            history: VecDeque::new(),
            current: NavigationRoute::Home,
            future: VecDeque::new(),
        }
    }
}

impl NavigationHistory {
    pub fn new(current: NavigationRoute) -> Self {
        Self {
            history: VecDeque::new(),
            current,
            future: VecDeque::new(),
        }
    }

    pub fn current_route(&self) -> &NavigationRoute {
        &self.current
    }

    pub fn push(&mut self, route: NavigationRoute) {
        if self.current == route {
            return;
        }
        self.history.push_front(self.current.clone());
        self.current = route;
        self.future.clear();
    }

    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn go_back(&mut self) {
        if let Some(prev) = self.history.pop_front() {
            self.future.push_back(self.current.clone());
            self.current = prev;
        }
    }

    pub fn go_forward(&mut self) {
        if let Some(next) = self.future.pop_back() {
            self.history.push_front(self.current.clone());
            self.current = next;
        }
    }
}

pub fn navigate_to(route: NavigationRoute) {
    if route == NavigationRoute::LocalLibrary {
        crate::library::load_local_library();
    }
    global::<NavigationHistory>().update(|nav| nav.push(route));
}

pub fn go_back() {
    global::<NavigationHistory>().update(|nav| nav.go_back());
}

pub fn go_forward() {
    global::<NavigationHistory>().update(|nav| nav.go_forward());
}
