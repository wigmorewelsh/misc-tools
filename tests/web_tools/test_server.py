import pytest
from fastmcp.client import Client
from fastmcp.client.transports import FastMCPTransport
from unittest.mock import patch, MagicMock
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src" / "web-tools"))

from server import mcp


@pytest.fixture
async def web_tools_client():
    async with Client(transport=mcp) as client:
        yield client


async def test_list_tools(web_tools_client: Client[FastMCPTransport]):
    tools = await web_tools_client.list_tools()

    assert len(tools) == 2
    tool_names = [tool.name for tool in tools]
    assert "websearch" in tool_names
    assert "fetch" in tool_names


async def test_websearch_success(web_tools_client: Client[FastMCPTransport]):
    mock_result = MagicMock()
    mock_result.stdout = b"Search result 1\nSearch result 2\nSearch result 3"

    with patch("server.subprocess.run", return_value=mock_result):
        result = await web_tools_client.call_tool(
            name="websearch",
            arguments={"query": "test query"}
        )

        assert result.data is not None
        assert isinstance(result.data, str)
        assert "Search result 1" in result.data


async def test_websearch_timeout(web_tools_client: Client[FastMCPTransport]):
    with patch("server.subprocess.run", side_effect=subprocess.TimeoutExpired("lynx", 30)):
        result = await web_tools_client.call_tool(
            name="websearch",
            arguments={"query": "test query"}
        )

        assert result.data is not None
        assert "timed out" in result.data


async def test_websearch_lynx_not_installed(web_tools_client: Client[FastMCPTransport]):
    with patch("server.subprocess.run", side_effect=FileNotFoundError()):
        result = await web_tools_client.call_tool(
            name="websearch",
            arguments={"query": "test query"}
        )

        assert result.data is not None
        assert "lynx is not installed" in result.data


async def test_fetch_success(web_tools_client: Client[FastMCPTransport]):
    mock_result = MagicMock()
    mock_result.stdout = b"Page content\nMore content"
    mock_result.returncode = 0

    with patch("server.subprocess.run", return_value=mock_result):
        result = await web_tools_client.call_tool(
            name="fetch",
            arguments={"url": "https://example.com"}
        )

        assert result.data is not None
        assert isinstance(result.data, str)
        assert "Page content" in result.data


async def test_fetch_timeout(web_tools_client: Client[FastMCPTransport]):
    with patch("server.subprocess.run", side_effect=subprocess.TimeoutExpired("lynx", 30)):
        result = await web_tools_client.call_tool(
            name="fetch",
            arguments={"url": "https://example.com"}
        )

        assert result.data is not None
        assert "timed out" in result.data


async def test_fetch_lynx_not_installed(web_tools_client: Client[FastMCPTransport]):
    with patch("server.subprocess.run", side_effect=FileNotFoundError()):
        result = await web_tools_client.call_tool(
            name="fetch",
            arguments={"url": "https://example.com"}
        )

        assert result.data is not None
        assert "lynx is not installed" in result.data
