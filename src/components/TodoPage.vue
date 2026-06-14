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
    <div class="todo-list">
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
              <div class="todo-item-header">
                <span v-if="itemData._conflict" class="todo-conflict" title="时间冲突">⚠️</span>
                <span class="todo-title">{{ itemData.title }}</span>
                <a-tag :color="statusColor(itemData.status)" size="small" style="margin-left: auto;">
                  {{ statusText(itemData.status) }}
                </a-tag>
              </div>
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

const filterTodos = () => {};

// ==================== 数据操作 ====================
const queryTodos = async () => {
  const result = await dbService.fetchTodoItemsByDate(currentDate.value);
  todoList.value = (result || []).map(t => ({ ...t, _conflict: false }));
  checkConflicts();
};

const checkConflicts = async () => {
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
    const success = await dbService.updateTodoItem(editingId.value, {
      ...itemData,
      startTime: st,
      endTime: et,
    });
    if (success) {
      await queryTodos();
      const updated = todoList.value.find(t => t.id === editingId.value);
      if (updated) todoNotifier.register(updated);
      message.success('保存成功');
    }
  } else {
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
