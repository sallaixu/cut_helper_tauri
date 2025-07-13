# ✂️ 剪贴小助手

> 🚀 **开源、离线、超轻量** 的剪贴板历史管理工具  
> 🦀 基于 Tauri + Vue3，体积 < 5 MB，无需联网，随叫随到！

---

## ✨ 功能亮点
- 📜 **历史保存** – 复制过的内容可快速找回
- ⚡ **极速唤醒** – 全局热键弹出\隐藏面板
- 🧊 **完全离线** – 本地存储，零隐私泄露
- 🎨 **主题切换** – 明暗两种界面（todo），支持模糊搜索 / 置顶 

---

## 🛠️ 快速上手

```bash
# 克隆仓库
git clone https://github.com/sallaixu/cut_helper_tauri.git
cd cut_helper_tauri/src-tauri

# 安装依赖
pnpm install          # 也可使用 yarn 或 npm

# 开发运行
pnpm tauri dev

# 构建正式版
pnpm tauri build

```

---
## 🎯 默认快捷键

| 热键                | 功能        |
| ------------------ | ------------- |
| `Ctrl + SPACE`     | 唤起 / 隐藏 面板（全局） |
| `Ctrl + F`         |  焦点到面板的搜索框 （软件内） |
| `方向键`         |   切换列表剪切板列表项 （列表获取焦点） |
| `回车`         |   搜索框获取焦点情况下回车让列表获取焦点  |


---

## 📦 下载安装

前往 [Releases](https://github.com/sallaixu/cut_helper_tauri/releases)  
获取 Windows / macOS / Linux 的**绿色安装包**。

---

## 🤝 参与贡献

欢迎 PR！请先阅读 [CONTRIBUTING.md](./CONTRIBUTING.md)。

## 📄 开源协议

MIT License © [sallai](https://github.com/sallaixu)  
```
