# HTML Sanitization Upgrade

## Overview

Comprehensive upgrade to HTML sanitization capabilities, enhancing security and performance while maintaining flexibility for content handling.

## Current State Analysis

### Existing Implementation Issues
- Limited XSS protection coverage
- Performance bottlenecks with large content
- Inflexible sanitization policies
- Missing CSP integration
- Inadequate test coverage for edge cases

### Security Gaps Identified
- Script injection via data attributes
- CSS-based XSS vulnerabilities
- SVG-embedded script execution
- Bypasses through HTML entity encoding
- Unicode normalization attacks

## Upgrade Strategy

### 1. Enhanced Sanitization Engine

#### Core Library Upgrade
```rust
// Upgrade to latest ammonia with custom policies
use ammonia::{Builder, clean_text, UrlRelative};

pub struct EnhancedHtmlSanitizer {
    policies: HashMap<ContentContext, Builder>,
    strict_mode: bool,
    performance_config: PerformanceConfig,
}

#[derive(Hash, PartialEq, Eq)]
pub enum ContentContext {
    UserComment,
    BlogPost,
    RichTextEditor,
    EmailTemplate,
    AdminContent,
}
```

#### Context-Aware Policies
```rust
impl EnhancedHtmlSanitizer {
    pub fn new() -> Self {
        let mut policies = HashMap::new();

        // Strict policy for user comments
        policies.insert(
            ContentContext::UserComment,
            Builder::default()
                .tags(hashset!["p", "br", "strong", "em", "u"])
                .generic_attributes(hashset!["title"])
                .rm_tags(hashset!["script", "style", "object", "embed"])
                .strip_comments(true)
        );

        // Rich content policy for blog posts
        policies.insert(
            ContentContext::BlogPost,
            Builder::default()
                .tags(hashset![
                    "p", "br", "strong", "em", "u", "h1", "h2", "h3",
                    "ul", "ol", "li", "blockquote", "a", "img"
                ])
                .tag_attributes(hashmap! {
                    "a" => hashset!["href", "title"],
                    "img" => hashset!["src", "alt", "width", "height"]
                })
                .url_relative(UrlRelative::RewriteWithBase(
                    Url::parse("https://example.com").unwrap()
                ))
        );

        Self {
            policies,
            strict_mode: false,
            performance_config: PerformanceConfig::default(),
        }
    }

    pub async fn sanitize(
        &self,
        content: &str,
        context: ContentContext,
    ) -> Result<String, SanitizationError> {
        let policy = self.policies.get(&context)
            .ok_or(SanitizationError::UnknownContext)?;

        // Pre-processing checks
        self.validate_input_size(content)?;
        self.check_encoding_attacks(content)?;

        // Apply sanitization
        let sanitized = policy.clean(content);

        // Post-processing validation
        self.validate_output(&sanitized, context).await?;

        Ok(sanitized)
    }
}
```

### 2. Advanced Security Features

#### CSS-Based XSS Prevention
```rust
impl EnhancedHtmlSanitizer {
    fn validate_css_properties(&self, css: &str) -> Result<(), SanitizationError> {
        // Block dangerous CSS properties
        let dangerous_patterns = [
            r"javascript:",
            r"expression\s*\(",
            r"@import\s+url\(",
            r"behavior\s*:",
            r"-moz-binding",
        ];

        for pattern in &dangerous_patterns {
            if regex::Regex::new(pattern)?.is_match(css) {
                return Err(SanitizationError::DangerousCssDetected);
            }
        }

        Ok(())
    }

    fn sanitize_style_attribute(&self, style: &str) -> Result<String, SanitizationError> {
        self.validate_css_properties(style)?;

        // Allow only safe CSS properties
        let safe_properties = [
            "color", "background-color", "font-size", "font-weight",
            "text-align", "margin", "padding", "border"
        ];

        let mut sanitized_rules = Vec::new();
        for rule in style.split(';') {
            if let Some(property) = rule.split(':').next() {
                let property = property.trim().to_lowercase();
                if safe_properties.contains(&property.as_str()) {
                    sanitized_rules.push(rule.trim());
                }
            }
        }

        Ok(sanitized_rules.join("; "))
    }
}
```

#### SVG Sanitization
```rust
impl EnhancedHtmlSanitizer {
    fn sanitize_svg_content(&self, svg: &str) -> Result<String, SanitizationError> {
        // Remove script elements from SVG
        let script_pattern = regex::Regex::new(r"<script[^>]*>.*?</script>")?;
        let mut cleaned = script_pattern.replace_all(svg, "").to_string();

        // Remove dangerous event handlers
        let event_pattern = regex::Regex::new(r#"on\w+\s*=\s*["'][^"']*["']"#)?;
        cleaned = event_pattern.replace_all(&cleaned, "").to_string();

        // Validate remaining SVG structure
        self.validate_svg_structure(&cleaned)?;

        Ok(cleaned)
    }

    fn validate_svg_structure(&self, svg: &str) -> Result<(), SanitizationError> {
        // Ensure SVG is well-formed and contains only safe elements
        let allowed_svg_tags = [
            "svg", "g", "path", "rect", "circle", "ellipse",
            "line", "polyline", "polygon", "text", "tspan"
        ];

        // Parse and validate SVG structure
        // Implementation would use an XML parser to verify structure
        Ok(())
    }
}
```

### 3. Performance Optimizations

#### Streaming Sanitization
```rust
use tokio::io::{AsyncRead, AsyncWrite};

impl EnhancedHtmlSanitizer {
    pub async fn sanitize_stream<R, W>(
        &self,
        mut input: R,
        mut output: W,
        context: ContentContext,
    ) -> Result<(), SanitizationError>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut buffer = vec![0; self.performance_config.buffer_size];
        let mut partial_content = String::new();

        loop {
            let bytes_read = input.read(&mut buffer).await?;
            if bytes_read == 0 {
                break; // EOF
            }

            let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);
            partial_content.push_str(&chunk);

            // Process complete HTML tags
            while let Some(tag_end) = partial_content.find('>') {
                let tag_content = partial_content.drain(..=tag_end).collect::<String>();
                let sanitized = self.sanitize_chunk(&tag_content, context)?;
                output.write_all(sanitized.as_bytes()).await?;
            }
        }

        // Process remaining content
        if !partial_content.is_empty() {
            let sanitized = self.sanitize(&partial_content, context).await?;
            output.write_all(sanitized.as_bytes()).await?;
        }

        output.flush().await?;
        Ok(())
    }
}
```

#### Caching Layer
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use lru::LruCache;

pub struct SanitizationCache {
    cache: Arc<RwLock<LruCache<SanitizationKey, String>>>,
    hit_rate_counter: AtomicU64,
    miss_rate_counter: AtomicU64,
}

#[derive(Hash, PartialEq, Eq)]
struct SanitizationKey {
    content_hash: u64,
    context: ContentContext,
    policy_version: u32,
}

impl SanitizationCache {
    pub async fn get_or_compute<F, Fut>(
        &self,
        key: SanitizationKey,
        compute_fn: F,
    ) -> Result<String, SanitizationError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<String, SanitizationError>>,
    {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.peek(&key) {
                self.hit_rate_counter.fetch_add(1, Ordering::Relaxed);
                return Ok(cached.clone());
            }
        }

        // Compute and cache result
        let result = compute_fn().await?;
        {
            let mut cache = self.cache.write().await;
            cache.put(key, result.clone());
        }

        self.miss_rate_counter.fetch_add(1, Ordering::Relaxed);
        Ok(result)
    }
}
```

### 4. Content Security Policy Integration

#### CSP Header Generation
```rust
pub struct CspGenerator {
    base_policy: ContentSecurityPolicy,
    context_policies: HashMap<ContentContext, CspModification>,
}

impl CspGenerator {
    pub fn generate_csp(
        &self,
        context: ContentContext,
        sanitized_content: &str,
    ) -> ContentSecurityPolicy {
        let mut policy = self.base_policy.clone();

        // Apply context-specific modifications
        if let Some(modification) = self.context_policies.get(&context) {
            policy.apply_modification(modification);
        }

        // Analyze content for additional restrictions
        if self.contains_inline_styles(sanitized_content) {
            policy.add_directive("style-src", "'unsafe-inline'");
        }

        if self.contains_images(sanitized_content) {
            let image_sources = self.extract_image_sources(sanitized_content);
            for source in image_sources {
                policy.add_directive("img-src", &source);
            }
        }

        policy
    }
}
```

### 5. Comprehensive Testing Strategy

#### Security Test Suite
```rust
#[cfg(test)]
mod security_tests {
    use super::*;

    #[tokio::test]
    async fn test_xss_prevention() {
        let sanitizer = EnhancedHtmlSanitizer::new();

        let xss_vectors = vec![
            "<script>alert('xss')</script>",
            "<img src=x onerror=alert('xss')>",
            "<div onclick=\"alert('xss')\">Click me</div>",
            "<style>body { background: url('javascript:alert(1)'); }</style>",
            "<svg><script>alert('xss')</script></svg>",
            "javascript:alert('xss')",
            "<iframe src=\"javascript:alert('xss')\"></iframe>",
        ];

        for vector in xss_vectors {
            let result = sanitizer.sanitize(vector, ContentContext::UserComment).await;
            assert!(result.is_ok());
            let sanitized = result.unwrap();

            // Verify no script execution possible
            assert!(!sanitized.contains("javascript:"));
            assert!(!sanitized.contains("<script"));
            assert!(!sanitized.contains("onerror="));
            assert!(!sanitized.contains("onclick="));
        }
    }

    #[tokio::test]
    async fn test_unicode_normalization_attacks() {
        let sanitizer = EnhancedHtmlSanitizer::new();

        // Test various Unicode normalization bypasses
        let unicode_vectors = vec![
            "＜script＞alert('xss')＜/script＞", // Full-width characters
            "<scr\u{0131}pt>alert('xss')</script>", // Dotless i
            "<\u{200B}script>alert('xss')</script>", // Zero-width space
        ];

        for vector in unicode_vectors {
            let result = sanitizer.sanitize(vector, ContentContext::UserComment).await;
            assert!(result.is_ok());
            let sanitized = result.unwrap();
            assert!(!sanitized.contains("script"));
        }
    }
}
```

#### Performance Test Suite
```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn benchmark_sanitization(c: &mut Criterion) {
        let sanitizer = EnhancedHtmlSanitizer::new();
        let large_content = generate_large_html_content(10_000); // 10KB

        c.bench_function("sanitize_large_content", |b| {
            b.iter(|| {
                tokio_test::block_on(sanitizer.sanitize(
                    black_box(&large_content),
                    ContentContext::BlogPost,
                ))
            })
        });
    }

    fn benchmark_streaming_sanitization(c: &mut Criterion) {
        let sanitizer = EnhancedHtmlSanitizer::new();
        let content = generate_large_html_content(100_000); // 100KB

        c.bench_function("sanitize_stream", |b| {
            b.iter(|| {
                let input = std::io::Cursor::new(content.as_bytes());
                let output = Vec::new();

                tokio_test::block_on(sanitizer.sanitize_stream(
                    input,
                    output,
                    ContentContext::BlogPost,
                ))
            })
        });
    }

    criterion_group!(benches, benchmark_sanitization, benchmark_streaming_sanitization);
    criterion_main!(benches);
}
```

## Migration Plan

### Phase 1: Infrastructure Setup
1. Update dependencies to latest ammonia version
2. Implement enhanced sanitizer with backward compatibility
3. Add comprehensive test suite
4. Set up performance monitoring

### Phase 2: Gradual Rollout
1. Deploy with feature flags for A/B testing
2. Monitor performance and security metrics
3. Gradually migrate content contexts
4. Update CSP policies

### Phase 3: Full Migration
1. Remove legacy sanitization code
2. Optimize based on production metrics
3. Document new sanitization policies
4. Train team on new security features

## Monitoring and Alerting

### Security Metrics
- XSS attempt detection rate
- Sanitization bypass attempts
- Policy violation frequency
- CSP violation reports

### Performance Metrics
- Sanitization latency percentiles
- Cache hit/miss rates
- Memory usage during processing
- Throughput under load

## Related Documentation

- [Validation TDD Methodology](../methodologies/validation-tdd-methodology.md)
- [Tower Validation Pipeline Implementation](tower-validation-pipeline-implementation.md)
- [Error-Driven Development](../methodologies/error-driven-development.md)

## Success Criteria

- **Security**: Zero XSS vulnerabilities in sanitized content
- **Performance**: <10ms latency for typical content sizes
- **Compatibility**: Backward compatibility with existing content
- **Maintainability**: Clear policies and comprehensive test coverage