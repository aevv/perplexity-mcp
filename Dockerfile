FROM python:3.11-slim

WORKDIR /app

# Copy requirements and install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy the MCP server
COPY perplexity_mcp.py .

# Expose port for HTTP transport (default: 8000)
EXPOSE 8000

# Run the server (use CMD to allow easy override)
# Default: stdio transport
# For HTTP: docker run -p 8000:8000 perplexity-mcp python perplexity_mcp.py --http
CMD ["python", "perplexity_mcp.py"]
