# pretty-log

一个高性能的流式日志美化工具。将原始 JSON 日志转换为易于阅读的彩色输出，支持多行堆栈跟踪和可自定义字段映射。

[![Rust](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## 特性

✨ **流式 JSON 解析** — 逐行处理日志，无缓冲  
🎨 **ANSI 彩色输出** — 自动检测终端，智能着色  
📋 **多行分组** — 保留堆栈跟踪，自动缩进  
⚙️ **开箱即用** — 内置合理的字段别名，无需配置  
🔧 **灵活配置** — YAML 配置文件，自定义字段映射  
⚡ **单一静态二进制** — 无运行时依赖，约 5MB  

## 安装

### Homebrew（macOS 和 Linux）

```bash
brew install jsooo/tap/pretty-log
```

详见 [HOMEBREW.md](HOMEBREW.md)。

### 从源码编译

```bash
git clone https://github.com/jsooo/pretty-log.git
cd pretty-log
cargo build --release
./target/release/pretty --help
```

## 快速开始

```bash
# 基本用法
tail -f app.log | pretty

# 高亮错误关键词
tail -f app.log | pretty -e

# 展开嵌套 JSON
cat app.log | pretty -s

# 禁用颜色（用于 grep）
cat app.log | pretty --no-color | grep "ERROR"
```

## 例子

**输入：**
```json
{"level":"info","msg":"server started","port":8080,"time":"2024-06-15T14:30:00Z"}
{"level":"error","msg":"crash","trace_id":"abc-123","time":"2024-06-15T14:30:01Z"}
goroutine 1 [running]:
main.handler(...)
```

**输出：**
```
14:30:00  INFO   server started  port=8080
14:30:01  ERROR  crash  trace=abc-123
  goroutine 1 [running]:
  main.handler(...)
```

## CLI 参数

| 参数 | 短形式 | 说明 |
|------|--------|------|
| `--expand` | `-s` | 展开嵌套 JSON 字段值 |
| `--highlight-errors` | `-e` | 高亮错误关键词（红色） |
| `--config PATH` | | 加载 YAML 配置文件 |
| `--no-color` | | 禁用 ANSI 彩色输出 |

## 配置

### 配置文件位置（按优先级）

1. `--config <path>` (命令行指定)
2. `.pretty.yaml` (当前目录)
3. `~/.config/pretty/config.yaml` (用户主目录)
4. 内置默认值

### 配置示例

创建 `~/.config/pretty/config.yaml`：

```yaml
# 字段别名映射（追加到内置别名）
fields:
  level:     [level, lvl, severity, log_level]
  timestamp: [time, timestamp, ts, "@timestamp"]
  message:   [msg, message, body]
  trace_id:  [trace_id, traceId, request_id]
  caller:    [caller, file, source]

# 是否默认展开嵌套 JSON（等同于 -s）
expand_nested: false

# 是否高亮错误关键词（等同于 -e）
highlight_errors: false

# 多行日志处理
multiline:
  enabled: true
  # 判断续行的正则表达式
  continuation_pattern: "^[^\{]"
```

## 内置字段别名

- **level：** `level`, `lvl`, `severity`, `log_level`
- **timestamp：** `time`, `timestamp`, `ts`, `@timestamp`
- **message：** `msg`, `message`, `body`
- **trace_id：** `trace_id`, `traceId`, `traceid`, `request_id`, `x-trace-id`
- **caller：** `caller`, `file`, `source`

## 颜色方案

| 元素 | 颜色 |
|------|------|
| ERROR | 红色 + 加粗 |
| WARN | 黄色 + 加粗 |
| INFO | 绿色 |
| DEBUG | 蓝色 |
| TRACE | 深灰色 |
| 时间戳 | 青色 |
| 消息 | 白色 + 加粗 |
| Trace ID | 品红色 |
| 续行 | 深灰色（缩进） |

## 使用场景

### 本地开发

使用自动着色监控应用日志：

```bash
cargo run -- app.log | pretty -e
```

### 生产环境

SSH 连接到服务器并查看日志：

```bash
ssh user@server "tail -f /var/log/app.log" | pretty
```

### 日志处理管道

与标准 Unix 工具结合使用：

```bash
# 只显示 ERROR 级别
cat app.log | pretty | grep "ERROR"

# 保存格式化的日志
tail -f app.log | pretty > formatted.log
```

## 编译

```bash
# 调试版本
cargo build

# 发布版本（优化）
cargo build --release

# 运行测试
cargo test
```

## 项目结构

```
pretty-log/
├── src/
│   ├── main.rs          # CLI 入口
│   ├── config.rs        # YAML 配置加载
│   ├── reader.rs        # 流式读取器（多行支持）
│   ├── parser.rs        # JSON 解析
│   ├── classifier.rs    # 字段语义识别
│   └── renderer.rs      # ANSI 颜色渲染
├── tests/
│   └── integration.rs   # 集成测试
├── Cargo.toml
└── README.md
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test basic_json_line_output

# 显示输出
cargo test -- --nocapture
```

## 已知限制

- 仅支持 JSON 对象（不支持数组作为顶级行）
- 单行输出格式
- 无内置过滤（使用 shell pipe）
- 配置中的正则表达式无效时使用默认值

## 许可证

MIT 许可证 — 详见 [LICENSE](LICENSE)

---

用 ❤️ 用 Rust 制作

**[English Version](README.md)**
