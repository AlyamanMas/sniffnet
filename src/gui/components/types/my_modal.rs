/// This enum defines the currently displayed modal.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MyModal {
    /// Quit modal.
    Quit,
    /// Clear all modal.
    ClearAll,
    /// Connection details modal.
    ConnectionDetails(usize),
    /// Process throttling modal which will be passed proccess id .
    ProcessThrottling(u32), //usize is the process id
}
