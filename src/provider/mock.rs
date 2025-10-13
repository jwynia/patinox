//! Mock provider for testing (no API calls)

use super::{LLMProvider, Message, ProviderResult};

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
    async fn complete(&self, _messages: Vec<Message>) -> ProviderResult<String> {
        Ok(self.response.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider::new("test response");
        let result = provider
            .complete(vec![Message::user("test")])
            .await
            .unwrap();
        assert_eq!(result, "test response");
    }
}
