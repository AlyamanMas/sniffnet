#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ThrottlingMode {
    /// Throttle by process.
    Process,
    /// Throttle by port.
    Port,
    /// Throttle by user.
    User,
}