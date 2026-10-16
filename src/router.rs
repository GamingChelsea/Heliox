use gpui_kit::Global;
use std::collections::VecDeque;

#[derive(Clone, PartialEq, Debug)]
pub enum NavigationRoute {
    Home,
    TestPage { message: String },
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
            current: current,
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
