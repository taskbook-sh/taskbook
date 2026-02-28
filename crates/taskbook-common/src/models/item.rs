/// Common trait for all items (tasks and notes)
pub trait Item {
    fn id(&self) -> u64;
    fn date(&self) -> &str;
    fn timestamp(&self) -> i64;
    fn description(&self) -> &str;
    fn is_starred(&self) -> bool;
    fn boards(&self) -> &[String];
    fn tags(&self) -> &[String];
    fn is_task(&self) -> bool;
}
