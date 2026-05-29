#![no_main]
#![allow(deprecated)]

// `libyml` 0.0.6 is a deprecation shim — the actual parsing is
// performed by `unsafe-libyaml`. Fuzz findings against this target
// are effectively findings against the upstream `unsafe-libyaml`
// parser; file them at
// <https://github.com/dtolnay/unsafe-libyaml/issues> rather than
// here. This target is kept so that `libyml` users running their
// own fuzz harness against the shim continue to work.

use core::mem::MaybeUninit;
use libfuzzer_sys::fuzz_target;
use libyml::{
    yaml_event_delete, yaml_parser_delete, yaml_parser_initialize,
    yaml_parser_parse, yaml_parser_set_input_string, YamlEventT,
    YamlParserT, YamlStreamEndEvent,
};

fuzz_target!(|data: &[u8]| unsafe {
    if data.len() > 10240 {
        return;
    }
    let mut parser = MaybeUninit::<YamlParserT>::uninit();
    if !yaml_parser_initialize(parser.as_mut_ptr()).ok {
        return;
    }
    let mut parser = parser.assume_init();
    yaml_parser_set_input_string(
        &mut parser,
        data.as_ptr(),
        data.len() as u64,
    );

    loop {
        let mut event = MaybeUninit::<YamlEventT>::uninit();
        let ok = yaml_parser_parse(&mut parser, event.as_mut_ptr()).ok;
        if !ok {
            break;
        }
        let event = event.assume_init();
        let type_ = event.type_;
        yaml_event_delete(&mut { event });
        if type_ == YamlStreamEndEvent {
            break;
        }
    }
    yaml_parser_delete(&mut parser);
});
