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
