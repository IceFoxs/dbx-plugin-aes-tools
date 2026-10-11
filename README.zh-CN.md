# aes-tools

基于 [DBX](https://dbxio.com) 平台的 AES-128-ECB 加解密插件，兼容 [Hutool](https://hutool.cn/) 默认 AES 配置。

后端采用 **Rust** 编写，前端工作台采用 shadcn/ui 风格设计。

![AES Tools 工作台界面](docs/screen.png)

## 功能特性

- **AES-128-ECB** 加密 / 解密，**PKCS7** 填充
- **Base64** 输出编码，兼容 Hutool 默认配置
- 支持多密钥标签管理
- 基于连接（Connection）的密钥隔离
- 暗色模式支持，遵循 shadcn/ui 设计系统
- 多语言界面：简体中文、繁体中文、英文、日文、韩文

## 环境要求

- [Node.js](https://nodejs.org/) 22+
- [Rust](https://www.rust-lang.org/) 工具链
- [DBX Plugin CLI](https://www.npmjs.com/package/@dbx-app/plugin-cli)

## 快速开始

```bash
git clone <repo-url> && cd aes-rust-plugin
dbx-plugin dev --path . --port 5190
```

CLI 会自动编译 Rust 后端并启动前端界面。开发凭证存储在 `.dbx-dev/` 目录中（已 git 忽略）。

## 使用说明

### 连接配置

创建连接时，可配置以下字段：

| 字段 | 说明 | 默认值 |
|------|------|--------|
| 主机 | 目标服务器地址 | `localhost` |
| 端口 | 目标服务器端口 | `8080` |
| aes 密钥 | 默认 AES 密钥（16 字节） | `1234567890123456` |
| aes 密钥标签 | 密钥标签列表（JSON） | 见下方 |

**密钥标签格式**：
```json
[{"label":"默认密钥","value":"default_key","aes_key":"1234567890123456"}]
```

### API 方法

插件通过 DBX Host API 暴露以下方法：

| 方法 | 说明 |
|------|------|
| `aes-rust-plugin/ping` | 健康检查 |
| `aes-rust-plugin/aesEncrypt` | AES 加密 |
| `aes-rust-plugin/aesDecrypt` | AES 解密 |
| `aes-rust-plugin/aesKey` | 获取当前连接的 AES 密钥 |
| `aes-rust-plugin/aesKeyLables` | 获取密钥标签配置 |

## 构建打包

```bash
bash scripts/build.sh       # 构建并打包为当前平台的 .dbxp
dbx-plugin package .        # 或直接使用 CLI 打包
```

产物输出到 `dist/` 目录。

## 项目结构

```
├── manifest.json          # 插件清单（v1）
├── dbx-plugin.toml        # 构建配置
├── backend/               # Rust 后端
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs        # 插件处理器 & IPC 服务
│       └── aes_util.rs    # AES-128-ECB 加解密实现
├── ui/
│   └── index.html         # shadcn/ui 风格工作台
├── assets/
│   └── plugin.svg         # 插件图标
├── scripts/               # 构建 & 开发脚本
└── shared/sdk/            # DBX Plugin SDK
```

## 技术栈

- **后端**：Rust + `dbx-plugin-sdk`（sidecar IPC 通信）
- **加密**：`aes` + `ecb` + `base64` crates
- **前端**：原生 HTML/CSS/JS，shadcn/ui 设计变量

## 发布流程

1. 发布 GitHub Release —— CI 将构建所有平台的 `.dbxp` 产物。
2. 向 [`t8y2/dbx-store`](https://github.com/t8y2/dbx-store) 提交候选 PR。
3. DBX Store 审核通过后进行签名并发布。

## 许可证

MIT
