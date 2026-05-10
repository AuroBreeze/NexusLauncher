# Nexus Launcher

中文 | [English](README_en.md)

一款使用 Rust 编写的高性能命令行 Minecraft 启动器。

这是一个非官方启动器，与 Mojang 或 Microsoft 无关。

## 特性

- **游戏安装**：异步下载核心 JAR、库文件和资源文件，支持 SHA1 校验和断点续传
- **Java 管理**：自动扫描系统 Java、缓存路径、支持从 Adoptium 下载 JRE
- **Mod 搜索**：集成 Modrinth API，支持全文搜索、分面过滤、排序和分页
- **Mod 依赖解析**：获取项目完整依赖树和版本列表
- **Mod 加载器**：内置 Fabric 安装和启动支持
- **身份验证**：Microsoft 设备码 OAuth 登录 + 离线模式
- **高性能**：轻量且快速，基于 Tokio 异步运行时构建

## 安装

```bash
git clone https://github.com/AuroBreeze/NexusLauncher.git
cd NexusLauncher
cargo build --release
```

## 使用方法

### 1. 安装游戏版本
```bash
cargo run -- install core --game-version 1.20.1
```

### 2. 安装加载器（可选）
```bash
cargo run -- install loader 1.20.1 --loader fabric
```

### 3. 启动游戏
```bash
cargo run -- launch 1.20.1
```

## 命令参考

### 核心
- `launch <instance>` — 启动游戏（`--offline`, `--max-memory`, `--force-scan`）
- `install core --game-version <V>` — 下载版本（`--name` 自定义目录名）
- `install loader <instance> --loader <fabric|quilt>` — 安装 Mod 加载器
- `install mod --query <Q>` — 搜索 Mod（`-g` 版本过滤, `-l` 数量）

### 搜索
- `search mod <query>` — Modrinth 全文搜索（`-l` 数量, `-g` 版本, `-i` 排序, `-o` 分页）
- `search java` — 列出已安装 Java（`-s` 扫描, `-v` 过滤版本）
- `search user <instance>` — 读取游戏实例的玩家缓存

### 认证与配置
- `auth --login` — Microsoft 设备码登录
- `auth --logout <name>` — 清除凭据
- `set -n <name> -u <uuid>` — 设置离线用户名/UUID
- `set --show` — 显示当前配置

## 贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解准则。

## 团队

我们是一群热爱 Minecraft 和编程的朋友。我们目前正在开发一款高性能、用户友好的 Minecraft 启动器。无论你是开发者还是游戏爱好者，都欢迎加入我们的社区与我们交流！

这是我们的 `Discord` 社区：[链接](https://discord.gg/gM85PKSYEe)

## 许可证

GPL-3.0
