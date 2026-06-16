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
