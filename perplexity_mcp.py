"""
Perplexity MCP Server

A simple local MCP server that provides tools to ask questions to Perplexity AI
and get answers via the Perplexity API.
"""

import os
import sys
from mcp.server.fastmcp import FastMCP
from pydantic import BaseModel, Field
import httpx
import json

# Configuration
API_KEY = os.getenv("PERPLEXITY_API_KEY")
API_BASE_URL = "https://api.perplexity.ai"

# Check if HTTP transport is requested
if len(sys.argv) > 1 and sys.argv[1] == "--http":
    # HTTP transport mode - initialize with custom host/port
    host = os.getenv("HOST", "0.0.0.0")
    port = int(os.getenv("PORT", "8000"))
    mcp = FastMCP("perplexity_mcp", host=host, port=port)
else:
    # Default: stdio transport
    mcp = FastMCP("perplexity_mcp")


class AskInput(BaseModel):
    """Input model for asking a question to Perplexity."""
    
    question: str = Field(
        ..., 
        description="The question to ask Perplexity AI", 
        min_length=1,
        max_length=5000
    )
    model: str = Field(
        default="sonar",
        description="The model to use. Options: 'sonar' (fast), 'sonar-pro' (detailed). Default is 'sonar'."
    )


@mcp.tool(
    name="ask_perplexity",
    annotations={
        "title": "Ask Perplexity a Question",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": True
    }
)
async def ask_perplexity(params: AskInput) -> str:
    """
    Ask a question to Perplexity AI and get an answer.
    
    This tool sends your question to Perplexity's API and returns the response.
    You can choose between 'sonar' (faster) and 'sonar-pro' (more detailed) models.
    
    Args:
        params: An object containing:
            - question (str): The question to ask (required)
            - model (str): Which model to use - 'sonar' or 'sonar-pro' (default: 'sonar')
    
    Returns:
        str: The answer from Perplexity AI
    
    Raises:
        ValueError: If the API key is not set or if the API request fails
    
    Example:
        Ask a quick question:
        {
            "question": "What is the capital of France?",
            "model": "sonar"
        }
        
        Ask a more detailed question:
        {
            "question": "Explain quantum computing and its applications",
            "model": "sonar-pro"
        }
    """
    
    if not API_KEY:
        raise ValueError(
            "PERPLEXITY_API_KEY environment variable is not set. "
            "Please set it before running this server."
        )
    
    if params.model not in ["sonar", "sonar-pro"]:
        raise ValueError(
            f"Invalid model '{params.model}'. Must be 'sonar' or 'sonar-pro'."
        )
    
    payload = {
        "model": params.model,
        "messages": [
            {
                "role": "user",
                "content": params.question
            }
        ]
    }
    
    headers = {
        "Authorization": f"Bearer {API_KEY}",
        "Content-Type": "application/json"
    }
    
    async with httpx.AsyncClient() as client:
        try:
            response = await client.post(
                f"{API_BASE_URL}/chat/completions",
                json=payload,
                headers=headers,
                timeout=30.0
            )
            response.raise_for_status()
            
            data = response.json()
            
            # Extract the answer from the response
            if "choices" in data and len(data["choices"]) > 0:
                message = data["choices"][0].get("message", {})
                answer = message.get("content", "No response received")
                return answer
            else:
                raise ValueError("Unexpected API response format")
                
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 401:
                raise ValueError(
                    "Authentication failed. Check that your PERPLEXITY_API_KEY is correct."
                )
            elif e.response.status_code == 429:
                raise ValueError(
                    "Rate limit exceeded. Please wait before making another request."
                )
            else:
                error_details = e.response.text
                raise ValueError(
                    f"API request failed with status {e.response.status_code}: {error_details}"
                )
        except httpx.TimeoutException:
            raise ValueError(
                "Request timed out. Perplexity API took too long to respond."
            )
        except httpx.RequestError as e:
            raise ValueError(f"Request failed: {str(e)}")


if __name__ == "__main__":
    # Check if HTTP transport is requested
    if len(sys.argv) > 1 and sys.argv[1] == "--http":
        # HTTP transport mode
        print(f"Starting Perplexity MCP Server with HTTP transport on {mcp.settings.host}:{mcp.settings.port}")
        mcp.run(transport="streamable-http")
    else:
        # Default: stdio transport
        mcp.run()
