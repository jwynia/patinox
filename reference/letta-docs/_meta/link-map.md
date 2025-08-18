# Link Map

> Cross-reference map showing which files reference which concepts

## Overview

This document provides a visual map of how concepts and files are interconnected throughout the agent-optimized documentation. Use this to understand relationships and find alternative paths to information.

## Core Concept Reference Map

### Agent → Referenced By
- [Quick Start: Minimal Example](../00-quick-start/minimal-example.md) - Primary agent creation example
- [Core Concepts: Key Abstractions](../01-core-concepts/key-abstractions.md) - Agent entity definition
- [API Reference: Agents](../02-api-reference/agents/index.md) - Complete API documentation
- [Patterns: Common Use Cases](../03-patterns/common-use-cases/index.md) - Real-world implementations
- [Gotchas: Common Mistakes](../05-gotchas/common-mistakes.md) - Agent design anti-patterns

### Memory → Referenced By
- [Core Concepts: Key Abstractions](../01-core-concepts/key-abstractions.md) - Memory system overview
- [Core Concepts: Data Flow](../01-core-concepts/data-flow.md) - Memory processing flow
- [API Reference: Agents](../02-api-reference/agents/index.md) - Memory management operations
- [Gotchas: Common Mistakes](../05-gotchas/common-mistakes.md) - Memory management pitfalls

### Tools → Referenced By  
- [Quick Start: Minimal Example](../00-quick-start/minimal-example.md) - Basic tool usage
- [Quick Start: Cheatsheet](../00-quick-start/cheatsheet.md) - Tool operations reference
- [Core Concepts: Key Abstractions](../01-core-concepts/key-abstractions.md) - Tool system architecture
- [Patterns: Common Use Cases](../03-patterns/common-use-cases/index.md) - Tool integration patterns
- [Gotchas: Common Mistakes](../05-gotchas/common-mistakes.md) - Tool development mistakes

### Server/Infrastructure → Referenced By
- [Quick Start: Installation](../00-quick-start/installation.md) - Server setup
- [Quick Start: Prerequisites](../00-quick-start/prerequisites.md) - Infrastructure requirements
- [Core Concepts: Architecture Overview](../01-core-concepts/architecture-overview.md) - Server architecture

## File Cross-Reference Matrix

### Quick Start Section Interconnections
```
installation.md ─────┐
                     ├──→ minimal-example.md ─────┐
prerequisites.md ────┘                           ├──→ cheatsheet.md
                                                 │
import-patterns.md ──────────────────────────────┘
```

**Link Density**: High (each file links to 2-3 others in section)

### Core Concepts Section Interconnections
```
architecture-overview.md ──┬──→ key-abstractions.md ──┬──→ mental-model.md
                           │                           │
                           └──→ data-flow.md ──────────┼──→ terminology.md
                                                       │
                                                       └──→ glossary.md
```

**Link Density**: Very High (extensive cross-referencing)

### API Reference to Patterns Flow
```
02-api-reference/agents/index.md ───┐
                                    ├──→ 03-patterns/common-use-cases/index.md
02-api-reference/index.md ──────────┘
```

**Link Density**: Medium (foundational links established)

## Concept Learning Paths

### Beginner Path
1. [Installation](../00-quick-start/installation.md) → 
2. [Minimal Example](../00-quick-start/minimal-example.md) → 
3. [Key Abstractions](../01-core-concepts/key-abstractions.md) → 
4. [Mental Model](../01-core-concepts/mental-model.md) → 
5. [Common Use Cases](../03-patterns/common-use-cases/index.md)

### Developer Path
1. [Prerequisites](../00-quick-start/prerequisites.md) → 
2. [Import Patterns](../00-quick-start/import-patterns.md) → 
3. [Architecture Overview](../01-core-concepts/architecture-overview.md) → 
4. [Agents API](../02-api-reference/agents/index.md) → 
5. [Common Mistakes](../05-gotchas/common-mistakes.md)

### Integration Path
1. [Installation](../00-quick-start/installation.md) → 
2. [Architecture Overview](../01-core-concepts/architecture-overview.md) → 
3. [API Reference](../02-api-reference/index.md) → 
4. [Integration Patterns](../03-patterns/common-use-cases/index.md) → 
5. [Prerequisites](../00-quick-start/prerequisites.md)

## External Link Strategy

### Official Letta Resources
- **Primary Documentation**: https://docs.letta.com
- **GitHub Repository**: https://github.com/letta-ai/letta  
- **Agent Development Environment**: https://app.letta.com
- **Python Package**: https://pypi.org/project/letta/

### Community Resources
- **Discord Community**: https://discord.gg/letta
- **Twitter/X Updates**: https://twitter.com/Letta_AI
- **Research Paper**: https://arxiv.org/abs/2310.08560

### Technical Resources
- **Docker Images**: https://hub.docker.com/r/letta/letta
- **OpenAPI Docs**: http://localhost:8283/docs (when server running)
- **Client SDK**: https://github.com/letta-ai/letta-client-python

## Link Validation

### Internal Links (Relative Paths)
- ✅ All quick-start files have working cross-references
- ✅ Core concepts section fully interconnected
- ✅ API reference links to patterns established
- ⏳ Pattern examples link back to API reference (partial)
- ❌ Advanced sections not yet created

### External Links (Absolute URLs)
- ✅ Official Letta resources verified
- ✅ Community links active
- ✅ Package manager links current
- ⚠️ Local server links require running server

## Orphaned Content Detection

### Files Without Incoming Links
- None detected in current structure

### Files Without Outgoing Links
- `_meta/validation.md` (by design - terminal document)
- `_meta/link-map.md` (by design - meta document)

### Broken Link Candidates
- Advanced section links (directories exist but no content yet)
- Some specific API method references (detailed pages not created)

## Navigation Optimization

### Hub Pages (High Link Density)
1. **[Core Concepts Index](../01-core-concepts/key-abstractions.md)** - Central concept definitions
2. **[API Reference Index](../02-api-reference/index.md)** - Technical documentation hub  
3. **[Common Use Cases](../03-patterns/common-use-cases/index.md)** - Practical examples hub
4. **[Cheatsheet](../00-quick-start/cheatsheet.md)** - Quick reference hub

### Bridge Documents (Connect Sections)
1. **[Mental Model](../01-core-concepts/mental-model.md)** - Concepts ↔ Patterns
2. **[Architecture Overview](../01-core-concepts/architecture-overview.md)** - Concepts ↔ API
3. **[Common Mistakes](../05-gotchas/common-mistakes.md)** - Patterns ↔ Troubleshooting

### Terminal Documents (Few Outgoing Links)
1. **[Glossary](../01-core-concepts/glossary.md)** - Reference destination
2. **[Cheatsheet](../00-quick-start/cheatsheet.md)** - Quick reference destination
3. **[Minimal Example](../00-quick-start/minimal-example.md)** - Getting started destination

## Link Maintenance Strategy

### Regular Validation
- **Monthly**: Check all external links for availability
- **Per Release**: Validate API reference links against current version
- **Per Addition**: Ensure new content integrates with link map

### Link Standards
- **Internal Links**: Use relative paths for maintainability
- **External Links**: Mark with 🔗 emoji for visual identification
- **Section Links**: Always include section context in link text
- **Deep Links**: Link to specific headings when relevant

### Expansion Guidelines
- **New Sections**: Must connect to at least 2 existing sections
- **New Files**: Must reference and be referenced by related content
- **API Documentation**: Must link to usage examples
- **Examples**: Must link back to API documentation

## See Also

- [Learning Path](learning-path.md) - Suggested reading order for different goals
- [Search Index](search-index.md) - Keywords/synonyms mapping
- [Decision Tree](decision-tree.md) - If you want to do X, read Y