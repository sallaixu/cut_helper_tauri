# Todo Feature Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a "待办" tab with time-based todo management, system notifications, status tracking, and conflict detection to the clipboard assistant.

**Architecture:** Database operations through `db_service.js` (existing pattern). Notifications via `tauri-plugin-notification` (new Rust plugin). Timer scheduling in a global `todo_notifier.js` module (independent of component lifecycle). TodoPage.vue uses date-switcher + single-column list layout.

**Tech Stack:** Vue 3 (Composition API), Ant Design Vue, @tauri-apps/plugin-notification, tauri-plugin-notification, SQLite

---

### Task 1: Add tauri-plugin-notification dependency and permissions

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`

- [ ] **Step 1: Add Rust dependency**

In `src-tauri/Cargo.toml`, after the `tauri-plugin-autostart = "2"` line, add:

```toml
tauri-plugin-notification = "2"
```

- [ ] **Step 2: Register plugin in lib.rs**

In `src-tauri/src/lib.rs`, in the `tauri::Builder::default()` chain, after `.plugin(tauri_plugin_autostart::init(...))`, add:

```rust
.plugin(tauri_plugin_notification::init())
```

- [ ] **Step 3: Add notification permission**

In `src-tauri/capabilities/default.json`, add `"notification:default"` to the `permissions` array.

- [ ] **Step 4: Install frontend notification package**

Run: `cd "D:/project/view/cut" && npm install @tauri-apps/plugin-notification`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs src-tauri/capabilities/default.json package.json package-lock.json
git commit -m "feat: add tauri-plugin-notification dependency and permissions"
```

---

### Task 2: Add TodoItems database migration and db_service methods

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/db_service.js`

- [ ] **Step 1: Add TodoItems table migration**

In `src-tauri/src/lib.rs`, in the `migrations` vector, after the version 4 migration, add a new migration:

```rust
        // 版本5 - 添加待办事项表
        Migration {
            version: 5,
            description: "add_todo_items_table",
            sql: r#"
            CREATE TABLE IF NOT EXISTS "TodoItems" (
                "id" UUID NOT NULL,
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
                "updateTime" DATETIME,
                PRIMARY KEY ("id")
            );
            "#,
            kind: MigrationKind::Up,
        },
```

- [ ] **Step 2: Add todo methods to db_service.js**

In `src/db_service.js`, after the existing `removeGroupItem` method and before the `getCurrentConfig` method, add:

```javascript
  // 待办相关方法
  async addTodoItem(item) {
    await this.init();
    const id = uuidv4();
    const createTime = new Date().toISOString();

    // 如果有持续时长但没有结束时间，自动计算
    let endTime = item.endTime || null;
    if (!endTime && item.startTime && item.duration) {
      endTime = new Date(new Date(item.startTime).getTime() + item.duration * 60000).toISOString();
    }

    try {
      await db.execute(
        'INSERT INTO TodoItems (id, title, note, status, startTime, endTime, duration, notifyStart, notifyEnd, notifyAdvance, createTime) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)',
        [id, item.title, item.note || '', item.status || 'pending', item.startTime || null, endTime, item.duration || null, item.notifyStart !== undefined ? (item.notifyStart ? 1 : 0) : 1, item.notifyEnd !== undefined ? (item.notifyEnd ? 1 : 0) : 1, item.notifyAdvance !== undefined ? item.notifyAdvance : 5, createTime]
      );
      return { id, title: item.title, note: item.note, status: item.status || 'pending', startTime: item.startTime, endTime, duration: item.duration, notifyStart: item.notifyStart !== undefined ? item.notifyStart : true, notifyEnd: item.notifyEnd !== undefined ? item.notifyEnd : true, notifyAdvance: item.notifyAdvance !== undefined ? item.notifyAdvance : 5, createTime };
    } catch (error) {
      console.error('Error adding todo item:', error);
      return null;
    }
  },

  async fetchTodoItems() {
    await this.init();
    try {
      const result = await db.select('SELECT * FROM TodoItems ORDER BY startTime ASC');
      return result || [];
    } catch (error) {
      console.error('Error fetching todo items:', error);
      return [];
    }
  },

  async fetchTodoItemsByDate(dateStr) {
    await this.init();
    try {
      // dateStr format: '2026-06-14', query todos whose startTime falls on that date
      const nextDay = new Date(dateStr);
      nextDay.setDate(nextDay.getDate() + 1);
      const nextDayStr = nextDay.toISOString().split('T')[0];

      const result = await db.select(
        'SELECT * FROM TodoItems WHERE startTime >= ? AND startTime < ? ORDER BY startTime ASC',
        [dateStr, nextDayStr]
      );
      return result || [];
    } catch (error) {
      console.error('Error fetching todo items by date:', error);
      return [];
    }
  },

  async updateTodoItem(id, fields) {
    await this.init();
    try {
      // If duration changed but endTime not provided, recalculate endTime
      if (fields.duration !== undefined && fields.startTime && !fields.endTime) {
        fields.endTime = new Date(new Date(fields.startTime).getTime() + fields.duration * 60000).toISOString();
      }

      const setClauses = [];
      const values = [];

      for (const [key, value] of Object.entries(fields)) {
        if (key === 'id' || key === 'createTime') continue;
        setClauses.push(`${key} = ?`);
        values.push(value);
      }

      if (setClauses.length === 0) return true;

      values.push(new Date().toISOString());
      setClauses.push('updateTime = ?');

      values.push(id);

      await db.execute(
        `UPDATE TodoItems SET ${setClauses.join(', ')} WHERE id = ?`,
        values
      );
      return true;
    } catch (error) {
      console.error('Error updating todo item:', error);
      return false;
    }
  },

  async removeTodoItem(id) {
    await this.init();
    try {
      await db.execute('DELETE FROM TodoItems WHERE id = ?', [id]);
      return true;
    } catch (error) {
      console.error('Error removing todo item:', error);
      return false;
    }
  },

  async checkTodoConflict(startTime, endTime, excludeId = null) {
    await this.init();
    if (!startTime || !endTime) return [];

    try {
      let sql = 'SELECT * FROM TodoItems WHERE status != ? AND startTime IS NOT NULL AND endTime IS NOT NULL AND startTime < ? AND endTime > ?';
      const params = ['done', endTime, startTime];

      if (excludeId) {
        sql += ' AND id != ?';
        params.push(excludeId);
      }

      const result = await db.select(sql, params);
      return result || [];
    } catch (error) {
      console.error('Error checking todo conflict:', error);
      return [];
    }
  },
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs src/db_service.js
git commit -m "feat: add TodoItems table migration and db_service CRUD methods"
```

---

### Task 3: Create todo_notifier.js global notification scheduler

**Files:**
- Create: `src/todo_notifier.js`
- Modify: `src/main.js`

- [ ] **Step 1: Create todo_notifier.js**

Create `src/todo_notifier.js` with the following content:

```javascript
import dbService from './db_service';
import { sendNotification, isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';

// 定时器映射: todoId -> [timerId1, timerId2, timerId3]
const timerMap = new Map();

/**
 * 请求通知权限
 */
async function ensurePermission() {
  try {
    let permitted = await isPermissionGranted();
    if (!permitted) {
      const permission = await requestPermission();
      permitted = permission === 'granted';
    }
    return permitted;
  } catch (error) {
    console.error('通知权限请求失败:', error);
    return false;
  }
}

/**
 * 发送系统通知
 */
function notify(title, body) {
  try {
    sendNotification({ title, body });
  } catch (error) {
    console.error('发送通知失败:', error);
  }
}

/**
 * 为单个 todo 注册定时器
 */
function registerTimers(todo) {
  // 先清除该 todo 已有的定时器
  unregisterTimers(todo.id);

  if (todo.status === 'done') return;

  const timers = [];
  const now = Date.now();

  // 提前提醒
  if (todo.startTime && todo.notifyAdvance > 0) {
    const advanceTime = new Date(todo.startTime).getTime() - todo.notifyAdvance * 60000;
    const delay = advanceTime - now;
    if (delay > 0) {
      timers.push(setTimeout(() => {
        notify('⏰ 即将开始', `${todo.title}（${todo.notifyAdvance}分钟后）`);
      }, delay));
    }
  }

  // 开始提醒
  if (todo.startTime && todo.notifyStart) {
    const delay = new Date(todo.startTime).getTime() - now;
    if (delay > 0) {
      timers.push(setTimeout(() => {
        notify('📋 待办开始', todo.title);
      }, delay));
    }
  }

  // 结束提醒
  if (todo.endTime && todo.notifyEnd) {
    const delay = new Date(todo.endTime).getTime() - now;
    if (delay > 0) {
      timers.push(setTimeout(() => {
        notify('✅ 待办结束', todo.title);
      }, delay));
    }
  }

  if (timers.length > 0) {
    timerMap.set(todo.id, timers);
  }
}

/**
 * 清除单个 todo 的所有定时器
 */
function unregisterTimers(todoId) {
  const timers = timerMap.get(todoId);
  if (timers) {
    timers.forEach(clearTimeout);
    timerMap.delete(todoId);
  }
}

/**
 * 初始化：加载所有未完成的 todo 并注册定时器
 */
async function init() {
  await ensurePermission();

  try {
    const todos = await dbService.fetchTodoItems();
    const activeTodos = todos.filter(t => t.status !== 'done');
    activeTodos.forEach(registerTimers);
    console.log(`通知调度器已初始化，注册了 ${activeTodos.length} 个待办通知`);
  } catch (error) {
    console.error('通知调度器初始化失败:', error);
  }
}

/**
 * 注册单个 todo 的通知（添加/修改时调用）
 */
function register(todo) {
  registerTimers(todo);
}

/**
 * 注销单个 todo 的通知（删除/完成时调用）
 */
function unregister(todoId) {
  unregisterTimers(todoId);
}

/**
 * 全量刷新：清除所有定时器，重新从数据库加载
 */
async function refreshAll() {
  // 清除所有定时器
  for (const timers of timerMap.values()) {
    timers.forEach(clearTimeout);
  }
  timerMap.clear();

  // 重新加载
  await init();
}

export default { init, register, unregister, refreshAll };
```

- [ ] **Step 2: Initialize notifier in main.js**

In `src/main.js`, add the import after the existing imports:

```javascript
import todoNotifier from './todo_notifier'
```

Then in the `main()` function, after the `if (windowLabel === 'main')` block (after the closing `}`), add:

```javascript
    // 初始化待办通知调度器
    todoNotifier.init();
```

The full main function should look like:

```javascript
async function main() {
    const currentWindow = getCurrentWebviewWindow();
    const windowLabel = currentWindow.label;

    if (windowLabel === 'main') {
        await init_hotkey();
        start();
    }

    // 初始化待办通知调度器
    todoNotifier.init();

    const app = createApp(App);
    app.use(router);
    app.mount("#app");
}
```

- [ ] **Step 3: Commit**

```bash
git add src/todo_notifier.js src/main.js
git commit -m "feat: create todo_notifier.js global notification scheduler and initialize in main.js"
```

---

### Task 4: Create TodoPage.vue component

**Files:**
- Create: `src/components/TodoPage.vue`

- [ ] **Step 1: Create TodoPage.vue**

Create `src/components/TodoPage.vue` with the following content:

```vue
<template>
  <div class="todo-page">
    <!-- 日期切换器 -->
    <div class="todo-date-bar">
      <a-button size="small" @click="changeDate(-1)">
        <template #icon><LeftOutlined /></template>
      </a-button>
      <span class="todo-date-text" @click="goToToday">{{ displayDate }}</span>
      <a-button size="small" @click="changeDate(1)">
        <template #icon><RightOutlined /></template>
      </a-button>
    </div>

    <!-- 操作栏 -->
    <div class="todo-toolbar">
      <a-button type="primary" size="small" @click="showAddModal">
        <template #icon><PlusOutlined /></template>
        新建
      </a-button>
      <a-select v-model:value="statusFilter" size="small" style="width: 90px; margin-left: auto;" @change="filterTodos">
        <a-select-option value="all">全部</a-select-option>
        <a-select-option value="pending">未开始</a-select-option>
        <a-select-option value="ongoing">进行中</a-select-option>
        <a-select-option value="done">已完成</a-select-option>
      </a-select>
    </div>

    <!-- 待办列表 -->
    <div class="todo-list" ref="scrollerRef">
      <a-back-top :target="() => document.getElementById('todoItemBox')" />

      <template v-if="filteredTodos.length > 0">
        <virt-list
          class="scroller"
          :list="filteredTodos"
          itemKey="id"
          :minSize="60"
          id="todoItemBox"
          ref="virtListRef"
        >
          <template #default="{ itemData }">
            <div class="todo-item" :class="{ 'todo-item-done': itemData.status === 'done' }">
              <!-- 冲突标记 + 标题 -->
              <div class="todo-item-header">
                <span v-if="itemData._conflict" class="todo-conflict" title="时间冲突">⚠️</span>
                <span class="todo-title">{{ itemData.title }}</span>
                <a-tag :color="statusColor(itemData.status)" size="small" style="margin-left: auto;">
                  {{ statusText(itemData.status) }}
                </a-tag>
              </div>

              <!-- 时间 + 操作 -->
              <div class="todo-item-footer">
                <span class="todo-time">
                  <ClockCircleOutlined style="margin-right: 4px;" />
                  {{ formatTime(itemData.startTime) }}
                  <template v-if="itemData.endTime"> → {{ formatTime(itemData.endTime) }}</template>
                </span>
                <div class="todo-item-actions">
                  <a-button v-if="itemData.status !== 'done'" type="link" size="small" @click="markDone(itemData)">
                    <CheckOutlined />
                  </a-button>
                  <a-dropdown :trigger="['click']">
                    <MoreOutlined style="cursor: pointer;" @click.prevent />
                    <template #overlay>
                      <a-menu>
                        <a-menu-item v-if="itemData.status === 'pending'" @click="markOngoing(itemData)" key="ongoing">
                          <PlayCircleOutlined /><span style="margin-left: 8px;">开始</span>
                        </a-menu-item>
                        <a-menu-item v-if="itemData.status !== 'done'" @click="markDone(itemData)" key="done">
                          <CheckOutlined /><span style="margin-left: 8px;">完成</span>
                        </a-menu-item>
                        <a-menu-item @click="showEditModal(itemData)" key="edit">
                          <EditOutlined /><span style="margin-left: 8px;">编辑</span>
                        </a-menu-item>
                        <a-menu-item @click="deleteTodo(itemData)" key="delete" style="color: #f5222d;">
                          <DeleteOutlined /><span style="margin-left: 8px;">删除</span>
                        </a-menu-item>
                      </a-menu>
                    </template>
                  </a-dropdown>
                </div>
              </div>
            </div>
          </template>
        </virt-list>
      </template>

      <!-- 空状态 -->
      <div v-else class="todo-empty">
        <a-empty :description="currentDate === todayStr ? '今天暂无待办' : '该日期暂无待办'" />
      </div>
    </div>

    <!-- 新建/编辑弹窗 -->
    <a-modal
      v-model:open="modalVisible"
      :title="editingId ? '编辑待办' : '新建待办'"
      ok-text="保存"
      cancel-text="取消"
      @ok="handleSave"
      :ok-button-props="{ disabled: !formData.title.trim() }"
    >
      <a-form layout="vertical" size="small">
        <a-form-item label="标题" required>
          <a-input v-model:value="formData.title" placeholder="待办标题" />
        </a-form-item>

        <a-form-item label="备注">
          <a-textarea v-model:value="formData.note" placeholder="备注信息" :rows="2" />
        </a-form-item>

        <a-form-item label="开始时间">
          <a-date-picker
            v-model:value="formData.startTime"
            show-time
            format="YYYY-MM-DD HH:mm"
            placeholder="选择开始时间"
            style="width: 100%;"
            @change="onStartTimeChange"
          />
        </a-form-item>

        <a-form-item label="结束方式">
          <a-radio-group v-model:value="endMode" @change="onEndModeChange">
            <a-radio value="endTime">指定结束时间</a-radio>
            <a-radio value="duration">指定持续时长</a-radio>
          </a-radio-group>
        </a-form-item>

        <a-form-item v-if="endMode === 'endTime'" label="结束时间">
          <a-date-picker
            v-model:value="formData.endTime"
            show-time
            format="YYYY-MM-DD HH:mm"
            placeholder="选择结束时间"
            style="width: 100%;"
            @change="onEndTimeChange"
          />
        </a-form-item>

        <a-form-item v-if="endMode === 'duration'" label="持续时长（分钟）">
          <a-input-number v-model:value="formData.duration" :min="1" placeholder="如 60" style="width: 100%;" @change="onDurationChange" />
        </a-form-item>

        <!-- 冲突警告 -->
        <a-alert v-if="conflictList.length > 0" type="warning" show-icon style="margin-bottom: 12px;">
          <template #message>时间冲突</template>
          <template #description>
            <div v-for="c in conflictList" :key="c.id">• {{ c.title }} ({{ formatTime(c.startTime) }} → {{ formatTime(c.endTime) }})</div>
          </template>
        </a-alert>

        <a-divider style="margin: 8px 0;" />

        <a-form-item label="通知设置">
          <div style="display: flex; flex-wrap: wrap; gap: 8px; align-items: center;">
            <a-checkbox v-model:checked="formData.notifyStart">开始提醒</a-checkbox>
            <a-checkbox v-model:checked="formData.notifyEnd">结束提醒</a-checkbox>
            <span style="margin-left: 8px;">提前</span>
            <a-input-number v-model:value="formData.notifyAdvance" :min="0" :max="120" size="small" style="width: 60px;" />
            <span>分钟</span>
          </div>
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { VirtList } from 'vue-virt-list';
import {
  PlusOutlined, LeftOutlined, RightOutlined,
  MoreOutlined, DeleteOutlined, EditOutlined,
  CheckOutlined, ClockCircleOutlined, PlayCircleOutlined
} from '@ant-design/icons-vue';
import { message } from 'ant-design-vue';
import dayjs from 'dayjs';
import dbService from '../db_service';
import todoNotifier from '../todo_notifier';

message.config({ top: '50px', duration: 1, maxCount: 2 });

// ==================== 日期相关 ====================
const todayStr = computed(() => dayjs().format('YYYY-MM-DD'));
const currentDate = ref(todayStr.value);

const displayDate = computed(() => {
  const d = dayjs(currentDate.value);
  const weekdays = ['日', '一', '二', '三', '四', '五', '六'];
  return `${d.format('YYYY年M月D日')} 周${weekdays[d.day()]}`;
});

const changeDate = (offset) => {
  currentDate.value = dayjs(currentDate.value).add(offset, 'day').format('YYYY-MM-DD');
};

const goToToday = () => {
  currentDate.value = todayStr.value;
};

// ==================== 列表数据 ====================
const todoList = ref([]);
const statusFilter = ref('all');
const virtListRef = ref(null);

const filteredTodos = computed(() => {
  let list = todoList.value;
  if (statusFilter.value !== 'all') {
    list = list.filter(t => t.status === statusFilter.value);
  }
  return list;
});

const filterTodos = () => {
  // computed 自动处理
};

// ==================== 数据操作 ====================
const queryTodos = async () => {
  const result = await dbService.fetchTodoItemsByDate(currentDate.value);
  todoList.value = (result || []).map(t => ({ ...t, _conflict: false }));
  checkConflicts();
};

const checkConflicts = async () => {
  // 对每个有时间区间的 todo 检测冲突
  for (const todo of todoList.value) {
    if (todo.startTime && todo.endTime && todo.status !== 'done') {
      const conflicts = await dbService.checkTodoConflict(todo.startTime, todo.endTime, todo.id);
      todo._conflict = conflicts.length > 0;
    }
  }
};

watch(currentDate, () => {
  queryTodos();
});

onMounted(() => {
  queryTodos();
});

// ==================== 状态操作 ====================
const markDone = async (item) => {
  const success = await dbService.updateTodoItem(item.id, { status: 'done' });
  if (success) {
    item.status = 'done';
    todoNotifier.unregister(item.id);
    message.success('已标记完成');
  }
};

const markOngoing = async (item) => {
  const success = await dbService.updateTodoItem(item.id, { status: 'ongoing' });
  if (success) {
    item.status = 'ongoing';
    message.success('已标记进行中');
  }
};

const deleteTodo = async (item) => {
  const success = await dbService.removeTodoItem(item.id);
  if (success) {
    todoList.value = todoList.value.filter(t => t.id !== item.id);
    todoNotifier.unregister(item.id);
    message.success('删除成功');
  }
};

// ==================== 新建/编辑弹窗 ====================
const modalVisible = ref(false);
const editingId = ref(null);
const endMode = ref('endTime');
const conflictList = ref([]);

const formData = ref({
  title: '',
  note: '',
  startTime: null,
  endTime: null,
  duration: null,
  notifyStart: true,
  notifyEnd: true,
  notifyAdvance: 5,
});

const resetForm = () => {
  formData.value = {
    title: '',
    note: '',
    startTime: null,
    endTime: null,
    duration: null,
    notifyStart: true,
    notifyEnd: true,
    notifyAdvance: 5,
  };
  endMode.value = 'endTime';
  conflictList.value = [];
  editingId.value = null;
};

const showAddModal = () => {
  resetForm();
  modalVisible.value = true;
};

const showEditModal = (item) => {
  editingId.value = item.id;
  formData.value = {
    title: item.title,
    note: item.note || '',
    startTime: item.startTime ? dayjs(item.startTime) : null,
    endTime: item.endTime ? dayjs(item.endTime) : null,
    duration: item.duration || null,
    notifyStart: !!item.notifyStart,
    notifyEnd: !!item.notifyEnd,
    notifyAdvance: item.notifyAdvance || 0,
  };
  endMode.value = item.duration && !item.endTime ? 'duration' : 'endTime';
  conflictList.value = [];
  modalVisible.value = true;
};

const onStartTimeChange = () => {
  if (endMode.value === 'duration' && formData.value.startTime && formData.value.duration) {
    calcEndTime();
  }
  checkConflictLive();
};

const onEndTimeChange = () => {
  if (formData.value.startTime && formData.value.endTime) {
    const diff = formData.value.endTime.diff(formData.value.startTime, 'minute');
    formData.value.duration = diff > 0 ? diff : null;
  }
  checkConflictLive();
};

const onDurationChange = () => {
  if (formData.value.startTime && formData.value.duration) {
    calcEndTime();
  }
  checkConflictLive();
};

const onEndModeChange = () => {
  if (endMode.value === 'duration' && formData.value.startTime && formData.value.duration) {
    calcEndTime();
  }
  checkConflictLive();
};

const calcEndTime = () => {
  formData.value.endTime = formData.value.startTime.add(formData.value.duration, 'minute');
};

const checkConflictLive = async () => {
  const st = formData.value.startTime ? formData.value.startTime.toISOString() : null;
  const et = formData.value.endTime ? formData.value.endTime.toISOString() : null;

  if (!st || !et) {
    conflictList.value = [];
    return;
  }

  const conflicts = await dbService.checkTodoConflict(st, et, editingId.value);
  conflictList.value = conflicts;
};

const handleSave = async () => {
  if (!formData.value.title.trim()) return;

  const st = formData.value.startTime ? formData.value.startTime.toISOString() : null;
  const et = formData.value.endTime ? formData.value.endTime.toISOString() : null;

  const itemData = {
    title: formData.value.title.trim(),
    note: formData.value.note.trim(),
    startTime: st,
    endTime: et,
    duration: formData.value.duration || null,
    notifyStart: formData.value.notifyStart,
    notifyEnd: formData.value.notifyEnd,
    notifyAdvance: formData.value.notifyAdvance || 0,
  };

  if (editingId.value) {
    // 编辑
    const success = await dbService.updateTodoItem(editingId.value, {
      ...itemData,
      startTime: st,
      endTime: et,
    });
    if (success) {
      await queryTodos();
      // 重新注册通知
      const updated = todoList.value.find(t => t.id === editingId.value);
      if (updated) todoNotifier.register(updated);
      message.success('保存成功');
    }
  } else {
    // 新建
    itemData.status = 'pending';
    const result = await dbService.addTodoItem(itemData);
    if (result) {
      await queryTodos();
      todoNotifier.register(result);
      message.success('创建成功');
    }
  }

  modalVisible.value = false;
  resetForm();
};

// ==================== 工具函数 ====================
const statusColor = (status) => {
  const map = { pending: 'default', ongoing: 'processing', done: 'success' };
  return map[status] || 'default';
};

const statusText = (status) => {
  const map = { pending: '未开始', ongoing: '进行中', done: '已完成' };
  return map[status] || status;
};

const formatTime = (dateStr) => {
  if (!dateStr) return '';
  return dayjs(dateStr).format('HH:mm');
};
</script>

<style scoped>
.todo-page {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.todo-date-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px 8px;
  gap: 8px;
  border-bottom: 1px solid #f0f0f0;
  height: 36px;
}

.todo-date-text {
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  min-width: 160px;
  text-align: center;
}

.todo-date-text:hover {
  color: #1890ff;
}

.todo-toolbar {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  border-bottom: 1px solid #f0f0f0;
  height: 36px;
}

.todo-list {
  flex: 1;
  overflow: hidden;
}

.scroller {
  height: 100%;
  padding: 4px;
}

.todo-item {
  padding: 8px;
  margin: 2px 0;
  border-radius: 6px;
  border: 1px solid #f0f0f0;
  transition: all 0.2s;
}

.todo-item:hover {
  background-color: #fafafa;
  border-color: #d9d9d9;
}

.todo-item-done {
  opacity: 0.6;
}

.todo-item-done .todo-title {
  text-decoration: line-through;
}

.todo-item-header {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 4px;
}

.todo-conflict {
  font-size: 14px;
}

.todo-title {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.todo-item-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.todo-time {
  font-size: 12px;
  color: #666;
}

.todo-item-actions {
  display: flex;
  align-items: center;
  gap: 2px;
}

.todo-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/TodoPage.vue
git commit -m "feat: create TodoPage.vue component with date switcher and todo management"
```

---

### Task 5: Add "待办" tab to CutPage.vue

**Files:**
- Modify: `src/components/CutPage.vue`

- [ ] **Step 1: Add import**

In `src/components/CutPage.vue`, after the `import FavoritePage from './FavoritePage.vue'` line, add:

```javascript
import TodoPage from './TodoPage.vue'
```

- [ ] **Step 2: Add tab pane**

In the template, after the "收藏" tab pane and before the "白板" tab pane, add:

```html
      <!-- 待办列表标签页 -->
      <a-tab-pane key="todoList" tab="待办">
        <todo-page></todo-page>
      </a-tab-pane>
```

The final tab order: 文本 → 图片 → 收藏 → 待办 → 白板

- [ ] **Step 3: Commit**

```bash
git add src/components/CutPage.vue
git commit -m "feat: add todo tab to CutPage"
```

---

### Task 6: Verify build

**Files:**
- No new files

- [ ] **Step 1: Install npm dependency**

Run: `cd "D:/project/view/cut" && npm install`

- [ ] **Step 2: Build frontend**

Run: `cd "D:/project/view/cut" && npm run build`

Expected: Build succeeds with no errors.

- [ ] **Step 3: Commit any auto-generated changes**

```bash
git add -A
git commit -m "chore: update dependencies and auto-generated files for todo feature"
```
