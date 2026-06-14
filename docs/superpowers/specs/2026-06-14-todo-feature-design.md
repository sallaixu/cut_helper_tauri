# Todo 待办功能设计

## 概述

为剪切板助手新增"待办"功能，支持带时间安排的 todo 管理，包含系统通知、状态流转、备注、冲突检测等能力。

## 需求

- 新增"待办"Tab，按天切换显示当日待办
- 每个 todo 支持：标题、备注、开始时间、结束时间/持续时长、状态
- 状态流转：未开始(pending) → 进行中(ongoing) → 已完成(done)
- 系统通知：开始时间提醒、结束时间提醒、提前提醒
- 冲突检测：时间区间重叠时警告提示，允许保存
- 通知在任何 Tab 下都能触发（不受 Tab 切换影响）

## 数据层

### 新增表 TodoItems（需数据库迁移）

```sql
CREATE TABLE IF NOT EXISTS "TodoItems" (
  "id" UUID NOT NULL PRIMARY KEY,
  "title" VARCHAR(255) NOT NULL,
  "note" TEXT,
  "status" VARCHAR(20) NOT NULL DEFAULT 'pending',
  "startTime" DATETIME,
  "endTime" DATETIME,
  "duration" INTEGER,
  "notifyStart" BOOLEAN NOT NULL DEFAULT 1,
  "notifyEnd" BOOLEAN NOT NULL DEFAULT 1,
  "notifyAdvance" INTEGER NOT NULL DEFAULT 5,
  "createTime" DATETIME NOT NULL,
  "updateTime" DATETIME
);
```

**时间模型（两种都支持）：**
- 开始时间 + 结束时间：`startTime` 和 `endTime` 有值，`duration` 可为空
- 开始时间 + 持续时长：`startTime` 和 `duration` 有值，`endTime` 自动算出存储

**状态值：** `pending`（未开始）、`ongoing`（进行中）、`done`（已完成）

**通知设置：**
- `notifyStart`：是否在开始时间发通知（默认 true）
- `notifyEnd`：是否在结束时间发通知（默认 true）
- `notifyAdvance`：提前提醒分钟数（默认 5，0 表示不提前提醒）

### db_service.js 新增方法

| 方法 | 功能 |
|------|------|
| `addTodoItem(item)` | 新增待办 |
| `fetchTodoItems()` | 查询所有待办（按开始时间排序） |
| `fetchTodoItemsByDate(date)` | 查询指定日期的待办 |
| `updateTodoItem(id, fields)` | 更新待办（状态、备注等） |
| `removeTodoItem(id)` | 删除待办 |
| `checkTodoConflict(startTime, endTime, excludeId)` | 检查时间冲突（返回冲突的 todo 列表） |

## 通知机制

### 核心原则：定时器逻辑独立于组件生命周期

定时器放在全局模块 `src/todo_notifier.js` 中，在 `main.js` 初始化时调用，不受 Tab 切换影响。

### 依赖新增

- Rust 侧：`tauri-plugin-notification = "2"`（Cargo.toml）
- 前端：`@tauri-apps/plugin-notification`（npm install）
- 权限：`default.json` 中添加 `notification:default`

### todo_notifier.js 模块

```
todo_notifier.js（全局模块）
├── init()           — 应用启动时：加载未完成 todo，注册所有定时器
├── register(todo)   — 添加/修改 todo 时：注册定时器
├── unregister(id)   — 删除/完成 todo 时：清除所有定时器
├── refreshAll()     — 全量刷新：重新从 DB 加载并注册
```

**内部维护 timerMap：**
```javascript
// key: todoId, value: [timerId1, timerId2, timerId3]
const timerMap = new Map()
```

**每个 todo 最多注册 3 个 setTimeout：**
1. 提前提醒（`notifyAdvance > 0` 时，`startTime - notifyAdvance` 分钟）
2. 开始提醒（`notifyStart = true` 时，`startTime` 到达）
3. 结束提醒（`notifyEnd = true` 时，`endTime` 到达）

**已过期的时间不注册定时器**（避免立即触发）。

**通知内容：**
- 开始提醒：「📋 待办开始：{title}」
- 结束提醒：「✅ 待办结束：{title}」
- 提前提醒：「⏰ 即将开始：{title}（{notifyAdvance}分钟后）」

**初始化位置：** `main.js` 中调用 `todoNotifier.init()`

**组件交互：** TodoPage.vue 添加/修改/删除/完成 todo 时调用 `todoNotifier.register()` / `todoNotifier.unregister()`

## UI 结构

### 待办 Tab（TodoPage.vue）

在 CutPage.vue 的 Tabs 中新增"待办"Tab（位于收藏和白板之间）。

**布局：**

```
┌──────────────────────────────────┐
│ [文本] [图片] [收藏] [待办] [白板] │
├──────────────────────────────────┤
│  ←  2026年6月14日 周六  →        │  ← 日期切换器
│  [+ 新建]          筛选:[全部 ▼]  │  ← 操作栏
├──────────────────────────────────┤
│  ⚠️ 14:00-16:00 会议讨论         │
│     进行中                        │
│  ──────────────────────────────  │
│  📋 17:00 写周报                 │
│     17:00→18:00 | 未开始          │
│  ──────────────────────────────  │
│  ✅ 10:00 晨会                   │
│     10:00→10:30 | 已完成          │
└──────────────────────────────────┘
```

**日期切换器：**
- `< 日期 >` 导航，支持左右切换
- 显示格式：`2026年6月14日 周六`
- 默认显示今天
- 列表只显示 startTime 落在选中日期的 todo

**筛选下拉：** 全部 / 未开始 / 进行中 / 已完成

**待办列表：**
- 每条显示：标题 + 时间范围 + 状态标签
- 有时间冲突的条目前显示 ⚠️ 警告标记
- 操作菜单：编辑 / 标记完成 / 删除

### 新建/编辑弹窗

```
┌── 新建待办 ──────────────────────┐
│  标题: [______________________]    │
│  备注: [______________________]    │
│                                   │
│  开始时间: [2026-06-14 14:00]     │
│  结束时间: [2026-06-14 16:00]  ☑  │
│  或持续时长: [___] 分钟           │
│  ⚠️ 与「项目评审」时间重叠         │
│                                   │
│  通知: ☑开始提醒  ☑结束提醒       │
│  提前提醒: [5] 分钟               │
│                                   │
│          [取消]  [保存]           │
└───────────────────────────────────┘
```

**时间设置区域：**
- 可输入「开始时间 + 结束时间」或「开始时间 + 持续时长」，二选一
- 输入结束时间后自动计算持续时长，反之亦然
- 保存时自动检测时间冲突，显示警告但允许保存

## 冲突检测

**检测时机：** 保存（新建/编辑）待办时

**重叠条件：** `startTimeA < endTimeB AND startTimeB < endTimeA`

**实现：** `db_service.checkTodoConflict(startTime, endTime, excludeId)` 返回冲突 todo 列表

**行为：** 显示警告 ⚠️ 但允许保存

**例外：** 无 endTime 的 todo 不参与冲突检测

## 修改文件清单

| 文件 | 变更 |
|------|------|
| `src-tauri/Cargo.toml` | 新增 tauri-plugin-notification 依赖 |
| `src-tauri/src/lib.rs` | 新增 TodoItems 表迁移 + 注册 notification 插件 |
| `src-tauri/capabilities/default.json` | 新增 notification 权限 |
| `src/db_service.js` | 新增 6 个 todo CRUD 方法 |
| `src/todo_notifier.js` | 新建：全局通知调度器 |
| `src/components/TodoPage.vue` | 新建：待办 Tab 组件 |
| `src/components/CutPage.vue` | 新增"待办"Tab |
| `src/main.js` | 初始化 todo_notifier |
| `package.json` | 新增 @tauri-apps/plugin-notification |
