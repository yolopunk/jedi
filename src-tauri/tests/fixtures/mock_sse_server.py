#!/usr/bin/env python3
"""最小 MCP HTTP+SSE server（协议 2024-11-05），用于集成测试 Jedi 的 SSE 传输。

流程：
  GET  /sse       → text/event-stream；先发 `event: endpoint`（data=/messages），
                    然后把队列里的响应作为 `event: message` 推回
  POST /messages  → 读取 JSON-RPC 请求，构造响应放入队列，返回 202

字段名用 snake_case，与 Jedi 客户端反序列化一致。绑定 0 端口并把实际端口打印到 stdout。
"""
import sys
import json
import queue
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

OUT: "queue.Queue[str]" = queue.Queue()

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


def build_response(msg):
    mid = msg.get("id")
    method = msg.get("method", "")
    if method == "initialize":
        return {"jsonrpc": "2.0", "id": mid, "result": {
            "protocol_version": "2024-11-05",
            "capabilities": {"tools": {}},
            "server_info": {"name": "mock-sse", "version": "1.0"},
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

    def do_GET(self):
        if not self.path.startswith("/sse"):
            self.send_response(404)
            self.end_headers()
            return
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.end_headers()
        try:
            self.wfile.write(b"event: endpoint\ndata: /messages\n\n")
            self.wfile.flush()
            while True:
                try:
                    item = OUT.get(timeout=30)
                except queue.Empty:
                    break
                if item is None:
                    break
                self.wfile.write(("event: message\ndata: " + item + "\n\n").encode())
                self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            pass

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length)
        try:
            msg = json.loads(body)
        except Exception:
            self.send_response(400)
            self.end_headers()
            return
        # 通知（无 id）不回复
        if "id" in msg:
            OUT.put(json.dumps(build_response(msg)))
        self.send_response(202)
        self.end_headers()


def main():
    srv = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    port = srv.server_address[1]
    sys.stdout.write("PORT %d\n" % port)
    sys.stdout.flush()
    srv.serve_forever()


if __name__ == "__main__":
    main()
