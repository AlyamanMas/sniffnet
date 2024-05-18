/// This enum defines the currently displayed modal.
use super::throttling_mode::ThrottlingMode;
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MyModal {
    /// Quit modal.
    Quit,
    /// Clear all modal.
    ClearAll,
    /// Connection details modal.
    ConnectionDetails(usize),
    /// Process, port or user throttling modal which will be passed proccess id .
    ThorttlingModal(u32, ThrottlingMode),
}
