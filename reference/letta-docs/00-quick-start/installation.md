# Installation

> Quick installation guide for Letta - an open source framework for building stateful agents with long-term memory

## Package Managers

### pip (Recommended for Development)
```bash
pip install -U letta
```

**Note**: pip installation defaults to SQLite database backend. For production use, consider Docker installation with PostgreSQL.

### Docker (Recommended for Production)
```bash
# Basic installation with PostgreSQL
docker run \
  -v ~/.letta/.persist/pgdata:/var/lib/postgresql/data \
  -p 8283:8283 \
  -e OPENAI_API_KEY="your_openai_api_key" \
  letta/letta:latest
```

### Poetry (For Contributors)
```bash
# Install Poetry from https://python-poetry.org/docs/#installation
git clone https://github.com/letta-ai/letta.git
cd letta
poetry install --all-extras
poetry shell
```

## Version Requirements

- **Python**: 3.11 - 3.13 (3.14 not yet supported)
- **PostgreSQL**: 12+ (for production deployments)
- **Docker**: 20.10+ (for containerized deployments)

## System Dependencies

### Required
- Python development headers
- SQLite 3.35+ (for development)
- Vector extension support (pgvector for PostgreSQL)

### Optional Dependencies
- **PostgreSQL**: For production database backend
- **Redis**: For caching and session management
- **Docker**: For containerized deployment
- **E2B/Modal**: For sandboxed tool execution

## Database Backend Selection

| Installation Method | Database | Migration Support | Use Case |
|---|---|---|---|
| `pip install letta` | SQLite | ❌ | Development, testing |
| `pip + LETTA_PG_URI` | PostgreSQL | ✅ | Development with production DB |
| Docker | PostgreSQL | ✅ | Production deployment |

## Environment Variables

### LLM Provider Keys
```bash
# OpenAI (most common)
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# Local models
export OLLAMA_BASE_URL="http://localhost:11434"
```

### Database Configuration
```bash
# PostgreSQL connection (optional for pip installs)
export LETTA_PG_URI="postgresql://user:password@localhost:5432/letta"

# Server security (for production)
export SECURE=true
export LETTA_SERVER_PASSWORD="your_secure_password"
```

## Quick Installation Test

After installation, verify everything works:

```bash
# Start server
letta server

# In another terminal, test client
python -c "
from letta_client import Letta
client = Letta(base_url='http://localhost:8283')
print('✅ Installation successful!')
"
```

## Troubleshooting

### Common Issues

**"ModuleNotFoundError: No module named 'letta'"**
- Ensure you're in the correct Python environment
- Try `pip install --upgrade letta`

**"Connection refused on port 8283"**
- Check if server is running: `ps aux | grep letta`
- Verify port availability: `netstat -an | grep 8283`

**Database migration errors**
- For SQLite: Delete `~/.letta/` and restart
- For PostgreSQL: Run `alembic upgrade head`

## Next Steps

- [Minimal Example](minimal-example.md) - Get your first agent running
- [Import Patterns](import-patterns.md) - How to import Letta in your code
- [Prerequisites](prerequisites.md) - Understanding the requirements

## See Also

- [🔗 Official Installation Guide](https://docs.letta.com/install)
- [Docker Setup](../04-integration/deployment.md)
- [Environment Configuration](../01-core-concepts/architecture-overview.md)