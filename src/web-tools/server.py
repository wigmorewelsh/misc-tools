import argparse
import os
import subprocess

from fastmcp import FastMCP

mcp = FastMCP("Web Tools")

proxy_url = None


def get_lynx_env():
    """Get environment with SSL certificate configured for mitmproxy and proxy settings."""
    env = os.environ.copy()

    mitmproxy_cert = os.path.expanduser("~/.mitmproxy/mitmproxy-ca-cert.pem")
    system_cert = "/etc/ssl/cert.pem"
    combined_cert = "/tmp/mcp-combined-certs.pem"

    if os.path.exists(mitmproxy_cert) and os.path.exists(system_cert):
        with open(combined_cert, 'w') as combined:
            with open(system_cert, 'r') as system:
                combined.write(system.read())
            with open(mitmproxy_cert, 'r') as mitmproxy:
                combined.write(mitmproxy.read())
        env["SSL_CERT_FILE"] = combined_cert

    if proxy_url:
        env["http_proxy"] = proxy_url
        env["https_proxy"] = proxy_url

    return env


@mcp.tool
def websearch(query: str) -> str:
    """Search the web using DuckDuckGo via lynx and return results as text."""
    search_url = f"https://lite.duckduckgo.com/lite/?q={query.replace(' ', '+')}"

    try:
        result = subprocess.run(
            ["lynx", "-dump", "-nolist", "-width=200", search_url],
            capture_output=True,
            text=False,
            timeout=30,
            env=get_lynx_env(),
        )

        return result.stdout.decode('utf-8', errors='replace').strip()
    except subprocess.TimeoutExpired:
        return "Error: Search request timed out"
    except subprocess.CalledProcessError as e:
        return f"Error: lynx command failed with exit code {e.returncode}"
    except FileNotFoundError:
        return "Error: lynx is not installed. Install it with: brew install lynx"


@mcp.tool
def fetch(url: str) -> str:
    """Fetch and convert a webpage to plain text using lynx."""
    try:
        result = subprocess.run(
            ["lynx", "-dump", "-nolist", "-width=120", url],
            capture_output=True,
            text=False,
            timeout=30,
            env=get_lynx_env(),
        )

        output = result.stdout.decode('utf-8', errors='replace').strip()

        # If proxy failed or returned empty/error content, try without proxy
        if result.returncode != 0 or not output or "Enable JavaScript and cookies to continue" in output:
            if proxy_url:
                # Retry without proxy
                env_no_proxy = os.environ.copy()
                result = subprocess.run(
                    ["lynx", "-dump", "-nolist", "-width=120", url],
                    capture_output=True,
                    text=False,
                    timeout=30,
                    env=env_no_proxy,
                )
                output = result.stdout.decode('utf-8', errors='replace').strip()

        return output
    except subprocess.TimeoutExpired:
        return f"Error: Request to {url} timed out"
    except subprocess.CalledProcessError as e:
        return f"Error: lynx command failed with exit code {e.returncode}"
    except FileNotFoundError:
        return "Error: lynx is not installed. Install it with: brew install lynx"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Web Tools MCP Server")
    parser.add_argument(
        "--proxy",
        type=str,
        help="HTTP proxy URL (e.g., http://localhost:8080)",
    )
    args, unknown = parser.parse_known_args()

    if args.proxy:
        proxy_url = args.proxy

    mcp.run()
