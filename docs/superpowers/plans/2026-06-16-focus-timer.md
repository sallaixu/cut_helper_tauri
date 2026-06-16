# Focus Timer (专注时间) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Pomodoro-style focus timer tab to the CutPage, with single/loop modes, preset+custom durations, and system notifications.

**Architecture:** Pure frontend timer service (`focus_timer.js`) using Vue `reactive()` + `setInterval`, no database. FocusPage.vue renders setup/countdown views. CutPage.vue adds the tab with dynamic title showing countdown.

**Tech Stack:** Vue 3 Composition API, Ant Design Vue, @tauri-apps/plugin-notification

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src/focus_timer.js` | Create | Timer state machine, countdown logic, system notifications |
| `src/components/FocusPage.vue` | Create | Focus tab UI: setup view, countdown view, rest view |
| `src/components/CutPage.vue` | Modify | Add "专注" tab pane, dynamic tab title with countdown |

---

### Task 1: Create focus_timer.js — Timer Service

**Files:**
- Create: `src/focus_timer.js`

- [ ] **Step 1: Create focus_timer.js with reactive state, start/stop/pause/resume/skipRest API, and notification logic**

```js
import { reactive } from 'vue';
import { sendNotification, isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';

// Reactive state
const state = reactive({
  phase: 'idle',        // 'idle' | 'focusing' | 'resting' | 'paused'
  remainingSeconds: 0,
  currentRound: 0,      // completed focus rounds
  totalFocusSeconds: 0, // cumulative focus seconds this session
  config: {
    focusMinutes: 25,
    restMinutes: 5,
    mode: 'loop',        // 'once' | 'loop'
  },
});

let intervalId = null;
let pausedPhase = null;  // phase before pause ('focusing')

// ==================== Notification helpers ====================

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

function notify(title, body) {
  try {
    sendNotification({ title, body });
  } catch (error) {
    console.error('发送通知失败:', error);
  }
}

// ==================== Countdown engine ====================

function startInterval() {
  stopInterval();
  intervalId = setInterval(() => {
    if (state.remainingSeconds > 0) {
      state.remainingSeconds--;

      // Track focus seconds
      if (state.phase === 'focusing') {
        state.totalFocusSeconds++;
      }
    } else {
      onPhaseEnd();
    }
  }, 1000);
}

function stopInterval() {
  if (intervalId !== null) {
    clearInterval(intervalId);
    intervalId = null;
  }
}

function onPhaseEnd() {
  stopInterval();

  if (state.phase === 'focusing') {
    // Focus round completed
    state.currentRound++;

    if (state.config.mode === 'once') {
      // Single mode: done
      const focusMin = Math.round(state.totalFocusSeconds / 60);
      notify('✅ 专注完成', `已专注 ${focusMin} 分钟！`);
      resetState();
      return;
    }

    // Loop mode: enter rest
    notify('✅ 专注完成', `已专注 ${Math.round(state.totalFocusSeconds / 60)} 分钟，休息一下！`);
    state.phase = 'resting';
    state.remainingSeconds = state.config.restMinutes * 60;
    startInterval();
  } else if (state.phase === 'resting') {
    // Rest completed: start next focus round
    notify('🍅 休息结束', '准备开始新一轮专注！');
    state.phase = 'focusing';
    state.remainingSeconds = state.config.focusMinutes * 60;
    startInterval();
  }
}

function resetState() {
  stopInterval();
  state.phase = 'idle';
  state.remainingSeconds = 0;
  state.currentRound = 0;
  state.totalFocusSeconds = 0;
  pausedPhase = null;
}

// ==================== Public API ====================

async function start(config) {
  await ensurePermission();

  state.config.focusMinutes = config.focusMinutes || 25;
  state.config.restMinutes = config.restMinutes || 5;
  state.config.mode = config.mode || 'loop';

  state.phase = 'focusing';
  state.remainingSeconds = state.config.focusMinutes * 60;
  state.currentRound = 0;
  state.totalFocusSeconds = 0;
  pausedPhase = null;

  notify('🍅 专注开始', '开始专注，加油！');
  startInterval();
}

function pause() {
  if (state.phase !== 'focusing') return;
  stopInterval();
  pausedPhase = 'focusing';
  state.phase = 'paused';
}

function resume() {
  if (state.phase !== 'paused') return;
  state.phase = pausedPhase || 'focusing';
  pausedPhase = null;
  startInterval();
}

function stop() {
  resetState();
}

function skipRest() {
  if (state.phase !== 'resting') return;
  stopInterval();
  notify('🍅 休息结束', '准备开始新一轮专注！');
  state.phase = 'focusing';
  state.remainingSeconds = state.config.focusMinutes * 60;
  startInterval();
}

export default {
  state,
  start,
  pause,
  resume,
  stop,
  skipRest,
};
```

- [ ] **Step 2: Commit**

```bash
git add src/focus_timer.js
git commit -m "feat: add focus_timer.js timer service with state machine and notifications"
```

---

### Task 2: Create FocusPage.vue — Focus Tab UI

**Files:**
- Create: `src/components/FocusPage.vue`

- [ ] **Step 1: Create FocusPage.vue with idle/focusing/resting/paused views**

```vue
<template>
  <div class="focus-page">
    <!-- 空闲状态：设置面板 -->
    <div v-if="timer.state.phase === 'idle'" class="focus-setup">
      <div class="focus-title">专注时间</div>

      <!-- 专注时长 -->
      <div class="focus-section">
        <div class="focus-label">专注时长</div>
        <div class="focus-presets">
          <a-button
            v-for="m in focusPresets"
            :key="m"
            :type="focusMinutes === m ? 'primary' : 'default'"
            size="small"
            @click="selectFocusPreset(m)"
          >{{ m }} 分钟</a-button>
        </div>
        <div class="focus-custom">
          <span>自定义:</span>
          <a-input-number v-model:value="focusMinutes" :min="1" :max="180" size="small" style="width: 70px; margin: 0 4px;" />
          <span>分钟</span>
        </div>
      </div>

      <!-- 休息时长（仅循环模式显示） -->
      <div v-if="focusMode === 'loop'" class="focus-section">
        <div class="focus-label">休息时长</div>
        <div class="focus-presets">
          <a-button
            v-for="m in restPresets"
            :key="m"
            :type="restMinutes === m ? 'primary' : 'default'"
            size="small"
            @click="selectRestPreset(m)"
          >{{ m }} 分钟</a-button>
        </div>
        <div class="focus-custom">
          <span>自定义:</span>
          <a-input-number v-model:value="restMinutes" :min="1" :max="60" size="small" style="width: 70px; margin: 0 4px;" />
          <span>分钟</span>
        </div>
      </div>

      <!-- 专注模式 -->
      <div class="focus-section">
        <div class="focus-label">专注模式</div>
        <a-radio-group v-model:value="focusMode" size="small">
          <a-radio value="once">单次专注</a-radio>
          <a-radio value="loop">循环专注</a-radio>
        </a-radio-group>
      </div>

      <!-- 开始按钮 -->
      <a-button type="primary" size="large" class="focus-start-btn" @click="handleStart">
        <template #icon><ClockCircleOutlined /></template>
        开始专注
      </a-button>
    </div>

    <!-- 专注中/暂停状态 -->
    <div v-else-if="timer.state.phase === 'focusing' || timer.state.phase === 'paused'" class="focus-countdown" :class="{ 'focus-paused': timer.state.phase === 'paused' }">
      <div class="focus-time">{{ formatCountdown(timer.state.remainingSeconds) }}</div>
      <div class="focus-status">{{ timer.state.phase === 'paused' ? '已暂停' : '专注中...' }}</div>

      <!-- 循环指示器 -->
      <div v-if="timer.state.config.mode === 'loop'" class="focus-rounds">
        <span
          v-for="i in Math.max(timer.state.currentRound + 1, 1)"
          :key="i"
          class="focus-round-dot"
          :class="{ 'focus-round-done': i <= timer.state.currentRound }"
        >●</span>
      </div>

      <div class="focus-actions">
        <a-button v-if="timer.state.phase === 'focusing'" @click="timer.pause()">
          <template #icon><PauseCircleOutlined /></template>
          暂停
        </a-button>
        <a-button v-else type="primary" @click="timer.resume()">
          <template #icon><PlayCircleOutlined /></template>
          继续
        </a-button>
        <a-button danger @click="handleStop">
          <template #icon><StopOutlined /></template>
          停止
        </a-button>
      </div>
    </div>

    <!-- 休息中状态 -->
    <div v-else-if="timer.state.phase === 'resting'" class="focus-countdown focus-resting">
      <div class="focus-time">{{ formatCountdown(timer.state.remainingSeconds) }}</div>
      <div class="focus-status">休息中...</div>

      <div class="focus-actions">
        <a-button @click="timer.skipRest()">
          <template #icon><ForwardOutlined /></template>
          跳过休息
        </a-button>
        <a-button danger @click="handleStop">
          <template #icon><StopOutlined /></template>
          结束专注
        </a-button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import {
  ClockCircleOutlined, PauseCircleOutlined,
  PlayCircleOutlined, StopOutlined, ForwardOutlined
} from '@ant-design/icons-vue';
import { Modal } from 'ant-design-vue';
import focusTimer from '../focus_timer';

const timer = focusTimer;

// ==================== 设置数据 ====================
const focusPresets = [15, 25, 30, 45, 60];
const restPresets = [5, 10, 15];
const focusMinutes = ref(25);
const restMinutes = ref(5);
const focusMode = ref('loop');

const selectFocusPreset = (m) => { focusMinutes.value = m; };
const selectRestPreset = (m) => { restMinutes.value = m; };

// ==================== 操作 ====================
const handleStart = () => {
  timer.start({
    focusMinutes: focusMinutes.value,
    restMinutes: restMinutes.value,
    mode: focusMode.value,
  });
};

const handleStop = () => {
  const totalMin = Math.round(timer.state.totalFocusSeconds / 60);
  const roundInfo = timer.state.config.mode === 'loop' && timer.state.currentRound > 0
    ? `\n已完成 ${timer.state.currentRound} 轮，累计专注 ${totalMin} 分钟`
    : '';

  Modal.confirm({
    title: '确认停止',
    content: `确定要停止当前专注吗？${roundInfo}`,
    okText: '停止',
    cancelText: '取消',
    onOk: () => {
      timer.stop();
    },
  });
};

// ==================== 工具函数 ====================
const formatCountdown = (seconds) => {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
};
</script>

<style scoped>
.focus-page {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* 设置面板 */
.focus-setup {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px;
  gap: 16px;
}

.focus-title {
  font-size: 18px;
  font-weight: 600;
  color: #333;
}

.focus-section {
  width: 100%;
  max-width: 280px;
}

.focus-label {
  font-size: 13px;
  color: #666;
  margin-bottom: 6px;
}

.focus-presets {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 6px;
}

.focus-custom {
  display: flex;
  align-items: center;
  font-size: 12px;
  color: #666;
}

.focus-start-btn {
  margin-top: 8px;
  min-width: 140px;
}

/* 倒计时面板 */
.focus-countdown {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
}

.focus-time {
  font-size: 56px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  color: #333;
  line-height: 1;
}

.focus-status {
  font-size: 14px;
  color: #666;
}

.focus-paused .focus-time {
  color: #999;
}

.focus-resting .focus-time {
  color: #52c41a;
}

.focus-resting .focus-status {
  color: #52c41a;
}

.focus-rounds {
  display: flex;
  gap: 6px;
  margin: 8px 0;
}

.focus-round-dot {
  font-size: 14px;
  color: #d9d9d9;
}

.focus-round-done {
  color: #1890ff;
}

.focus-actions {
  display: flex;
  gap: 12px;
  margin-top: 16px;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/FocusPage.vue
git commit -m "feat: add FocusPage.vue with setup/countdown/rest views"
```

---

### Task 3: Integrate Focus Tab into CutPage.vue

**Files:**
- Modify: `src/components/CutPage.vue`

- [ ] **Step 1: Add FocusPage import and tab pane with dynamic title**

In `CutPage.vue`, make these changes:

**1a. Add import** — after the existing `import TodoPage` line (line 57), add:

```js
import FocusPage from './FocusPage.vue'
import focusTimer from '../focus_timer'
```

**1b. Add computed property for dynamic tab title** — after the `appConfig` ref (around line 69), add:

```js
const focusTabTitle = computed(() => {
  if (focusTimer.state.phase === 'idle') return '专注';
  const m = Math.floor(focusTimer.state.remainingSeconds / 60);
  const s = focusTimer.state.remainingSeconds % 60;
  const time = `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  return `专注 ${time}`;
});
```

Also add `computed` to the import from `vue` on line 54:

```js
import { ref, computed, onMounted, onUnmounted } from 'vue'
```

**1c. Add tab pane** — after the "待办" tab pane (after line 22), add:

```html
      <!-- 专注时间标签页 -->
      <a-tab-pane key="focusList" :tab="focusTabTitle">
        <focus-page></focus-page>
      </a-tab-pane>
```

- [ ] **Step 2: Commit**

```bash
git add src/components/CutPage.vue
git commit -m "feat: add focus tab to CutPage with dynamic countdown title"
```

---

### Task 4: Manual Verification

- [ ] **Step 1: Run the app**

```bash
cd D:/project/view/cut && npm run tauri dev
```

- [ ] **Step 2: Verify the following behaviors**

1. "专注" tab appears after "待办" in the tab bar
2. Setup view shows focus presets (15/25/30/45/60), rest presets (5/10/15), mode radio, and start button
3. Clicking "开始专注" starts countdown, tab title shows "专注 24:59" etc.
4. Pause/continue works during focusing
5. Stop shows confirmation modal
6. When focus ends, system notification appears
7. In loop mode, rest countdown starts after focus ends
8. "跳过休息" skips to next focus round
9. In single mode, no rest duration shown; timer stops after one focus round
10. Switching to other tabs while timer runs — countdown continues in background, tab title still updates
