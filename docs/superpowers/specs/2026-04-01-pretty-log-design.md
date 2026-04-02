# pretty-log 设计文档

**日期**：2026-04-01
**技术栈**：Rust
**定位**：流式日志格式美化 CLI 工具

---

## 一、概述

`pretty` 是一个支持流式输入的日志格式美化工具，通过 Unix 管道使用：

```bash
tail -f app.log | pretty
tail -f app.log | pretty -s -e
cat app.log | pretty --config ./myconfig.yaml
```

**目标**：
- 本地开发调试和生产环境排查均可使用
- 单一静态二进制，无运行时依赖
- 默认零配置，开箱即用

---

## 二、数据流架构

```
stdin
  │
  ▼
LineReader          ← 处理多行日志粘合（堆栈跟踪等）
  │  逻辑行（可能跨多个原始行）
  ▼
Parser              ← 尝试 JSON 解析；失败则 passthrough
  │  ParsedLine { fields: Map<String, Value>, raw: String }
  ▼
FieldClassifier     ← 根据 key 识别语义类型
  │  ClassifiedLine { level, timestamp, message, extras, continuation_lines }
  ▼
Renderer            ← 应用 ANSI 颜色，格式化输出
  │
  ▼
stdout
```

**TTY 检测**：非 TTY 输出时（如 `pretty | grep`）自动关闭 ANSI 颜色，也可通过 `--no-color` 强制关闭。

---

## 三、字段语义识别

### 内置别名映射

| 语义角色 | 默认 key 别名 |
|---------|-------------|
| `level` | level, lvl, severity, log_level |
| `timestamp` | time, timestamp, ts, @timestamp |
| `message` | msg, message, body |
| `trace_id` | trace_id, traceId, traceid, request_id, x-trace-id |
| `caller` | caller, file, source |

### 颜色方案

| 字段 | 颜色规则 |
|------|---------|
| `level` | ERROR=红, WARN=黄, INFO=绿, DEBUG=蓝灰 |
| `timestamp` | 暗青色 |
| `message` | 白色（高亮） |
| `trace_id` | 紫色 |
| `caller` | 暗灰 |
| 其他 key | 暗黄 |
| 其他 value | 白色 |
| 续行（堆栈等） | 暗灰，2 空格缩进 |

### 错误关键词高亮（`-e`）

当 `message` 字段内容包含以下词时，整行背景或关键词加红色高亮：
`error`, `err`, `Err`, `Error`, `ERROR`

---

## 四、多行日志处理

### 场景

日志系统输出堆栈跟踪等多行内容时：

```
{"level":"error","msg":"panic recovered","time":"2024-01-01T10:00:00Z"}
goroutine 1 [running]:
main.handler(...)
    /app/main.go:42
```

### 策略

`LineReader` 维护当前逻辑行缓冲区：
- 新行以 `{` 开头（或匹配 `continuation_pattern` 取反）→ flush 上一条，开始新逻辑行
- 新行不以 `{` 开头 → 追加为当前逻辑行的续行

### 输出效果

```
10:00:00 ERROR panic recovered
  goroutine 1 [running]:
  main.handler(...)
      /app/main.go:42
```

---

## 五、CLI 参数

| 参数 | 说明 |
|------|------|
| `-s`, `--expand` | 展开嵌套 JSON 字段值（多行显示） |
| `-e`, `--highlight-errors` | 高亮 message 中的错误关键词 |
| `--config <path>` | 指定配置文件路径 |
| `--no-color` | 强制关闭 ANSI 颜色输出 |

---

## 六、配置文件

### 查找顺序

1. `--config <path>` 命令行指定
2. `.pretty.yaml`（当前目录）
3. `~/.config/pretty/config.yaml`
4. 无配置 → 内置默认值

### 配置结构

```yaml
# 字段别名（追加到内置别名，不覆盖）
fields:
  level:     [level, lvl, severity, log_level]
  timestamp: [time, timestamp, ts]
  message:   [msg, message, body]
  trace_id:  [trace_id, traceId, request_id]

# 嵌套 JSON 是否默认展开（等同于 -s）
expand_nested: false

# 错误关键词高亮（等同于 -e）
highlight_errors: false

# 多行日志处理
multiline:
  enabled: true
  # 判断为"续行"的正则：不以 { 开头
  continuation_pattern: "^[^{]"
```

---

## 七、项目结构

```
pretty-log/
├── Cargo.toml
├── src/
│   ├── main.rs          ← CLI 入口，参数解析（clap）
│   ├── config.rs        ← 配置加载与合并（YAML + CLI 覆盖）
│   ├── reader.rs        ← 流式行读取 + 多行粘合
│   ├── parser.rs        ← JSON 解析，字段提取
│   ├── classifier.rs    ← 字段语义识别
│   └── renderer.rs      ← ANSI 颜色渲染，单行输出格式化
└── tests/
    └── integration.rs   ← 端到端测试（各种输入场景）
```

### 主要依赖

| crate | 用途 |
|-------|------|
| `clap` | CLI 参数解析 |
| `serde` + `serde_json` | JSON 解析 |
| `serde_yaml` | YAML 配置文件解析 |
| `owo-colors` | ANSI 颜色输出，支持 TTY 检测 |
| `dirs` | 获取 `~/.config` 等跨平台路径 |

---

## 八、错误处理

- 无法解析为 JSON 的行 → 原样输出（灰色提示为 raw 行）
- 配置文件解析失败 → 打印警告，使用默认配置继续运行
- stdin 关闭（pipe 结束）→ flush 最后一条逻辑行，正常退出

---

## 九、测试策略

- 单元测试：`parser`、`classifier`、`renderer` 各自独立测试
- 集成测试：构造 JSON 行、多行堆栈、非 JSON 行等场景，验证输出字符串
- 手动冒烟测试：`echo '{"level":"info","msg":"hello"}' | pretty`
