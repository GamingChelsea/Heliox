use gpui_kit::{Global, assets::IconName};
use std::collections::VecDeque;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Clone, Copy, Debug)]
pub struct RouteMeta {
    pub name: &'static str,
    pub icon: IconName,
    pub no_input: bool,
}

#[derive(Clone, PartialEq, Debug, EnumIter)]
pub enum NavigationRoute {
    Home,
    LocalLibrary,
    TestPage { message: String },
    Settings,
}
impl NavigationRoute {
    // Get all routes which don't require a input
    pub fn get_clean_routes() -> Vec<NavigationRoute> {
        Self::iter().filter(|route| route.meta().no_input).collect()
    }

    pub const fn meta(&self) -> RouteMeta {
        match self {
            NavigationRoute::Home => RouteMeta {
                name: "Home",
                icon: IconName::House,
                no_input: true,
            },
            NavigationRoute::LocalLibrary => RouteMeta {
                name: "Local Library",
                icon: IconName::Library,
                no_input: true,
            },
            NavigationRoute::TestPage { .. } => RouteMeta {
                name: "Test Page",
                icon: IconName::TestTube,
                no_input: false,
            },
            NavigationRoute::Settings => RouteMeta {
                name: "Settings",
                icon: IconName::Settings,
                no_input: true,
            },
        }
    }
}

#[derive(Debug)]
pub struct NavigationHistory {
    pub history: VecDeque<NavigationRoute>,
    pub current: NavigationRoute,
    pub future: VecDeque<NavigationRoute>,
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

impl Global for NavigationHistory {}
