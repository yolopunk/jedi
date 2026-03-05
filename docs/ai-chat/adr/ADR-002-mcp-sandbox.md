# ADR-002: MCP 沙箱方案

**日期**：2026-03-05  
**状态**：已批准  
**架构师**：架构师智能体

## 上下文

MCP (Model Context Protocol) 服务器可以执行敏感操作：
- 访问文件系统
- 执行进程
- 调用系统 API
- 网络访问

如果没有适当的隔离，恶意或有缺陷的 MCP 服务器可能导致：
- 数据泄露
- 系统破坏
- 恶意代码执行

## 决策

使用 Docker 容器作为 MCP 服务器的沙箱环境。

## 备选方案

### 方案 1：不使用沙箱（否决）
- **优点**：实现简单，性能好
- **缺点**：安全风险极高，不可接受

### 方案 2：虚拟机（否决）
- **优点**：隔离性最强
- **缺点**：资源消耗大，启动慢，用户体验差

### 方案 3：Docker 容器（选中）
- **优点**：成熟的隔离技术，资源占用适中，支持资源限制
- **缺点**：需要用户安装 Docker，有一定的资源消耗

### 方案 4：Wasm 运行时（备选）
- **优点**：轻量，安全
- **缺点**：生态不成熟，MCP 服务器支持有限

## 理由

1. **隔离性**：Docker 提供进程级隔离，支持 cgroups 资源限制
2. **成熟度**：Docker 是成熟的容器技术，广泛使用
3. **灵活性**：支持只读文件系统、网络策略、用户权限控制
4. **可观测性**：有丰富的工具监控和管理容器

## 后果

### 正面
- 良好的安全隔离
- 支持细粒度权限控制
- 资源限制可控

### 负面
- 用户需要安装 Docker Desktop（Windows/macOS）或 Docker Engine（Linux）
- 增加系统资源消耗
- 容器管理增加复杂度
- 启动延迟增加

## 实现细节

### 沙箱架构
```
Jedi App → MCP Security Layer → Docker Container → MCP Server
```

### 安全控制
- 只读文件系统（除指定目录）
- 非特权用户运行
- CPU/内存限制
- 网络策略控制
- 操作审计日志

### 依赖
- `bollard` crate：Docker API 客户端
- `docker`（用户安装）

### 代码结构
```
src-tauri/src/mcp/
├── sandbox.rs      # 沙箱管理
├── docker.rs       # Docker 集成
├── permissions.rs  # 权限模型
└── mod.rs
```

## 相关决策

- ADR-005: 错误处理方案
- ADR-001: API Key 存储方案

## 参考资料

- Docker Security Guide
- MCP Specification
- OWASP Container Security
