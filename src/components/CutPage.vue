<template>
  <div class="box">
    <a-tabs class="box-tabs" style="height: 100%" v-model:activeKey="activeKey" type="card" size="small">
      <!-- 文本列表标签页 -->
      <a-tab-pane key="timeList" tab="文本" style="height:100%;">
        <time-list id="timelist" ref="timeListVue"></time-list>
      </a-tab-pane>
      
      <!-- 图片列表标签页 -->
      <a-tab-pane key="imageList" tab="图片">
        <image-item></image-item>
      </a-tab-pane>

      <!-- 收藏列表标签页 -->
      <a-tab-pane key="favoriteList" tab="收藏">
        <favorite-page></favorite-page>
      </a-tab-pane>

      <!-- 待办列表标签页 -->
      <a-tab-pane key="todoList" tab="待办">
        <todo-page></todo-page>
      </a-tab-pane>

      <!-- 白板标签页 -->
      <a-tab-pane key="time" tab="白板">
        <text-edit></text-edit>
      </a-tab-pane>
      
      <!-- 标签页右侧额外内容 -->
      <template #rightExtra>
        <!-- 置顶按钮 -->
        <a-button :type="appConfig.top ? 'primary' : 'default'" @click="toggleTop">
          <template #icon><PushpinOutlined /></template>
        </a-button>
        
        <!-- 搜索输入框 -->
        <a-input 
          ref="searchInputRef" 
          v-model:value="searchKey" 
          tabindex="0" 
          @keydown="handleSearchKeyDown" 
          @change="handleSearch" 
          size="default" 
          style="width: 8em; margin-left: 4px;" 
          placeholder="搜索" 
          allow-clear 
        />
      </template>
    </a-tabs>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { PushpinOutlined } from '@ant-design/icons-vue'
import FavoritePage from './FavoritePage.vue'
import TodoPage from './TodoPage.vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

// ==================== 响应式数据 ====================
const searchKey = ref('') // 搜索关键词
const activeKey = ref('timeList') // 当前激活的标签页
const timeListVue = ref(null) // TimeList组件引用
const searchInputRef = ref(null) // 搜索框引用

// 应用配置
const appConfig = ref({
  top: true // 窗口置顶状态
})

// ==================== 窗口管理 ====================
/**
 * 更新窗口置顶状态
 * @param {boolean} isTop - 是否置顶
 */
const updateTop = (isTop) => {
  getCurrentWindow().setAlwaysOnTop(isTop)
}

/**
 * 切换窗口置顶状态
 */
const toggleTop = () => {
  appConfig.value.top = !appConfig.value.top
  updateTop(appConfig.value.top)
}

// ==================== 搜索功能 ====================
/**
 * 处理搜索关键词变化
 */
const handleSearch = () => {
  console.log('搜索关键词:', searchKey.value)
  if (timeListVue.value) {
    timeListVue.value.search(searchKey.value)
  }
}

/**
 * 处理搜索框键盘事件
 * @param {KeyboardEvent} event - 键盘事件对象
 */
const handleSearchKeyDown = (event) => {
  console.log('搜索框按键:', event.key)
  
  // 处理上下箭头键 - 切换到列表导航
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    if (timeListVue.value) {
      // 先让TimeList获取焦点
      timeListVue.value.abtainFocus()
      
      // 延迟执行导航，确保焦点切换完成
      setTimeout(() => {
        // 如果TimeList还没有选中项，设置第一个为选中项
        if (timeListVue.value.currentItemIdx === undefined || timeListVue.value.currentItemIdx < 0) {
          timeListVue.value.navigateToFirst?.()
        } else if (event.key === 'ArrowDown') {
          // 如果已经有选中项，直接向下导航
          timeListVue.value.navigateDown?.()
        } else if (event.key === 'ArrowUp') {
          // 如果已经有选中项，直接向上导航
          timeListVue.value.navigateUp?.()
        }
      }, 10)
    }
  } 
  // 处理回车键 - 切换到列表并选中第一项
  else if (event.key === 'Enter') {
    event.preventDefault()
    if (timeListVue.value) {
      timeListVue.value.abtainFocus()
      setTimeout(() => {
        timeListVue.value.navigateToFirst?.()
      }, 10)
    }
  }
}

// ==================== 全局快捷键 ====================
/**
 * 处理全局键盘事件
 * @param {KeyboardEvent} event - 键盘事件对象
 */
const handleGlobalKeyDown = (event) => {
  // 检查是否按下 Ctrl + F
  if (event.ctrlKey && event.key === 'f') {
    event.preventDefault() // 阻止默认的浏览器搜索行为
    focusSearchInput()
  }
}

/**
 * 设置焦点到搜索框
 */
const focusSearchInput = () => {
  if (searchInputRef.value) {
    searchInputRef.value.focus()
    // 选中搜索框中的文本，方便用户直接输入新的搜索内容
    searchInputRef.value.select()
  }
}

// ==================== 生命周期 ====================
onMounted(() => {
  // 添加全局键盘事件监听器
  document.addEventListener('keydown', handleGlobalKeyDown)
})

onUnmounted(() => {
  // 移除全局键盘事件监听器，防止内存泄漏
  document.removeEventListener('keydown', handleGlobalKeyDown)
})
</script>

<style scoped>
.box {
  height: 100vh;
  width: 100vw;
}

.box-tabs :deep() .ant-tabs-content {
  height: 100%;
}

.box :deep() .ant-list-item-action {
  margin-left: 10px !important;
}

.box :deep() .ant-tabs-nav {
  margin: 0 0 2px 0;
}
</style>