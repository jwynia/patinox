//! OpenAI provider implementation using async-openai crate

use super::{LLMProvider, Message, ProviderConfig, ProviderResult};

/// OpenAI provider using async-openai crate
#[derive(Debug)]
pub struct OpenAIProvider {
    client: async_openai::Client<async_openai::config::OpenAIConfig>,
    config: ProviderConfig,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider with the given configuration
    pub fn new(config: ProviderConfig) -> ProviderResult<Self> {
        // Validate that we have an API key
        let api_key = config
            .api_key
            .as_ref()
            .ok_or("OPENAI_API_KEY is required but not set")?;

        // Create async-openai config
        let openai_config = async_openai::config::OpenAIConfig::new().with_api_key(api_key);

        // Create client
        let client = async_openai::Client::with_config(openai_config);

        Ok(Self { client, config })
    }
}

#[async_trait::async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, messages: Vec<Message>) -> ProviderResult<String> {
        use async_openai::types::{
            ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
            ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
        };

        // Check for empty messages
        if messages.is_empty() {
            return Err("Cannot complete with empty messages".into());
        }

        // Convert our Message type to OpenAI's message types
        let mut openai_messages = Vec::new();
        for msg in messages {
            let openai_msg = match msg.role.as_str() {
                "system" => ChatCompletionRequestSystemMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .map(Into::into)?,
                "user" => ChatCompletionRequestUserMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .map(Into::into)?,
                "assistant" => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .map(Into::into)?,
                role => return Err(format!("Unknown message role: {}", role).into()),
            };
            openai_messages.push(openai_msg);
        }

        // Build the request
        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder
            .model(&self.config.model)
            .messages(openai_messages);

        if let Some(temp) = self.config.temperature {
            request_builder.temperature(temp);
        }

        if let Some(max_tokens) = self.config.max_tokens {
            request_builder.max_tokens(max_tokens as u32);
        }

        let request = request_builder.build()?;

        // Make the API call
        let response = self.client.chat().create(request).await?;

        // Extract the content from the first choice
        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or("No response content from OpenAI")?;

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::Provider;

    /// Test that OpenAIProvider can be created with valid configuration
    #[tokio::test]
    async fn test_openai_provider_creation() {
        // Arrange - Need to set an API key for the test
        let mut config = ProviderConfig::new(Provider::OpenAI)
            .model("gpt-4o-mini")
            .temperature(0.7)
            .max_tokens(100);

        // If no API key in environment, use a test key for provider creation test
        if config.api_key.is_none() {
            config.api_key = Some("sk-test-key-for-creation-test".to_string());
        }

        // Act
        let result = OpenAIProvider::new(config);

        // Assert
        assert!(
            result.is_ok(),
            "Should create provider with valid config including API key"
        );
    }

    /// Test that OpenAIProvider fails without API key
    #[tokio::test]
    async fn test_openai_provider_requires_api_key() {
        // Arrange - config without API key
        let mut config = ProviderConfig::new(Provider::OpenAI);
        config.api_key = None;

        // Act
        let result = OpenAIProvider::new(config);

        // Assert
        assert!(result.is_err(), "Should fail without API key");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("API key") || err_msg.contains("OPENAI_API_KEY"),
            "Error should mention API key: {}",
            err_msg
        );
    }

    /// Test that empty messages are handled
    #[tokio::test]
    async fn test_openai_empty_messages() {
        // Arrange
        let config = ProviderConfig::new(Provider::OpenAI);
        let provider = OpenAIProvider::new(config);

        if provider.is_err() {
            // If no API key, skip this test
            return;
        }

        let provider = provider.unwrap();
        let messages: Vec<Message> = vec![];

        // Act
        let result = provider.complete(messages).await;

        // Assert
        assert!(
            result.is_err(),
            "Should fail with empty messages or return error"
        );
    }
}
