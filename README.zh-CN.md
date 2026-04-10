# pretty-log

高性能流式 JSON 日志美化工具。通过 `tail -f` 管道获得实时彩色输出，或使用 `-t` 进入全终端交互式表格视图。

**[中文版本](README.zh-CN.md)** | **[English](README.md)**

[![Rust](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)

## 特性

- **流式 JSON 解析** — 逐行读取，无缓冲延迟，`tail -f` 实时生效
- **ANSI 彩色输出** — 自动检测终端，按日志级别着色
- **多行分组** — 自动将堆栈跟踪归属到上一条 JSON 日志并缩进显示
- **开箱即用** — 内置常见字段别名，无需配置
- **交互式表格模式** — 全终端 TUI，支持搜索、滚动和详情面板
- **YAML 配置** — 可自定义字段映射和行为
- **单一静态二进制** — 约 5 MB，无运行时依赖

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
tail -f app.log | pretty              # 实时流式美化
cat app.log | pretty                  # 管道处理文件
tail -f app.log | pretty -t           # 交互式表格模式
tail -f app.log | pretty -t -x        # 表格模式 + 详情面板显示额外字段
cat app.log | pretty --no-color | grep ERROR  # 管道友好输出
```

## 效果展示

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

![普通模式](assets/common_mode.jpg)

## CLI 参数

| 参数 | 短形式 | 说明 |
|------|--------|------|
| `--expand` | `-s` | 展开嵌套 JSON 字段值 |
| `--highlight-errors` | `-e` | 高亮消息中的错误关键词 |
| `--table` | `-t` | 启用交互式表格视图 |
| `--extras` | `-x` | 在详情面板显示额外字段（仅表格模式） |
| `--config <path>` | | 指定配置文件路径 |
| `--no-color` | | 禁用 ANSI 彩色输出 |

## 表格模式

使用 `-t` 启动。以全终端交互式表格展示日志，消息超长自动换行，支持实时搜索和详情面板。

**快捷键：**

| 按键 | 操作 |
|------|------|
| `↑` / `↓` | 上下移动光标 |
| 鼠标滚轮 | 滚动 |
| `g` / `Home` | 跳转到第一条 |
| `G` / `End` | 跳转到最新一条 |
| `Space` | 暂停 / 恢复自动滚动 |
| `/` | 打开搜索 |
| `n` / `N` | 下一个 / 上一个搜索结果 |
| `Esc` | 清除搜索 |
| `q` | 退出 |

**搜索**采用 KMP 算法，大小写不敏感，在消息和所有额外字段中匹配，结果高亮显示。

实时滚动期间向上翻页后，新日志持续缓冲。状态栏显示 `↓ N new`，按 `G` 或 `End` 跳回最新。

![表格模式](assets/table_mode.jpg)

## 配置

配置文件加载优先级：

1. `--config <path>`（命令行指定）
2. `.pretty.yaml`（当前目录）
3. `~/.config/pretty/config.yaml`（用户目录）

```yaml
fields:
  level:     [level, lvl, severity, log_level]
  timestamp: [time, timestamp, ts, "@timestamp"]
  message:   [msg, message, body]
  trace_id:  [trace_id, traceId, request_id, x-trace-id]
  caller:    [caller, file, source]

expand_nested: false
highlight_errors: false

multiline:
  enabled: true
  continuation_pattern: "^[^{]"

table:
  columns: [time, level, message]
  show_extras_in_detail: false
```

## 内置字段别名

| 语义字段 | 识别的键名 |
|---------|-----------|
| level | `level`, `lvl`, `severity`, `log_level` |
| timestamp | `time`, `timestamp`, `ts`, `@timestamp` |
| message | `msg`, `message`, `body` |
| trace_id | `trace_id`, `traceId`, `traceid`, `request_id`, `x-trace-id` |
| caller | `caller`, `file`, `source` |

## 颜色方案

| 级别 | 颜色 |
|------|------|
| ERROR | 红色 |
| WARN | 黄色 |
| INFO | 绿色 |
| DEBUG | 蓝色 |
| TRACE | 深灰色 |

## 工作原理

```
stdin
  └─ 读取线程（阻塞式 read_line）
       └─ channel（50ms 超时 flush）
            └─ 多行组装器
                 └─ parser → classifier → renderer → stdout
```

50ms 超时机制确保每次日志突发输出的最后一行始终及时显示，这是 `tail -f` 能正常工作的关键。

## 项目结构

```
pretty-log/
├── src/
│   ├── main.rs          CLI 入口，流式处理循环
│   ├── config.rs        YAML 配置加载与合并
│   ├── reader.rs        多行分组，续行检测
│   ├── parser.rs        JSON 解析，字段提取
│   ├── classifier.rs    字段语义识别
│   ├── renderer.rs      格式化与着色输出
│   └── table.rs         交互式 TUI 表格模式（-t）
├── tests/
│   └── integration.rs   端到端流水线测试
├── Cargo.toml
└── README.md
```

## 编译与测试

```bash
cargo build
cargo build --release
cargo test
```

## 已知限制

- 仅支持 JSON 对象（数组顶层行将原样透传）
- 无内置过滤，请配合 `grep` 或 `jq` 使用
- 配置中无效的正则表达式将使用默认 pattern

## 许可证

GPL-3.0 — 详见 [LICENSE](LICENSE)

---

用 ❤️ 用 Rust 制作
