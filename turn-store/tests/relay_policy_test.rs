use serde_json::json;
use turn_store::domain::event::{AgentEvent, PersistenceAction};
use turn_store::domain::model::{
    ConversationInput, CreateConversationStreamRequest, CreateTurnStreamRequest, Turn, TurnInput,
    UpstreamSpeed,
};
use uuid::Uuid;

fn valid_turn() -> TurnInput {
    TurnInput {
        input_context: "继续说明".to_string(),
        document_content_version_id: 1,
    }
}

#[test]
fn first_turn_request_validates_conversation_and_turn() {
    let request = CreateConversationStreamRequest {
        conversation: ConversationInput {
            doc_id: "doc-1".to_string(),
            doc_type: "markdown".to_string(),
            user_id: 1,
            title: "新对话".to_string(),
            r#type: "CHAT_EDIT".to_string(),
            inline_type: None,
        },
        turn: valid_turn(),
        speed: UpstreamSpeed::Fast,
    };

    assert_eq!(request.validate(), Ok(()));
}

#[test]
fn follow_up_request_does_not_require_conversation_metadata() {
    let request = CreateTurnStreamRequest {
        turn: valid_turn(),
        speed: UpstreamSpeed::Slow,
    };

    assert_eq!(request.validate(), Ok(()));
}

#[test]
fn follow_up_request_rejects_empty_input_context() {
    let request = CreateTurnStreamRequest {
        turn: TurnInput {
            input_context: "   ".to_string(),
            document_content_version_id: 1,
        },
        speed: UpstreamSpeed::Fast,
    };

    assert_eq!(
        request.validate(),
        Err("turn.input_context 不能为空".to_string())
    );
}

#[test]
fn text_event_is_appended_to_one_turn_response() {
    let event = AgentEvent::from_sse(
        "text",
        r#"{"type":"text","id":"run-1","index":0,"content":"你好"}"#,
    );

    assert_eq!(
        event.persistence_action(),
        PersistenceAction::AppendText("你好".to_string())
    );
}

#[test]
fn non_text_event_is_stored_as_raw_json() {
    let raw = r#"{"type":"status","id":"run-1","stage":"queued","content":"排队中"}"#;
    let event = AgentEvent::from_sse("status", raw);

    assert_eq!(
        event.persistence_action(),
        PersistenceAction::InsertRaw {
            event_type: "status".to_string(),
            response: raw.to_string(),
        }
    );
}

#[test]
fn malformed_text_event_is_stored_raw_instead_of_losing_data() {
    let raw = r#"{"type":"text","content":42}"#;
    let event = AgentEvent::from_sse("text", raw);

    assert_eq!(
        event.persistence_action(),
        PersistenceAction::InsertRaw {
            event_type: "text".to_string(),
            response: raw.to_string(),
        }
    );
}

#[test]
fn turn_stream_key_is_stable_and_scoped_to_the_turn() {
    let turn_id = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();

    assert_eq!(
        Turn::stream_key(turn_id),
        "turn:11111111-1111-1111-1111-111111111111:events"
    );
}

#[test]
fn turn_created_event_contains_the_domain_ids() {
    let conversation_id = Uuid::parse_str("11111111-1111-1111-1111-111111111101").unwrap();
    let turn_id = Uuid::parse_str("11111111-1111-1111-1111-111111111102").unwrap();

    assert_eq!(
        AgentEvent::turn_created(conversation_id, turn_id).data,
        json!({
            "type": "turn_created",
            "conversation_id": conversation_id,
            "turn_id": turn_id,
        })
        .to_string()
    );
}
