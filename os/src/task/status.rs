#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

impl TaskStatus {
    pub(super) fn init() -> Self {
        TaskStatus::UnInit
    }
}
