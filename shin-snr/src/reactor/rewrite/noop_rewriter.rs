use bumpalo::Bump;
use shin_versions::MessageCommandStyle;

use crate::{
    layout::message_parser::MessageReflowMode,
    reactor::{AnyLocalizableStringSource, rewrite::StringRewriter},
};

/// Exercises the string encoder and decoder, but does not actually rewrite strings.
pub struct NoopRewriter {
    snr_style: MessageCommandStyle,
    user_style: MessageCommandStyle,
}

impl NoopRewriter {
    pub fn new(snr_style: MessageCommandStyle, user_style: MessageCommandStyle) -> Self {
        Self {
            snr_style,
            user_style,
        }
    }
}

impl StringRewriter for NoopRewriter {
    fn rewrite_string<'bump>(
        &'bump self,
        bump: &'bump Bump,
        raw_decoded: &'bump str,
        _instr_index: u32,
        _instr_offset: u32,
        source: AnyLocalizableStringSource,
    ) -> Option<&'bump str> {
        let transformed = crate::layout::message_parser::transform_reflow(
            bump,
            raw_decoded,
            self.snr_style,
            MessageReflowMode::NoReflow,
            self.user_style,
            source,
        );

        Some(transformed)
    }
}
