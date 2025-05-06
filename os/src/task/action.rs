use crate::task::{SignalFlags, MAX_SIG};

/// Action for a signal
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct SignalAction {
    pub handler: usize,
    pub mask: SignalFlags,
}

impl Default for SignalAction {
    fn default() -> Self {
        Self {
            handler: 0,
            mask: SignalFlags::from_bits(40).unwrap() // QUIT & TRAP
        }
    }
}

#[derive(Clone)]
pub struct SignalActions {
    pub table: [SignalAction; MAX_SIG + 1],
}

impl Default for SignalActions {
    fn default() -> Self {
        Self {
            table: [SignalAction::default(); MAX_SIG + 1],
        }
    }
}