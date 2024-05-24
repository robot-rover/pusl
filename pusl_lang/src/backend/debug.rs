use super::ExecutionState;

pub enum DebugCommand {
    RunToIndex(usize),
    Run,
}

pub enum DebugResponse {
    Paused(usize),
    Done,
}

pub fn make_interrupt() -> impl FnMut(&mut ExecutionState) {
    |state| {
        println!("Idx: {}", state.current_frame.index);
    }
}