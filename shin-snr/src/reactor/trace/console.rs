use tracing::info;

use crate::reactor::{AnyLocalizableStringSource, trace::StringTraceListener};

pub struct ConsoleTraceListener;

impl StringTraceListener for ConsoleTraceListener {
    fn on_string(&mut self, instr_offset: u32, source: AnyLocalizableStringSource, s: &str) {
        info!("{:08x} {:?}: {}", instr_offset, source, s)
    }
}
