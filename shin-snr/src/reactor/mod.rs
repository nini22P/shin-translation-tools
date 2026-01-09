use shin_versions::{AnyStringKind, LocalizableStringKind, StringArrayKind};

use crate::{
    operation::{
        OperationElementRepr,
        arena::OperationArena,
        schema::{Command, EngineSchema, Opcode, OperationSchema},
    },
    reader::Reader,
};

pub mod dump_bin;
pub mod offset_validator;
pub mod rewrite;
pub mod string_roundrip_validator;
pub mod trace;

pub trait Reactor {
    fn react(
        &mut self,
        operation_position: u32,
        raw_opcode: u8,
        opcode: Opcode,
        op_schema: &OperationSchema,
        arena: &OperationArena,
    );
    fn end_of_stream(&mut self) {}
}

pub fn react_with<R: Reactor>(mut reader: Reader, schema: &EngineSchema, reactor: &mut R) {
    let mut arena = OperationArena::new();

    while reader.has_instr() {
        let operation_position = reader.position();
        let raw_opcode = reader.take_u8();

        let Some(opcode) = schema.lookup_opcode(raw_opcode) else {
            panic!(
                "Undefined opcode: 0x{:02x} @ 0x{:08x}",
                raw_opcode, operation_position
            );
        };
        let Some(op_schema) = schema.lookup_operation(opcode) else {
            panic!(
                "Opcode with undefined schema: {:?} @ 0x{:08x}",
                opcode, operation_position
            );
        };

        // eprintln!("{:08x} {:?}", operation_position, opcode);

        op_schema.parse(schema.number_style(), &mut reader, &mut arena);

        reactor.react(operation_position, raw_opcode, opcode, &op_schema, &arena);
    }

    reactor.end_of_stream();
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnyLocalizableStringSource {
    Singular(LocalizableStringSource),
    Array(LocalizableStringArraySource, u32),
}

impl AnyLocalizableStringSource {
    pub fn kind(&self) -> AnyStringKind {
        match self {
            AnyLocalizableStringSource::Singular(s) => AnyStringKind::Singular(s.kind()),
            AnyLocalizableStringSource::Array(s, _) => AnyStringKind::Array(s.kind()),
        }
    }

    pub fn subindex(&self) -> u32 {
        match self {
            AnyLocalizableStringSource::Singular(s) => s.subindex(),
            &AnyLocalizableStringSource::Array(_, i) => i,
        }
    }

    pub fn is_for_messagebox(&self) -> bool {
        match self {
            AnyLocalizableStringSource::Singular(s) => s.is_for_messagebox(),
            AnyLocalizableStringSource::Array(s, _) => s.is_for_messagebox(),
        }
    }

    pub fn contains_commands(&self) -> bool {
        match self {
            AnyLocalizableStringSource::Singular(s) => s.contains_commands(),
            AnyLocalizableStringSource::Array(s, _) => s.contains_commands(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LocalizableStringSource {
    Saveinfo,
    Select,
    Msgset(u32),
    Dbgout,
    Logset,
    Voiceplay,
    // Game-specific string sources
    // DC4
    Chatset,
    // Alias Carnival
    Named,
    Stageinfo,
}

impl LocalizableStringSource {
    pub fn for_operation(
        opcode: Opcode,
        op_schema: &OperationSchema,
        arena: &OperationArena,
    ) -> Option<Self> {
        match opcode {
            Opcode::Instruction(_) => {
                panic!(
                    "Could not determine LocalizableStringSource for opcode {:?}",
                    opcode
                )
            }
            Opcode::Command(cmd) => match cmd {
                Command::SAVEINFO => Some(LocalizableStringSource::Saveinfo),
                Command::SELECT => Some(LocalizableStringSource::Select),
                Command::MSGSET => {
                    let OperationElementRepr::U32(first_argument) = arena
                        .iter(op_schema)
                        .next()
                        .expect("Expected MSGSET to have at least one element")
                    else {
                        panic!("Expected the first MSGSET element to be u32");
                    };
                    // the lower 24-bit are the message id
                    // the top 8 bits are some other thing
                    Some(LocalizableStringSource::Msgset(first_argument & 0xffffff))
                }
                Command::DEBUGOUT => Some(LocalizableStringSource::Dbgout),
                Command::LOGSET => Some(LocalizableStringSource::Logset),
                Command::VOICEPLAY => Some(LocalizableStringSource::Voiceplay),
                Command::CHATSET => Some(LocalizableStringSource::Chatset),
                Command::NAMED => Some(LocalizableStringSource::Named),
                Command::STAGEINFO => Some(LocalizableStringSource::Stageinfo),
                // those are not localizable because they do not get shown to the user
                Command::SPINEANM => None,
                Command::DEBUGTITLE => None,

                _ => panic!(
                    "Could not determine LocalizableStringSource for opcode {:?}",
                    opcode
                ),
            },
        }
    }

    pub fn from_kind(kind: LocalizableStringKind, subindex: u32) -> Self {
        match kind {
            LocalizableStringKind::Saveinfo => LocalizableStringSource::Saveinfo,
            LocalizableStringKind::Select => LocalizableStringSource::Select,
            LocalizableStringKind::Msgset => LocalizableStringSource::Msgset(subindex),
            LocalizableStringKind::Dbgout => LocalizableStringSource::Dbgout,
            LocalizableStringKind::Logset => LocalizableStringSource::Logset,
            LocalizableStringKind::Voiceplay => LocalizableStringSource::Voiceplay,
            LocalizableStringKind::Chatset => LocalizableStringSource::Chatset,
            LocalizableStringKind::Named => LocalizableStringSource::Named,
            LocalizableStringKind::Stageinfo => LocalizableStringSource::Stageinfo,
        }
    }

    pub fn kind(&self) -> LocalizableStringKind {
        match *self {
            LocalizableStringSource::Saveinfo => LocalizableStringKind::Saveinfo,
            LocalizableStringSource::Select => LocalizableStringKind::Select,
            LocalizableStringSource::Msgset(_) => LocalizableStringKind::Msgset,
            LocalizableStringSource::Dbgout => LocalizableStringKind::Dbgout,
            LocalizableStringSource::Logset => LocalizableStringKind::Logset,
            LocalizableStringSource::Voiceplay => LocalizableStringKind::Voiceplay,
            LocalizableStringSource::Chatset => LocalizableStringKind::Chatset,
            LocalizableStringSource::Named => LocalizableStringKind::Named,
            LocalizableStringSource::Stageinfo => LocalizableStringKind::Stageinfo,
        }
    }

    pub fn subindex(&self) -> u32 {
        match *self {
            LocalizableStringSource::Saveinfo => 0,
            LocalizableStringSource::Select => 0,
            LocalizableStringSource::Msgset(i) => i,
            LocalizableStringSource::Dbgout => 0,
            LocalizableStringSource::Logset => 0,
            LocalizableStringSource::Voiceplay => 0,
            LocalizableStringSource::Chatset => 0,
            LocalizableStringSource::Named => 0,
            LocalizableStringSource::Stageinfo => 0,
        }
    }

    pub fn is_for_messagebox(&self) -> bool {
        match self {
            LocalizableStringSource::Msgset(_) | LocalizableStringSource::Logset => true,
            _ => false,
        }
    }

    pub fn contains_commands(&self) -> bool {
        match self {
            LocalizableStringSource::Msgset(_) | LocalizableStringSource::Logset => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LocalizableStringArraySource {
    Select,
}

impl LocalizableStringArraySource {
    pub fn for_operation(
        opcode: Opcode,
        _op_schema: &OperationSchema,
        _arena: &OperationArena,
    ) -> Option<Self> {
        match opcode {
            Opcode::Instruction(_) => None,
            Opcode::Command(cmd) => match cmd {
                Command::SELECT => Some(LocalizableStringArraySource::Select),
                _ => None,
            },
        }
    }
    pub fn from_kind(kind: StringArrayKind) -> Self {
        match kind {
            StringArrayKind::SelectChoice => LocalizableStringArraySource::Select,
        }
    }

    pub fn kind(&self) -> StringArrayKind {
        match self {
            LocalizableStringArraySource::Select => StringArrayKind::SelectChoice,
        }
    }

    pub fn is_for_messagebox(&self) -> bool {
        match self {
            LocalizableStringArraySource::Select => false,
        }
    }

    pub fn contains_commands(&self) -> bool {
        match self {
            LocalizableStringArraySource::Select => true,
        }
    }
}
