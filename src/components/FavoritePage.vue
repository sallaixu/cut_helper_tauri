<template>
  <div class="favorite-page">
    <!-- 顶部操作栏 -->
    <div class="favorite-toolbar">
      <a-select
        v-model:value="currentGroupId"
        style="flex: 1; min-width: 0;"
        placeholder="选择分组"
        :options="groupOptions"
        @change="onGroupChange"
      />
      <a-button size="small" @click="showCreateGroup" style="margin-left: 4px;">
        <template #icon><PlusOutlined /></template>
      </a-button>
      <a-button size="small" @click="showManageModal" style="margin-left: 4px;">
        <template #icon><SettingOutlined /></template>
      </a-button>
    </div>

    <!-- 收藏内容列表 -->
    <div class="favorite-list" ref="scrollerRef" style="height: calc(100% - 40px);">
      <a-back-top :target="() => getTarget()" />

      <template v-if="currentGroupId && favoriteList.length > 0">
        <virt-list
          class="scroller"
          :list="favoriteList"
          itemKey="id"
          :minSize="40"
          id="favoriteItemBox"
          ref="virtListRef"
        >
          <template #default="{ itemData, index }">
            <div
              class="favorite-item"
              @dblclick="handleCopy(itemData)"
            >
              <!-- 内容 -->
              <div class="favorite-item-content">
                <a-popover trigger="hover" :mouseEnterDelay="1" placement="topLeft">
                  <template #title>{{ formatDate(itemData.createTime) }}</template>
                  <template #content>
                    <div class="detail-style" style="max-height: 80vh; max-width: 90vw;">
                      <pre>{{ itemData.content }}</pre>
                    </div>
                  </template>
                  <div style="white-space: nowrap;">
                    <li>{{ (index + 1) }} . {{ itemData.title || itemData.content }}</li>
                  </div>
                </a-popover>
              </div>

              <!-- 时间 + 操作 -->
              <div class="favorite-item-actions">
                <span>{{ format(itemData.createTime, 'short') }}</span>
                <a-dropdown :trigger="['click']">
                  <more-outlined class="jump" @click.prevent style="cursor: pointer; color: black;" />
                  <template #overlay>
                    <a-menu>
                      <a-menu-item @click="handleCopy(itemData)" key="copy">
                        <div><CopyOutlined /><span style="margin-left: 8px;">复制</span></div>
                      </a-menu-item>
                      <a-menu-item @click="handleDelete(itemData)" key="delete" style="color: #f5222d;">
                        <div><DeleteOutlined /><span style="margin-left: 8px;">删除</span></div>
                      </a-menu-item>
                    </a-menu>
                  </template>
                </a-dropdown>
              </div>
            </div>
          </template>
        </virt-list>
      </template>

      <!-- 空状态 -->
      <div v-else class="favorite-empty">
        <a-empty :description="currentGroupId ? '该分组暂无收藏' : '请选择一个分组'" />
      </div>
    </div>

    <!-- 新建分组弹窗 -->
    <a-modal
      v-model:open="createGroupVisible"
      title="新建分组"
      ok-text="创建"
      cancel-text="取消"
      @ok="handleCreateGroup"
    >
      <a-input v-model:value="newGroupName" placeholder="分组名称" />
    </a-modal>

    <!-- 分组管理弹窗 -->
    <a-modal
      v-model:open="manageGroupVisible"
      title="分组管理"
      :footer="null"
      width="400px"
    >
      <div class="manage-group-list">
        <div v-for="group in groupList" :key="group.id" class="manage-group-row">
          <template v-if="editingGroupId === group.id">
            <a-input
              v-model:value="editingGroupName"
              size="small"
              style="flex: 1;"
              @pressEnter="handleRenameGroup(group.id)"
            />
            <a-button size="small" type="link" @click="handleRenameGroup(group.id)">保存</a-button>
            <a-button size="small" type="link" @click="cancelEditGroup">取消</a-button>
          </template>
          <template v-else>
            <span style="flex: 1; line-height: 32px;">{{ group.name }}</span>
            <a-button size="small" type="link" @click="startEditGroup(group)">
              <template #icon><EditOutlined /></template>
            </a-button>
            <a-popconfirm title="删除分组将同时删除该分组下所有收藏，确认删除？" @confirm="handleDeleteGroup(group.id)">
              <a-button size="small" type="link" danger>
                <template #icon><DeleteOutlined /></template>
              </a-button>
            </a-popconfirm>
          </template>
        </div>

        <div v-if="groupList.length === 0" style="text-align: center; color: #999; padding: 20px;">
          暂无分组，请先新建分组
        </div>
      </div>

      <div style="text-align: right; margin-top: 16px;">
        <a-button @click="manageGroupVisible = false">关闭</a-button>
      </div>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { VirtList } from 'vue-virt-list';
import {
  PlusOutlined, SettingOutlined, MoreOutlined,
  DeleteOutlined, EditOutlined, CopyOutlined
} from '@ant-design/icons-vue';
import { message } from 'ant-design-vue';
import { format, register } from 'timeago.js';
import dbService from '../db_service';
import { copyToSystem } from '../cut_service';

message.config({
  top: '50px',
  duration: 1,
  maxCount: 2,
});

// ==================== 时间格式化配置 ====================
const localeFunc = (number, index, totalSec) => {
  return [
    ['刚刚', 'right now'],
    ['%s秒前', 'in %s seconds'],
    ['1分前', 'in 1 minute'],
    ['%s分前', 'in %s minutes'],
    ['1小时前', 'in 1 hour'],
    ['%s小时前', 'in %s hours'],
    ['昨天', 'in 1 day'],
    ['%s天前', 'in %s days'],
    ['1周前', 'in 1 week'],
    ['%s周前', 'in %s weeks'],
    ['1月前', 'in 1 month'],
    ['%s月前', 'in %s months'],
    ['1年前', 'in 1 year'],
    ['%s年前', 'in %s years']
  ][index];
};
register('short', localeFunc);

// ==================== 响应式数据 ====================
const groupList = ref([]);
const currentGroupId = ref(undefined);
const favoriteList = ref([]);
const createGroupVisible = ref(false);
const newGroupName = ref('');
const manageGroupVisible = ref(false);
const editingGroupId = ref(null);
const editingGroupName = ref('');
const virtListRef = ref(null);

// 分组下拉选项（computed）
const groupOptions = computed(() =>
  groupList.value.map(g => ({ value: g.id, label: g.name }))
);

// ==================== 生命周期 ====================
onMounted(async () => {
  await queryGroups();
  // 默认选中第一个分组
  if (groupList.value.length > 0) {
    currentGroupId.value = groupList.value[0].id;
    await queryGroupItems();
  }
});

// 监听分组切换，重新加载收藏列表
watch(currentGroupId, () => {
  queryGroupItems();
});

// ==================== 数据操作 ====================
const queryGroups = async () => {
  const result = await dbService.fetchGroups();
  groupList.value = result || [];
};

const queryGroupItems = async () => {
  if (!currentGroupId.value) {
    favoriteList.value = [];
    return;
  }
  const result = await dbService.fetchGroupItems(currentGroupId.value);
  favoriteList.value = result || [];
};

const onGroupChange = (value) => {
  currentGroupId.value = value;
};

// ==================== 新建分组 ====================
const showCreateGroup = () => {
  newGroupName.value = '';
  createGroupVisible.value = true;
};

const handleCreateGroup = async () => {
  if (!newGroupName.value.trim()) {
    message.warning('分组名称不能为空');
    return;
  }

  const group = await dbService.addGroup(newGroupName.value.trim());
  if (group) {
    await queryGroups();
    currentGroupId.value = group.id;
    message.success('分组创建成功');
  }
  createGroupVisible.value = false;
  newGroupName.value = '';
};

// ==================== 分组管理 ====================
const showManageModal = async () => {
  await queryGroups();
  manageGroupVisible.value = true;
  editingGroupId.value = null;
};

const startEditGroup = (group) => {
  editingGroupId.value = group.id;
  editingGroupName.value = group.name;
};

const cancelEditGroup = () => {
  editingGroupId.value = null;
  editingGroupName.value = '';
};

const handleRenameGroup = async (groupId) => {
  if (!editingGroupName.value.trim()) {
    message.warning('分组名称不能为空');
    return;
  }

  const success = await dbService.renameGroup(groupId, editingGroupName.value.trim());
  if (success) {
    await queryGroups();
    message.success('重命名成功');
  }
  editingGroupId.value = null;
  editingGroupName.value = '';
};

const handleDeleteGroup = async (groupId) => {
  const success = await dbService.removeGroup(groupId);
  if (success) {
    await queryGroups();
    // 如果删除的是当前选中的分组，切换到第一个或清空
    if (currentGroupId.value === groupId) {
      currentGroupId.value = groupList.value.length > 0 ? groupList.value[0].id : undefined;
    }
    await queryGroupItems();
    message.success('删除成功');
  }
};

// ==================== 收藏项操作 ====================
const handleCopy = (item) => {
  copyToSystem(item.content);
  message.success('已复制到剪贴板');
};

const handleDelete = async (item) => {
  const success = await dbService.removeGroupItem(item.id);
  if (success) {
    const index = favoriteList.value.findIndex(f => f.id === item.id);
    if (index !== -1) {
      favoriteList.value.splice(index, 1);
    }
    message.success('删除成功');
  }
};

// ==================== 工具函数 ====================
const formatDate = (dateStr) => {
  const date = new Date(dateStr);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  const seconds = String(date.getSeconds()).padStart(2, '0');
  return `${year}年${month}月${day}日，${hours}:${minutes}:${seconds}`;
};

const getTarget = () => document.getElementById('favoriteItemBox');
</script>

<style scoped>
.favorite-page {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.favorite-toolbar {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  border-bottom: 1px solid #f0f0f0;
  height: 40px;
}

.favorite-list {
  overflow: hidden;
}

.scroller {
  height: 100%;
  padding: 4px;
}

.favorite-item {
  display: flex;
  height: 3em;
  align-items: center;
  padding: 5px;
  cursor: pointer;
}

.favorite-item:hover {
  background-color: rgb(171, 225, 153);
}

.favorite-item-content {
  flex: 1;
  overflow: hidden;
  margin-right: 6px;
  line-height: 1.5em;
  height: 1.5em;
}

.favorite-item-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  width: fit-content;
}

.favorite-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.manage-group-list {
  max-height: 300px;
  overflow-y: auto;
}

.manage-group-row {
  display: flex;
  align-items: center;
  padding: 4px 0;
  border-bottom: 1px solid #f0f0f0;
}

.detail-style {
  overflow-y: scroll;
  overflow-x: scroll;
}

.jump {
  cursor: pointer;
}
</style>
