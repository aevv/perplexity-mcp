# Perplexity MCP Server

A lightweight local MCP server for querying Perplexity AI. This server provides a simple interface to ask questions and get answers from Perplexity's API.

## Disclaimer

This repository was generated with Claude Sonnet 4.5

## Setup

### Option 1: Local Installation

1. **Install dependencies:**
   ```bash
   pip install -r requirements.txt
   ```

2. **Set your API key:**
   ```bash
   export PERPLEXITY_API_KEY="your-api-key-here"
   ```

### Option 2: Docker Container

1. **Build the image:**
   ```bash
   docker build -t perplexity-mcp .
   ```

2. **Set your API key:**
   ```bash
   export PERPLEXITY_API_KEY="your-api-key-here"
   ```

3. **Run with docker (recommended):**
   ```bash
   docker run -it \
     -e PERPLEXITY_API_KEY="${PERPLEXITY_API_KEY} \
     perplexity-mcp
   ```

## Usage

### Running the Server

Start the MCP server with stdio transport:

```bash
python perplexity_mcp.py
```

The server will start and wait for requests over stdin/stdout.

### Tools Available

#### `ask_perplexity`

Ask a question to Perplexity AI and get an answer.

**Parameters:**
- `question` (string, required): The question to ask Perplexity AI
- `model` (string, optional): Which model to use
  - `"sonar"` (default): Faster responses
  - `"sonar-pro"`: More detailed/comprehensive responses

**Example requests:**

Quick question with default model:
```json
{
  "question": "What is quantum entanglement?"
}
```

Detailed question with pro model:
```json
{
  "question": "Explain the history and current state of quantum computing",
  "model": "sonar-pro"
}
```

### Integrating with Claude

#### Local Installation

Add it to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "perplexity": {
      "command": "python",
      "args": ["/path/to/perplexity_mcp.py"],
      "env": {
        "PERPLEXITY_API_KEY": "${PERPLEXITY_API_KEY}"
      }
    }
  }
}
```

#### Docker Container

```json
{
  "mcpServers": {
    "perplexity": {
      "type": "stdio",
      "command": "docker",
      "args": [
        "run",
        "--rm",
        "-i",
        "-e",
        "PERPLEXITY_API_KEY=${PERPLEXITY_API_KEY}",
        "perplexity-mcp"
      ]
    }
  }
}
```

Make sure to set `PERPLEXITY_API_KEY` in your shell environment.

## Integrating with other tools

Check the docs for that specific tool, the MCP setup will be similar to Claude Code.

## Troubleshooting

**Error: PERPLEXITY_API_KEY environment variable is not set**
- Make sure you've exported the environment variable: `export PERPLEXITY_API_KEY="your-key"` or use a `.env` file for environment config.

**Error: Authentication failed**
- Check that your API key is correct and has not expired

**Error: Rate limit exceeded**
- Wait a moment before making another request. Perplexity has rate limits on API usage.

**Timeout errors**
- The Perplexity API took too long to respond. Try again or use the faster "sonar" model instead of "sonar-pro".

## Notes

- This is a simple, local-only server designed for personal use
- The server uses async/await for non-blocking I/O
- Responses are limited by the Perplexity API's capabilities and rate limits
- The server respects the MCP protocol's stdio transport
