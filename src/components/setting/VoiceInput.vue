<template>
  <div class="voice-input-settings">
    <h3>语音输入</h3>
    <p class="hint">按住 Alt+Space 说话，松开即将识别结果输入到焦点窗口</p>

    <!-- 快捷键显示 -->
    <div class="setting-row">
      <label>语音输入快捷键</label>
      <span class="hotkey-display">Alt + Space</span>
    </div>

    <!-- 模型管理 -->
    <div class="model-section">
      <h4>模型管理</h4>
      <a-table
        :columns="columns"
        :data-source="models"
        :pagination="false"
        size="small"
        row-key="lang"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'status'">
            <a-tag v-if="record.status === 'ready'" color="green">已就绪</a-tag>
            <a-tag v-else-if="record.status === 'downloading'" color="blue">
              下载中 {{ record.downloadPercent }}%
            </a-tag>
            <a-tag v-else color="default">未下载</a-tag>
          </template>
          <template v-if="column.key === 'action'">
            <a-button
              v-if="record.status === 'not_downloaded'"
              type="primary"
              size="small"
              :loading="record.downloading"
              @click="handleDownload(record.lang)"
            >
              下载
            </a-button>
            <a-popconfirm
              v-else-if="record.status === 'ready'"
              title="确定删除此模型？"
              @confirm="handleDelete(record.lang)"
            >
              <a-button size="small" danger>删除</a-button>
            </a-popconfirm>
          </template>
        </template>
      </a-table>
    </div>

    <!-- 当前状态 -->
    <div class="status-section" v-if="speechStatus.model_loaded">
      <a-tag color="blue">当前语言: {{ speechStatus.current_lang === 'zh-en' ? '中英双语' : speechStatus.current_lang }}</a-tag>
      <a-tag :color="speechStatus.is_recording ? 'red' : 'green'">
        {{ speechStatus.is_recording ? '录音中' : '就绪' }}
      </a-tag>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const models = ref([]);
const speechStatus = ref({ model_loaded: false, is_recording: false, current_lang: '' });

const columns = [
  { title: '模型', dataIndex: 'display_name', key: 'display_name' },
  { title: '大小', dataIndex: 'size_mb', key: 'size_mb', customRender: ({ text }) => text > 0 ? `${text.toFixed(1)} MB` : '-' },
  { title: '状态', key: 'status' },
  { title: '操作', key: 'action' },
];

let unlistenProgress = null;
let unlistenRecording = null;

async function loadModels() {
  try {
    const list = await invoke('list_speech_models');
    models.value = list.map(m => ({ ...m, downloading: false, downloadPercent: 0 }));
  } catch (e) {
    console.error('加载模型列表失败:', e);
  }
}

async function loadSpeechStatus() {
  try {
    speechStatus.value = await invoke('get_speech_status');
  } catch (e) {
    console.error('获取语音状态失败:', e);
  }
}

async function handleDownload(lang) {
  const model = models.value.find(m => m.lang === lang);
  if (model) {
    model.downloading = true;
    model.status = 'downloading';
  }

  try {
    await invoke('download_speech_model', { lang });
    await loadModels();
  } catch (e) {
    console.error('下载模型失败:', e);
    alert(`下载失败: ${e}`);
    await loadModels();
  }
}

async function handleDelete(lang) {
  try {
    await invoke('delete_speech_model', { lang });
    await loadModels();
    await loadSpeechStatus();
  } catch (e) {
    console.error('删除模型失败:', e);
    alert(`删除失败: ${e}`);
  }
}

onMounted(async () => {
  await loadModels();
  await loadSpeechStatus();

  // 监听下载进度事件
  unlistenProgress = await listen('model-download-progress', (event) => {
    const { lang, progress } = event.payload;
    const model = models.value.find(m => m.lang === lang);
    if (model) {
      model.downloadPercent = Math.round(progress * 100);
    }
  });

  // 监听录音状态
  unlistenRecording = await listen('recording-state', (event) => {
    speechStatus.value.is_recording = event.payload.is_recording;
  });
});

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
  }
  if (unlistenRecording) {
    unlistenRecording();
  }
});
</script>

<style scoped>
.voice-input-settings {
  padding: 16px;
}

.hint {
  color: #888;
  font-size: 13px;
  margin-bottom: 16px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.setting-row label {
  font-weight: 500;
}

.hotkey-display {
  background: #f5f5f5;
  padding: 4px 12px;
  border-radius: 4px;
  font-family: monospace;
}

.model-section {
  margin-top: 20px;
}

.model-section h4 {
  margin-bottom: 12px;
}

.status-section {
  margin-top: 16px;
  display: flex;
  gap: 8px;
}
</style>
