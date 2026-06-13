# Favorites Feature Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a "收藏" tab with group-based text favorites to the clipboard assistant, allowing users to save text history items into named groups that persist permanently.

**Architecture:** All database operations go through `src/db_service.js` using `@tauri-apps/plugin-sql` (matching existing patterns). The new `FavoritePage.vue` component uses a dropdown group selector + single-column list layout to fit the narrow clipboard window. TimeList.vue's existing group menu modal is wired to real db_service methods.

**Tech Stack:** Vue 3 (Composition API), Ant Design Vue, vue-virt-list, @tauri-apps/plugin-sql, SQLite

---

### Task 1: Add group/favorite CRUD methods to db_service.js

**Files:**
- Modify: `src/db_service.js`

- [ ] **Step 1: Add the 7 new methods to db_service.js**

Append the following methods inside the `export default { ... }` object, after the existing `removeImageItem` method:

```javascript
  // 分组相关方法
  async addGroup(name) {
    await this.init();
    const id = uuidv4();
    const createTime = new Date().toISOString();
    
    try {
      await db.execute(
        'INSERT INTO Groups (id, name, createTime) VALUES (?, ?, ?)',
        [id, name, createTime]
      );
      return { id, name, createTime };
    } catch (error) {
      console.error('Error adding group:', error);
      return null;
    }
  },

  async fetchGroups() {
    await this.init();
    try {
      const result = await db.select('SELECT * FROM Groups ORDER BY createTime ASC');
      return result || [];
    } catch (error) {
      console.error('Error fetching groups:', error);
      return [];
    }
  },

  async renameGroup(id, name) {
    await this.init();
    try {
      await db.execute(
        'UPDATE Groups SET name = ? WHERE id = ?',
        [name, id]
      );
      return true;
    } catch (error) {
      console.error('Error renaming group:', error);
      return false;
    }
  },

  async removeGroup(id) {
    await this.init();
    try {
      // 级联删除：先删除该分组下所有收藏项
      await db.execute(
        'DELETE FROM GroupItems WHERE groupId = ?',
        [id]
      );
      // 再删除分组本身
      await db.execute(
        'DELETE FROM Groups WHERE id = ?',
        [id]
      );
      return true;
    } catch (error) {
      console.error('Error removing group:', error);
      return false;
    }
  },

  // 收藏项相关方法
  async addGroupItem(groupId, content, title) {
    await this.init();
    const id = uuidv4();
    const createTime = new Date().toISOString();
    
    try {
      await db.execute(
        'INSERT INTO GroupItems (id, groupId, content, title, createTime) VALUES (?, ?, ?, ?, ?)',
        [id, groupId, content, title, createTime]
      );
      return { id, groupId, content, title, createTime };
    } catch (error) {
      console.error('Error adding group item:', error);
      return null;
    }
  },

  async fetchGroupItems(groupId) {
    await this.init();
    try {
      const result = await db.select(
        'SELECT * FROM GroupItems WHERE groupId = ? ORDER BY createTime DESC',
        [groupId]
      );
      return result || [];
    } catch (error) {
      console.error('Error fetching group items:', error);
      return [];
    }
  },

  async removeGroupItem(id) {
    await this.init();
    try {
      await db.execute('DELETE FROM GroupItems WHERE id = ?', [id]);
      return true;
    } catch (error) {
      console.error('Error removing group item:', error);
      return false;
    }
  },
```

- [ ] **Step 2: Verify db_service.js syntax**

Run: `node -e "import('./src/db_service.js').then(m => console.log('OK, methods:', Object.keys(m.default))).catch(e => console.error(e))"` (may fail due to Tauri deps, so instead just visually verify the file is well-formed — no syntax errors, all methods inside the export object)

- [ ] **Step 3: Commit**

```bash
git add src/db_service.js
git commit -m "feat: add group and favorite CRUD methods to db_service"
```

---

### Task 2: Create FavoritePage.vue component

**Files:**
- Create: `src/components/FavoritePage.vue`

- [ ] **Step 1: Create FavoritePage.vue with full implementation**

Create `src/components/FavoritePage.vue` with the following content:

```vue
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
```

- [ ] **Step 2: Commit**

```bash
git add src/components/FavoritePage.vue
git commit -m "feat: create FavoritePage.vue component with group selector and favorite list"
```

---

### Task 3: Add "收藏" tab to CutPage.vue

**Files:**
- Modify: `src/components/CutPage.vue`

- [ ] **Step 1: Add FavoritePage import and tab**

In `src/components/CutPage.vue`, add the import in the `<script setup>` section. Insert after the existing `import { PushpinOutlined } from '@ant-design/icons-vue'` line:

```javascript
import FavoritePage from './FavoritePage.vue'
```

- [ ] **Step 2: Add the "收藏" tab pane**

In the `<template>` section, add a new `<a-tab-pane>` between the "图片" tab and the "白板" tab. Insert after the closing `</a-tab-pane>` of the image list tab and before the "白板" tab:

```html
      <!-- 收藏列表标签页 -->
      <a-tab-pane key="favoriteList" tab="收藏">
        <favorite-page></favorite-page>
      </a-tab-pane>
```

The final tab order should be: 文本 → 图片 → 收藏 → 白板.

- [ ] **Step 3: Commit**

```bash
git add src/components/CutPage.vue
git commit -m "feat: add favorites tab to CutPage"
```

---

### Task 4: Wire up TimeList.vue group selection modal to real db_service methods

**Files:**
- Modify: `src/components/TimeList.vue`

- [ ] **Step 1: Replace the stub `queryGroups` function**

In `src/components/TimeList.vue`, find the existing stub `queryGroups` function (around line 306-308):

```javascript
const queryGroups = () => {
  // 预留接口
}
```

Replace it with:

```javascript
const queryGroups = async () => {
  const result = await dbService.fetchGroups()
  groupList.value = result || []
}
```

Note: `dbService` is already imported as `dbService` in this file (line 157: `import dbService from '../db_service'`).

- [ ] **Step 2: Replace the stub `addGroupItem` function**

Find the existing stub `addGroupItem` function (around line 314-316):

```javascript
const addGroupItem = (groupItem) => {
  // 预留接口
}
```

Replace it with:

```javascript
const addGroupItem = async (groupItem) => {
  const result = await dbService.addGroupItem(groupItem.groupId, groupItem.content, groupItem.title)
  if (result) {
    showMessageShort('已收藏到分组')
  } else {
    showMessageShort('收藏失败')
  }
}
```

- [ ] **Step 3: Update the group selection modal to include "新建分组" functionality**

Find the existing group selection modal (around lines 75-89):

```html
    <!-- 分组选择模态框 -->
    <div style="overflow: scroll;">
      <a-modal 
        v-model:open="groupSelectOpen" 
        title="添加分组" 
        ok-text="确认" 
        cancel-text="取消" 
        @ok="addItemToGroup()"
      >
      <a-radio-group v-model:value="groupSelectId">
          <div v-for="item, index in groupList" :key="item.id">
          <a-radio :style="radioStyle" :value="item.id">{{ item.name }}</a-radio>
        </div>
      </a-radio-group>
    </a-modal>
    </div>
```

Replace it with:

```html
    <!-- 分组选择模态框 -->
    <a-modal
      v-model:open="groupSelectOpen"
      title="添加到分组"
      ok-text="确认"
      cancel-text="取消"
      @ok="addItemToGroup()"
    >
      <a-radio-group v-model:value="groupSelectId" style="width: 100%;">
        <div v-for="item in groupList" :key="item.id" style="margin-bottom: 8px;">
          <a-radio :value="item.id">{{ item.name }}</a-radio>
        </div>
      </a-radio-group>

      <a-divider v-if="groupList.length > 0" style="margin: 8px 0;" />

      <div style="display: flex; align-items: center; gap: 8px;">
        <a-input
          v-model:value="quickNewGroupName"
          placeholder="新建分组名称"
          size="small"
          style="flex: 1;"
          @pressEnter="handleQuickCreateGroup"
        />
        <a-button size="small" type="primary" @click="handleQuickCreateGroup">
          <template #icon><PlusOutlined /></template>
        </a-button>
      </div>
    </a-modal>
```

- [ ] **Step 4: Add new reactive data and methods for quick group creation**

Add `quickNewGroupName` ref and `handleQuickCreateGroup` method. Find the reactive data section (around lines 199-207), add after `const currCutItem = ref({})`:

```javascript
const quickNewGroupName = ref('') // 快捷新建分组名称
```

Add `radioStyle` ref (it was referenced in the old template but not defined, add it now):

```javascript
const radioStyle = { display: 'flex', height: '30px', lineHeight: '30px' }
```

Add the `handleQuickCreateGroup` method after the `addItemToGroup` function (around line 391):

```javascript
const handleQuickCreateGroup = async () => {
  if (!quickNewGroupName.value.trim()) {
    showMessageShort('分组名称不能为空')
    return
  }
  const group = await dbService.addGroup(quickNewGroupName.value.trim())
  if (group) {
    await queryGroups()
    groupSelectId.value = group.id
    quickNewGroupName.value = ''
    showMessageShort('分组创建成功')
  }
}
```

- [ ] **Step 5: Update `openGroupSelect` to also load groups**

Find the `openGroupSelect` function (around line 370-374):

```javascript
const openGroupSelect = (item) => {
    groupSelectOpen.value = true
    currCutItem.value = item
    queryGroups()
}
```

Replace with:

```javascript
const openGroupSelect = async (item) => {
  currCutItem.value = item
  groupSelectId.value = ''
  quickNewGroupName.value = ''
  await queryGroups()
  groupSelectOpen.value = true
}
```

- [ ] **Step 6: Add PlusOutlined import**

Find the imports section (around line 152):

```javascript
import { MoreOutlined, DeleteOutlined, EditOutlined, GroupOutlined, CopyOutlined } from '@ant-design/icons-vue'
```

Add `PlusOutlined`:

```javascript
import { MoreOutlined, DeleteOutlined, EditOutlined, GroupOutlined, CopyOutlined, PlusOutlined } from '@ant-design/icons-vue'
```

- [ ] **Step 7: Commit**

```bash
git add src/components/TimeList.vue
git commit -m "feat: wire up TimeList group selection modal to real db_service methods"
```

---

### Task 5: Verify and integration test

**Files:**
- No new files

- [ ] **Step 1: Build and run the application**

Run: `npm run tauri dev`

Verify:
1. The "收藏" tab appears between "图片" and "白板"
2. Clicking "收藏" tab shows the FavoritePage with empty state
3. Creating a new group works via the "+新建" button
4. Switching groups in the dropdown works
5. In the "文本" tab, clicking `⋯` → "分组" opens the modal with real group list
6. Selecting a group and confirming adds the text as a favorite
7. Back in the "收藏" tab, the item appears under the selected group
8. Group management modal (⚙ button) allows rename and delete
9. Deleting a group removes its favorites too
10. Double-clicking a favorite copies it to clipboard

- [ ] **Step 2: Final commit (if any fixes needed)**

```bash
git add -A
git commit -m "feat: favorites feature complete — group-based text favorites"
```
