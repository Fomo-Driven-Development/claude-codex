use serde::Serialize;

/// User can configure a program that will receive notifications. Each
/// notification is serialized as JSON and passed as an argument to the
/// program.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub(crate) enum UserNotification {
    #[serde(rename_all = "kebab-case")]
    AgentTurnComplete {
        turn_id: String,

        /// Messages that the user sent to the agent to initiate the turn.
        input_messages: Vec<String>,

        /// The last message sent by the assistant in the turn.
        last_assistant_message: Option<String>,
    },
    #[serde(rename_all = "kebab-case")]
    AgentTurnStopped {
        turn_id: String,
        session_id: String,
        cwd: String,
        input_messages: Vec<String>,
        last_assistant_message: Option<String>,
    },

    #[serde(rename_all = "kebab-case")]
    ToolPermissionRequest {
        session_id: String,
        cwd: String,
        approval_type: String,
        tool_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        command: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        changes: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    #[serde(rename_all = "kebab-case")]
    PromptIdleTimeout {
        session_id: String,
        cwd: String,
        idle_duration_seconds: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_user_notification() {
        let notification = UserNotification::AgentTurnComplete {
            turn_id: "12345".to_string(),
            input_messages: vec!["Rename `foo` to `bar` and update the callsites.".to_string()],
            last_assistant_message: Some(
                "Rename complete and verified `cargo build` succeeds.".to_string(),
            ),
        };
        let serialized = serde_json::to_string(&notification).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"agent-turn-complete","turn-id":"12345","input-messages":["Rename `foo` to `bar` and update the callsites."],"last-assistant-message":"Rename complete and verified `cargo build` succeeds."}"#
        );
    }

    #[test]
    fn test_agent_turn_stopped_notification() {
        let notification = UserNotification::AgentTurnStopped {
            turn_id: "12345".to_string(),
            session_id: "abc123".to_string(),
            cwd: "/home/user/project".to_string(),
            input_messages: vec!["Fix the authentication bug".to_string()],
            last_assistant_message: Some("I've fixed the authentication issue.".to_string()),
        };
        let serialized = serde_json::to_string(&notification).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"agent-turn-stopped","turn-id":"12345","session-id":"abc123","cwd":"/home/user/project","input-messages":["Fix the authentication bug"],"last-assistant-message":"I've fixed the authentication issue."}"#
        );
    }

    #[test]
    fn test_tool_permission_request_serialization() {
        let notification = UserNotification::ToolPermissionRequest {
            session_id: "sess-42".to_string(),
            cwd: "/tmp/project".to_string(),
            approval_type: "exec".to_string(),
            tool_name: "Bash".to_string(),
            command: Some("ls -la".to_string()),
            changes: None,
            reason: Some("Inspect directory".to_string()),
        };

        let serialized = serde_json::to_string(&notification).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"tool-permission-request","session-id":"sess-42","cwd":"/tmp/project","approval-type":"exec","tool-name":"Bash","command":"ls -la","reason":"Inspect directory"}"#
        );
    }

    #[test]
    fn test_prompt_idle_timeout_serialization() {
        let notification = UserNotification::PromptIdleTimeout {
            session_id: "sess-123".to_string(),
            cwd: "/tmp".to_string(),
            idle_duration_seconds: 120,
        };

        let serialized = serde_json::to_string(&notification).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"prompt-idle-timeout","session-id":"sess-123","cwd":"/tmp","idle-duration-seconds":120}"#
        );
    }
}
