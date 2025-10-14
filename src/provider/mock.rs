//! Mock provider for testing (no API calls)

use super::{LLMProvider, Message, ProviderResponse, ProviderResult, ToolDefinition};

/// Mock provider that returns a pre-configured response
pub struct MockProvider {
    response: String,
}

impl MockProvider {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

#[async_trait::async_trait]
impl LLMProvider for MockProvider {
    async fn complete(
        &self,
        _messages: Vec<Message>,
        _tools: Vec<ToolDefinition>,
    ) -> ProviderResult<ProviderResponse> {
        Ok(ProviderResponse::Text(self.response.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider::new("test response");
        let result = provider
            .complete(vec![Message::user("test")], vec![])
            .await
            .unwrap();
        match result {
            ProviderResponse::Text(text) => assert_eq!(text, "test response"),
            _ => panic!("Expected text response"),
        }
    }
}
