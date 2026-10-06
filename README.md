# Perplexity MCP Server

A lightweight local MCP server for asking Perplexity quick questions. Written in Rust: a ~3MB binary using ~10MB of memory, so running several copies at once is cheap.

It is deliberately cheap to use. Every request uses the `sonar` model with `search_context_size: low` and no research mode. `sonar-pro` is available when asked for explicitly.

## Disclaimer

This repository was generated with Claude Sonnet 4.5 and rewritten in Rust with Claude Opus 5.5.

## Setup

### Option 1: Local binary

```bash
cargo install --path .
export PERPLEXITY_API_KEY="your-api-key-here"
```

This installs `perplexity-mcp` to `~/.cargo/bin`.

### Option 2: Docker

```bash
docker build -t perplexity-mcp .
```

## Usage

### Transports

**stdio (default):**

```bash
perplexity-mcp
```

**Streamable HTTP:**

```bash
perplexity-mcp --http
```

Serves on `http://127.0.0.1:8000/mcp`. Override with `HOST` and `PORT`:

```bash
HOST=127.0.0.1 PORT=9000 perplexity-mcp --http
```

With Docker:

```bash
docker run -p 8000:8000 -e PERPLEXITY_API_KEY perplexity-mcp --http
```

### Tools

#### `ask_perplexity`

- `question` (string, required): the question to ask, up to 5000 characters.
- `model` (string, optional): `sonar` (default, fast and cheap) or `sonar-pro` (more detailed, several times the cost).

```json
{ "question": "What is quantum entanglement?" }
```

```json
{ "question": "Explain the history of quantum computing", "model": "sonar-pro" }
```

### Claude Code

Local binary:

```bash
claude mcp add perplexity --scope user -e PERPLEXITY_API_KEY="$PERPLEXITY_API_KEY" -- perplexity-mcp
```

Or in JSON config:

```json
{
  "mcpServers": {
    "perplexity": {
      "type": "stdio",
      "command": "perplexity-mcp",
      "env": { "PERPLEXITY_API_KEY": "${PERPLEXITY_API_KEY}" }
    }
  }
}
```

Docker:

```json
{
  "mcpServers": {
    "perplexity": {
      "type": "stdio",
      "command": "docker",
      "args": ["run", "--rm", "-i", "-e", "PERPLEXITY_API_KEY", "perplexity-mcp"]
    }
  }
}
```

Other MCP clients are configured the same way.

## Troubleshooting

- **`PERPLEXITY_API_KEY environment variable is not set`**: the server exits on startup without a key. Export it or pass it in your MCP config.
- **Authentication failed**: check the API key is correct and has not expired.
- **Rate limit exceeded**: wait a moment before trying again.
- **Request timed out**: requests time out after 30 seconds. Try again, or use `sonar` instead of `sonar-pro`.
