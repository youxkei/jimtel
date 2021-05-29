use std::os::raw::c_void;
use std::sync::Arc;

use vst::editor::Editor;

use crate::params::LoudnessCeilingParams;

struct State {
    params: Arc<LoudnessCeilingParams>,
}

impl State {
    fn new(params: Arc<LoudnessCeilingParams>) -> Self {
        State { params }
    }
}

pub struct LoudnessCeilingEditor {
    opened: bool,
    params: Arc<LoudnessCeilingParams>,
}

impl LoudnessCeilingEditor {
    pub fn new(params: Arc<LoudnessCeilingParams>) -> Self {
        Self {
            params,
            opened: false,
        }
    }
}

impl Editor for LoudnessCeilingEditor {
    fn size(&self) -> (i32, i32) {
        (1024, 360)
    }

    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn open(&mut self, parent: *mut c_void) -> bool {
        false
    }

    fn is_open(&mut self) -> bool {
        false
    }

    fn close(&mut self) {}
}
