import { reactive } from 'vue';
import { sendNotification, isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';

const DEFAULTS = { focusMinutes: 25, restMinutes: 5, mode: 'loop' };

// Reactive state
const state = reactive({
  phase: 'idle',        // 'idle' | 'focusing' | 'resting' | 'paused'
  remainingSeconds: 0,
  currentRound: 0,      // completed focus rounds
  totalFocusSeconds: 0, // cumulative focus seconds this session
  config: { ...DEFAULTS },
});

let intervalId = null;

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
    const result = sendNotification({ title, body });
    if (result && typeof result.catch === 'function') {
      result.catch(error => console.error('发送通知失败:', error));
    }
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

function beginFocusRound() {
  state.phase = 'focusing';
  state.remainingSeconds = state.config.focusMinutes * 60;
  startInterval();
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
    beginFocusRound();
  }
}

function resetState() {
  stopInterval();
  state.phase = 'idle';
  state.remainingSeconds = 0;
  state.currentRound = 0;
  state.totalFocusSeconds = 0;
}

// ==================== Public API ====================

/**
 * 开始专注计时
 * @param {Object} config - 配置
 * @param {number} config.focusMinutes - 专注时长（分钟）
 * @param {number} config.restMinutes - 休息时长（分钟）
 * @param {string} config.mode - 模式: 'once' | 'loop'
 */
async function start(config) {
  await ensurePermission();

  state.config.focusMinutes = config.focusMinutes || DEFAULTS.focusMinutes;
  state.config.restMinutes = config.restMinutes || DEFAULTS.restMinutes;
  state.config.mode = config.mode || DEFAULTS.mode;

  state.phase = 'focusing';
  state.remainingSeconds = state.config.focusMinutes * 60;
  state.currentRound = 0;
  state.totalFocusSeconds = 0;

  notify('🍅 专注开始', '开始专注，加油！');
  startInterval();
}

/**
 * 暂停专注（仅 focusing 状态可用）
 */
function pause() {
  if (state.phase !== 'focusing') return;
  stopInterval();
  state.phase = 'paused';
}

/**
 * 继续专注
 */
function resume() {
  if (state.phase !== 'paused') return;
  state.phase = 'focusing';
  startInterval();
}

/**
 * 停止专注，重置所有状态
 */
function stop() {
  resetState();
}

/**
 * 跳过休息，直接进入下一轮专注
 */
function skipRest() {
  if (state.phase !== 'resting') return;
  stopInterval();
  notify('🍅 休息结束', '准备开始新一轮专注！');
  beginFocusRound();
}

export default {
  state,
  start,
  pause,
  resume,
  stop,
  skipRest,
};
