# Nexus Launcher

中文 | [English](README_en.md)

一款使用 Rust 编写的高性能命令行 Minecraft 启动器。

这是一个非官方启动器，与 Mojang 或 Microsoft 无关。

## 特性

- **Java 管理**：自动检测并下载所需的 Java 版本（默认为 17）。
- **游戏安装**：异步下载核心 JAR、库文件和资源文件。
- **Mod 加载器支持**：内置对 Fabric 和 Quilt 安装的支持。
- **身份验证**：支持 Microsoft 在线登录和离线模式。
- **高性能**：轻量且快速，基于 Tokio 运行时构建。

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

- `launch <instance>` - 启动游戏实例（选项：`--offline`, `--max-memory`, `--force-scan`）
- `install core --game-version <V>` - 下载特定的 Minecraft 版本
- `install loader <instance> --loader <fabric|quilt>` - 安装 Mod 加载器
- `install mod --query <Q> --game-version <V>` - 搜索并安装 Mod
- `java --scan` - 扫描本地系统的 Java 安装
- `java --download --version <N>` - 下载特定的 Java 运行时
- `auth --login` - 使用 Microsoft 身份验证
- `set --show` - 显示当前配置和设置

## 贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解准则。

## 团队

我们是一群热爱 Minecraft 和编程的朋友。我们目前正在开发一款高性能、用户友好的 Minecraft 启动器。无论你是开发者还是游戏爱好者，都欢迎加入我们的社区与我们交流！

这是我们的 `Discord` 社区：[链接](https://discord.gg/gM85PKSYEe)

## 许可证

GPL-3.0
