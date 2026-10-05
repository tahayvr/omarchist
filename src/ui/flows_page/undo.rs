//! Undo and redo for the flow editor. One step back is one change to what
//! the editor holds outside its text fields: the steps, the icon, the
//! switches, and what starts the flow. The name and description are text
//! fields with an undo of their own.
use crate::system::flows::{Flow, InputFallback, OnError, Step, Triggers};

/// How many changes can be taken back.
const LIMIT: usize = 100;

/// The undoable part of a flow at one moment.
#[derive(Debug, Clone, PartialEq)]
struct Snapshot {
    steps: Vec<Step>,
    icon: String,
    on_error: OnError,
    input: InputFallback,
    triggers: Triggers,
}

impl Snapshot {
    fn of(flow: &Flow) -> Self {
        Self {
            steps: flow.steps.clone(),
            icon: flow.icon.clone(),
            on_error: flow.on_error,
            input: flow.input,
            triggers: flow.triggers.clone(),
        }
    }

    /// Whether `flow` still holds this, without copying it to find out.
    fn matches(&self, flow: &Flow) -> bool {
        self.steps == flow.steps
            && self.icon == flow.icon
            && self.on_error == flow.on_error
            && self.input == flow.input
            && self.triggers == flow.triggers
    }

    fn apply(self, flow: &mut Flow) {
        flow.steps = self.steps;
        flow.icon = self.icon;
        flow.on_error = self.on_error;
        flow.input = self.input;
        flow.triggers = self.triggers;
    }
}

/// The editor's changes, oldest first, and the ones taken back.
#[derive(Debug)]
pub(super) struct UndoStack {
    /// The flow as it was when last looked at.
    seen: Snapshot,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
}

impl UndoStack {
    pub fn new(flow: &Flow) -> Self {
        Self {
            seen: Snapshot::of(flow),
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    /// Looks at the flow. A change since the last look becomes one step to
    /// take back, and what was taken back before it can no longer return.
    /// The editor calls this whenever it changes, so no command has to
    /// remember to record itself.
    pub fn track(&mut self, flow: &Flow) {
        if self.seen.matches(flow) {
            return;
        }
        let before = std::mem::replace(&mut self.seen, Snapshot::of(flow));
        self.undo.push(before);
        if self.undo.len() > LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Takes the last change back. False when there is none.
    pub fn undo(&mut self, flow: &mut Flow) -> bool {
        self.track(flow);
        let Some(before) = self.undo.pop() else {
            return false;
        };
        self.redo
            .push(std::mem::replace(&mut self.seen, before.clone()));
        before.apply(flow);
        true
    }

    /// Makes the change last taken back again. False when there is none.
    pub fn redo(&mut self, flow: &mut Flow) -> bool {
        self.track(flow);
        let Some(after) = self.redo.pop() else {
            return false;
        };
        self.undo
            .push(std::mem::replace(&mut self.seen, after.clone()));
        after.apply(flow);
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::UndoStack;
    use crate::system::flows::{Flow, OnError, Step, StepKind};

    fn wait(ms: u64) -> Step {
        Step::new(StepKind::Wait { ms })
    }

    fn waits(flow: &Flow) -> Vec<u64> {
        flow.steps
            .iter()
            .filter_map(|step| match step.kind {
                StepKind::Wait { ms } => Some(ms),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn changes_go_back_and_come_again_in_order() {
        let mut flow = Flow::new("f".into(), "F".into());
        let mut stack = UndoStack::new(&flow);
        assert!(!stack.can_undo() && !stack.undo(&mut flow));

        flow.steps.push(wait(1));
        stack.track(&flow);
        // Looking twice at the same flow records nothing.
        stack.track(&flow);
        flow.steps.push(wait(2));
        flow.on_error = OnError::Continue;
        stack.track(&flow);

        assert!(stack.undo(&mut flow));
        assert_eq!(waits(&flow), vec![1]);
        assert_eq!(flow.on_error, OnError::Stop);
        assert!(stack.undo(&mut flow));
        assert!(flow.steps.is_empty());
        assert!(!stack.undo(&mut flow));

        assert!(stack.redo(&mut flow));
        assert!(stack.redo(&mut flow));
        assert_eq!(waits(&flow), vec![1, 2]);
        assert_eq!(flow.on_error, OnError::Continue);
        assert!(!stack.redo(&mut flow));
    }

    #[test]
    fn a_new_change_drops_what_was_taken_back() {
        let mut flow = Flow::new("f".into(), "F".into());
        let mut stack = UndoStack::new(&flow);
        flow.steps.push(wait(1));
        stack.track(&flow);
        assert!(stack.undo(&mut flow));
        // Not looked at yet: undo and redo look first.
        flow.steps.push(wait(9));
        assert!(!stack.redo(&mut flow));
        assert_eq!(waits(&flow), vec![9]);
        assert!(stack.undo(&mut flow));
        assert!(flow.steps.is_empty());
    }

    #[test]
    fn the_name_and_the_id_are_not_its_business() {
        let mut flow = Flow::new(String::new(), "F".into());
        let mut stack = UndoStack::new(&flow);
        flow.steps.push(wait(1));
        stack.track(&flow);
        // A save gives the flow its id and takes the name from its field.
        flow.id = "saved".into();
        flow.name = "Renamed".into();
        stack.track(&flow);
        assert!(stack.undo(&mut flow));
        assert_eq!((flow.id.as_str(), flow.name.as_str()), ("saved", "Renamed"));
        assert!(!stack.can_undo());
    }
}
