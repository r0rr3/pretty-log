# pretty-log

把 JSON 日志管道进来，得到带颜色、可读的输出。接 `tail -f` 看实时流，加 `-t` 切换成全终端表格视图。

**[中文版本](README.zh-CN.md)** | **[English](README.md)**

[![Rust](https://img.shields.io/badge/language-Rust-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)

## 特性

- 流式解析，逐行读取，无缓冲延迟，`tail -f` 实时生效
- 彩色输出，自动检测终端，按日志级别着色
- 多行分组，堆栈跟踪自动归到上一条日志并缩进显示
- 开箱即用，内置常见字段别名，不用写配置
- 交互式表格，全终端 TUI，搜索、滚动、详情面板都有
- YAML 配置，字段名不一样时可以自定义映射
- 单一静态二进制，约 5 MB，无运行时依赖

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
cat app.log | pretty --no-color | grep ERROR  # 管道友好输出
```

## 效果展示

输入：

```json
{"level":"info","msg":"server started","port":8080,"time":"2024-06-15T14:30:00Z"}
{"level":"error","msg":"crash","trace_id":"abc-123","time":"2024-06-15T14:30:01Z"}
goroutine 1 [running]:
main.handler(...)
```

输出：

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
| `--config <path>` | | 指定配置文件路径 |
| `--no-color` | | 禁用 ANSI 彩色输出 |

## 表格模式

加 `-t` 启动。日志以可滚动的表格铺满终端，长消息自动换行，详情面板显示完整条目，搜索覆盖所有字段。

快捷键：

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

搜索采用 KMP 算法，大小写不敏感，在消息和所有字段中匹配，结果高亮显示。

实时接收时向上翻页，新日志继续在后台缓冲。状态栏会显示 `↓ N new`，按 `G` 或 `End` 跳回最新一条。

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

50ms 超时确保每次突发输出的最后一行都能及时刷出，`tail -f` 不会卡在最后一行。

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

- 仅支持 JSON 对象，顶层数组会原样透传
- 没有内置过滤，用 `grep` 或 `jq` 接管
- 配置里写错的正则会回退到默认 pattern

## 许可证

GPL-3.0 — 详见 [LICENSE](LICENSE)

---

用 Rust 写的 ❤️
