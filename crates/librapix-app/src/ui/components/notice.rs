//! Bounded shell notice data; rendering is added by the component task.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Non-error tones are consumed by the upcoming reusable renderer.
pub enum NoticeTone {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellNotice {
    #[allow(dead_code)] // Read by the upcoming reusable notice renderer.
    pub tone: NoticeTone,
    #[allow(dead_code)] // Read by the upcoming reusable notice renderer.
    pub summary: String,
}
