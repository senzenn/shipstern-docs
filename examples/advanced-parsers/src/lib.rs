// #region generate-events
use shipstern_proc_macro::include_shipstern_parser;

include_shipstern_parser!("padded-envelope.json");
// #endregion generate-events

// #region generated-name
impl prost::Name for padded_envelope::ProgramEventOutput {
    const NAME: &'static str = "ProgramEventOutput";
    const PACKAGE: &'static str = "padded_envelope";
}
// #endregion generated-name

// #region custom-output
use shipstern_proc_macro::shipstern;

#[shipstern]
#[derive(Clone, PartialEq)]
pub struct Transfer {
    pub amount: u64,
}

#[shipstern(oneof)]
#[derive(Clone, PartialEq)]
pub enum Action {
    Transfer(Transfer),
}

#[shipstern(enumeration)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Pending = 0,
    Finalized = 1,
}

#[shipstern]
#[derive(Clone, PartialEq)]
pub struct Update {
    #[hint(oneof = "Action", tags = "1")]
    pub action: Option<Action>,
    #[hint(enumeration = "Status")]
    pub status: i32,
}
// #endregion custom-output

// #region direct-events
pub fn events_from_logs(logs: &[String]) -> Vec<padded_envelope::Events> {
    padded_envelope::resolve_events_from_logs(logs)
}

pub fn event_from_bytes(bytes: &[u8]) -> shipstern_core::ParseResult<padded_envelope::Events> {
    padded_envelope::resolve_event_default(&[], bytes)
}
// #endregion direct-events

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use base64::{engine::general_purpose::STANDARD, Engine};
    use prost::Message;
    use shipstern_core::{
        instruction::{InstructionShared, InstructionUpdate, Path},
        Parser, Pubkey,
    };

    use super::*;

    fn event_bytes() -> Vec<u8> {
        let mut bytes = vec![0xa1, 0xa2];
        bytes.extend_from_slice(&42_u64.to_le_bytes());
        bytes
    }

    fn logs() -> Vec<String> {
        let program = Pubkey::new(padded_envelope::PROGRAM_ID);
        let encoded = STANDARD.encode(event_bytes());
        vec![
            format!("Program {program} invoke [1]"),
            format!("Program data: {encoded}"),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]".into(),
            format!("Program data: {encoded}"),
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success".into(),
            format!("Program {program} success"),
        ]
    }

    #[test]
    fn direct_events_match_wire_bytes_and_ignore_other_programs() {
        let expected = event_from_bytes(&event_bytes()).expect("event should decode");
        let padded_envelope::event::Event::AlphaEvent { args, .. } = &expected.event;
        assert_eq!(args.amount, 42);
        let events = events_from_logs(&logs());
        assert_eq!(events, vec![expected]);
        assert!(
            events_from_logs(&[format!("Program data: {}", STANDARD.encode(event_bytes()))])
                .is_empty()
        );
    }

    #[tokio::test]
    async fn padded_cpi_and_log_twin_produce_one_event() {
        let logs = logs();
        let log_end = logs.len();
        let shared = Arc::new(InstructionShared {
            log_messages: logs,
            ..Default::default()
        });
        let program = Pubkey::new(padded_envelope::PROGRAM_ID);
        let mut data = vec![0xfe, 0, 0, 0];
        data.extend_from_slice(&event_bytes());
        let inner = InstructionUpdate {
            program,
            data,
            accounts: vec![],
            inner: vec![],
            path: Path::new_single(0),
            log_range: 0..0,
            shared: Arc::clone(&shared),
        };
        let outer = InstructionUpdate {
            program,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8],
            accounts: vec![],
            inner: vec![inner],
            path: Path::new_single(0),
            log_range: 0..log_end,
            shared,
        };
        let value = padded_envelope::InstructionParser
            .parse(&outer)
            .await
            .expect("instruction should decode");
        assert_eq!(
            value.program_events,
            vec![event_from_bytes(&event_bytes()).expect("event should decode")]
        );
    }

    #[test]
    fn custom_output_round_trips_all_macro_modes() {
        let update = Update {
            action: Some(Action::Transfer(Transfer { amount: 42 })),
            status: Status::Finalized as i32,
        };
        let decoded =
            Update::decode(update.encode_to_vec().as_slice()).expect("protobuf should decode");
        assert_eq!(decoded, update);
    }

    #[test]
    fn generated_parser_has_a_named_output_matching_its_schema() {
        fn require_named_output<P: shipstern_core::ProgramParser>()
        where
            P::Output: prost::Message + prost::Name,
        {
        }
        require_named_output::<padded_envelope::InstructionParser>();
        assert_eq!(
            <padded_envelope::ProgramEventOutput as prost::Name>::full_name(),
            "padded_envelope.ProgramEventOutput"
        );
        assert!(padded_envelope::PROTOBUF_SCHEMA.contains("package padded_envelope;"));
        assert!(padded_envelope::PROTOBUF_SCHEMA.contains("message ProgramEventOutput {"));
        let any = prost_types::Any::from_msg(&padded_envelope::ProgramEventOutput::default())
            .expect("named output should encode as Any");
        assert_eq!(any.type_url, "/padded_envelope.ProgramEventOutput");
    }
}
