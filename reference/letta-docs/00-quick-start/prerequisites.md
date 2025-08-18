# Prerequisites

> Environment setup, required knowledge, and dependencies for using Letta

## Required Knowledge

### Essential Concepts
- **Python Programming**: Intermediate level (classes, async/await, decorators)
- **HTTP APIs**: Understanding REST endpoints and JSON responses
- **LLM Basics**: Familiarity with prompts, tokens, and model capabilities
- **Database Concepts**: Basic SQL knowledge helpful for debugging

### Recommended Background
- **Agent Architecture**: Understanding of autonomous systems and state management
- **Memory Systems**: Concepts of short-term vs long-term memory in AI
- **Tool Integration**: How AI systems interact with external APIs/tools
- **Containerization**: Docker basics for deployment

## Technical Requirements

### Hardware Specifications

#### Minimum Requirements
- **CPU**: 2 cores, 2.0 GHz
- **RAM**: 4 GB (8 GB recommended)
- **Storage**: 10 GB free space
- **Network**: Stable internet for LLM API calls

#### Recommended for Production
- **CPU**: 4+ cores, 3.0 GHz
- **RAM**: 16 GB+ 
- **Storage**: 50 GB+ SSD
- **Database**: Dedicated PostgreSQL instance

### Operating System Support
- **Linux**: Ubuntu 20.04+, CentOS 8+, Debian 11+
- **macOS**: 11.0+ (Big Sur)
- **Windows**: 10/11 with WSL2 recommended

## Software Dependencies

### Core Dependencies (Auto-installed)
```
Python 3.11-3.13
FastAPI 0.115.6+
SQLAlchemy 2.0.41+
Pydantic 2.10.6+
httpx 0.28.0+
```

### LLM Provider Dependencies
```bash
# OpenAI (most common)
pip install openai>=1.60.0

# Anthropic 
pip install anthropic>=0.49.0

# Local models
pip install ollama  # or specific provider packages
```

### Database Dependencies
```bash
# PostgreSQL (recommended)
pip install psycopg2-binary>=2.9.10
pip install pgvector>=0.2.3

# SQLite (development only)
# Included with Python standard library
```

## Environment Setup

### Development Environment
```bash
# 1. Python environment
python -m venv letta-env
source letta-env/bin/activate  # Linux/Mac
# or letta-env\Scripts\activate  # Windows

# 2. Install Letta
pip install -U letta

# 3. Set environment variables
export OPENAI_API_KEY="sk-..."
export LETTA_PG_URI="postgresql://user:pass@localhost:5432/letta"  # optional

# 4. Initialize database
letta server --init-db
```

### Production Environment
```bash
# 1. Docker setup (recommended)
docker run --name letta-postgres \
  -e POSTGRES_DB=letta \
  -e POSTGRES_USER=letta \
  -e POSTGRES_PASSWORD=secure_password \
  -p 5432:5432 \
  -d postgres:15

# 2. Install pgvector extension
docker exec letta-postgres psql -U letta -d letta \
  -c "CREATE EXTENSION IF NOT EXISTS vector;"

# 3. Run Letta server
docker run \
  -v ~/.letta/.persist/pgdata:/var/lib/postgresql/data \
  -p 8283:8283 \
  --env-file .env \
  letta/letta:latest
```

## LLM Provider Setup

### OpenAI Configuration
```bash
# Get API key from https://platform.openai.com/api-keys
export OPENAI_API_KEY="sk-proj-..."

# Optional: Set organization (for team accounts)
export OPENAI_ORG_ID="org-..."
```

### Anthropic Configuration
```bash
# Get API key from https://console.anthropic.com/
export ANTHROPIC_API_KEY="sk-ant-..."
```

### Local Model Setup (Ollama)
```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Download models
ollama pull llama3:8b
ollama pull mxbai-embed-large

# Configure Letta
export OLLAMA_BASE_URL="http://localhost:11434"
```

### Azure OpenAI Setup
```bash
export AZURE_OPENAI_API_KEY="your-key"
export AZURE_OPENAI_ENDPOINT="https://your-resource.openai.azure.com/"
export AZURE_OPENAI_VERSION="2024-02-15-preview"
```

## Database Setup

### PostgreSQL Setup (Recommended)
```bash
# Install PostgreSQL
sudo apt-get install postgresql postgresql-contrib  # Ubuntu
brew install postgresql  # macOS

# Create database and user
sudo -u postgres createuser letta
sudo -u postgres createdb letta --owner=letta
sudo -u postgres psql -c "ALTER USER letta WITH PASSWORD 'password';"

# Install pgvector extension
sudo -u postgres psql -d letta -c "CREATE EXTENSION vector;"

# Set connection string
export LETTA_PG_URI="postgresql://letta:password@localhost:5432/letta"
```

### SQLite Setup (Development Only)
```bash
# No setup required - SQLite is default for pip installations
# Database files stored in ~/.letta/
ls ~/.letta/
```

## Network Configuration

### Firewall Rules
```bash
# Allow Letta server port
sudo ufw allow 8283/tcp

# Allow PostgreSQL (if remote)
sudo ufw allow 5432/tcp
```

### Proxy Configuration
```bash
# If behind corporate proxy
export HTTP_PROXY="http://proxy.company.com:8080"
export HTTPS_PROXY="http://proxy.company.com:8080"
export NO_PROXY="localhost,127.0.0.1"
```

## IDE and Development Tools

### Recommended IDEs
- **VS Code**: With Python extension and Letta snippets
- **PyCharm**: Professional or Community edition
- **Vim/Neovim**: With LSP support for Python

### Useful Extensions/Packages
```bash
# Code formatting and linting
pip install black isort autoflake pyright

# Testing framework
pip install pytest pytest-asyncio

# Debugging tools
pip install ipdb rich
```

## Validation Checklist

Before starting development, verify:

### Environment Check
```bash
# Python version
python --version  # Should be 3.11-3.13

# Letta installation
python -c "import letta; print(letta.__version__)"

# Server connectivity
curl http://localhost:8283/health
```

### Database Check
```bash
# Test database connection
python -c "
from letta.server.db import db_registry
print('✅ Database connection successful')
"
```

### LLM Provider Check
```bash
# Test OpenAI connection
python -c "
import openai
client = openai.OpenAI()
print('✅ OpenAI connection successful')
"
```

## Common Setup Issues

### Python Version Conflicts
```bash
# Use pyenv for version management
pyenv install 3.11.8
pyenv local 3.11.8
```

### Database Connection Issues
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Test connection manually
psql -h localhost -U letta -d letta -c "SELECT version();"
```

### LLM API Issues
```bash
# Verify API key format
echo $OPENAI_API_KEY | grep -E "^sk-[a-zA-Z0-9]{48,}$"

# Test with minimal request
curl -H "Authorization: Bearer $OPENAI_API_KEY" \
  https://api.openai.com/v1/models
```

## Next Steps

- [Minimal Example](minimal-example.md) - Run your first agent
- [Installation](installation.md) - Detailed installation options
- [Architecture Overview](../01-core-concepts/architecture-overview.md) - Understanding how Letta works

## See Also

- [Deployment Guide](../04-integration/deployment.md)
- [Troubleshooting](../05-gotchas/troubleshooting.md)
- [🔗 Official Installation Guide](https://docs.letta.com/install)