use crate::application::wire::{BackendMessage, ErrorResponse};

/// Builds the temporary transaction-pooling boundary response while complete
/// extended-protocol support remains a separate work item. Both transaction
/// engines use this exact frame so clients receive one stable diagnostic.
pub(crate) fn extended_query_rejection() -> BackendMessage {
    BackendMessage::ErrorResponse(ErrorResponse {
        fields: vec![
            (b'S', "FATAL".to_string()),
            (b'C', "0A000".to_string()),
            (
                b'M',
                "extended query protocol not yet supported in transaction pooling mode".to_string(),
            ),
        ],
    })
}
