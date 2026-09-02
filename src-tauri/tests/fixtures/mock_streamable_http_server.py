#!/usr/bin/env python3
"""最小 MCP Streamable HTTP server（协议 2025-03-26），用于集成测试。

单端点 POST /mcp：
  - initialize  → application/json，并在响应头带 Mcp-Session-Id
  - tools/list  → application/json
  - tools/call  → text/event-stream（走 SSE 分支，覆盖另一条代码路径）
  - 通知（无 id）→ 202，无响应体

字段名用 snake_case，与 Jedi 客户端反序列化一致。绑定 0 端口并打印实际端口。
"""
import sys
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

SESSION = "test-session-1"

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


def build_result(msg):
    mid = msg.get("id")
    method = msg.get("method", "")
    if method == "initialize":
        return {"jsonrpc": "2.0", "id": mid, "result": {
            "protocol_version": "2025-03-26",
            "capabilities": {"tools": {}},
            "server_info": {"name": "mock-http", "version": "1.0"},
        }}
    if method == "tools/list":
        return {"jsonrpc": "2.0", "id": mid, "result": {"tools": TOOLS}}
    if method == "tools/call":
        args = (msg.get("params") or {}).get("arguments") or {}
        return {"jsonrpc": "2.0", "id": mid, "result": {
            "content": [{"type": "text", "text": "echo: " + str(args.get("text", ""))}],
            "is_error": False,
        }}
    return {"jsonrpc": "2.0", "id": mid, "error": {"code": -32601, "message": "method not found"}}


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.0"

    def log_message(self, *a):
        pass

    def do_DELETE(self):
        # 会话结束
        self.send_response(204)
        self.end_headers()

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        raw = self.rfile.read(length)
        try:
            msg = json.loads(raw)
        except Exception:
            self.send_response(400)
            self.end_headers()
            return

        # 通知（无 id）→ 202 无体
        if "id" not in msg:
            self.send_response(202)
            self.end_headers()
            return

        payload = json.dumps(build_result(msg))
        method = msg.get("method", "")

        if method == "tools/call":
            # SSE 分支：在事件流里回推响应
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Cache-Control", "no-cache")
            self.end_headers()
            try:
                self.wfile.write(("event: message\ndata: " + payload + "\n\n").encode())
                self.wfile.flush()
            except (BrokenPipeError, ConnectionResetError):
                pass
            return

        # JSON 分支
        body = payload.encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        if method == "initialize":
            self.send_header("Mcp-Session-Id", SESSION)
        self.end_headers()
        self.wfile.write(body)


def main():
    srv = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    sys.stdout.write("PORT %d\n" % srv.server_address[1])
    sys.stdout.flush()
    srv.serve_forever()


if __name__ == "__main__":
    main()
