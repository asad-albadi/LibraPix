#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Gallery,
    Timeline,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub active_route: Route,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_route: Route::Gallery,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMessage {
    OpenGallery,
    OpenTimeline,
}

impl AppState {
    pub fn apply(&mut self, message: AppMessage) {
        match message {
            AppMessage::OpenGallery => self.active_route = Route::Gallery,
            AppMessage::OpenTimeline => self.active_route = Route::Timeline,
        }
    }
}
