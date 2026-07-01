#!/usr/bin/env python3
"""最小 MCP server（stdio / 行分隔 JSON-RPC），用于集成测试 Jedi 的 MCP 客户端。

注意：字段名采用 Jedi 客户端反序列化所用的 snake_case（protocol_version /
server_info / input_schema），与 Jedi 的 types.rs 保持一致。
"""
import sys
import json


def respond(obj):
    sys.stdout.write(json.dumps(obj) + "\n")
    sys.stdout.flush()


TOOLS = [
    {
        "name": "echo",
        "description": "echo back the text",
        "input_schema": {
            "type": "object",
            "properties": {"text": {"type": "string"}},
            "required": ["text"],
        },
    }
]


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue

        # 通知（无 id）不回复
        if "id" not in msg:
            continue

        mid = msg["id"]
        method = msg.get("method", "")

        if method == "initialize":
            respond({
                "jsonrpc": "2.0",
                "id": mid,
                "result": {
                    "protocol_version": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "server_info": {"name": "mock", "version": "1.0"},
                },
            })
        elif method == "tools/list":
            respond({"jsonrpc": "2.0", "id": mid, "result": {"tools": TOOLS}})
        elif method == "tools/call":
            params = msg.get("params", {}) or {}
            args = params.get("arguments", {}) or {}
            text = args.get("text", "")
            respond({
                "jsonrpc": "2.0",
                "id": mid,
                "result": {
                    "content": [{"type": "text", "text": "echo: " + str(text)}],
                    "is_error": False,
                },
            })
        else:
            respond({
                "jsonrpc": "2.0",
                "id": mid,
                "error": {"code": -32601, "message": "method not found: " + method},
            })


if __name__ == "__main__":
    main()
