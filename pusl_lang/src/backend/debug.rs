pub enum DebugCommand {
    RunToIndex(usize),
    Run
}

pub enum DebugResponse {
    Paused(usize),
    Done
}