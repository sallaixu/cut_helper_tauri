<template>
  <div class="cut-list" ref="scrollerRef" style="height: 100%;" @keydown="handleKeyDown">
    <!-- 回到顶部按钮 -->
    <a-back-top :target="() => getTarget()" />
    
    <!-- 虚拟滚动列表 -->
    <virt-list 
      class="scroller" 
      tabindex="0" 
      :list="showItemList" 
      itemKey="id" 
      :minSize="40" 
      id="cutItemBox" 
      ref="virtListRef"
      :key="listKey"
    >
      <template #default="{ itemData, index }">
        <div 
          class="list-item" 
          v-on:dblclick="sendCopyItem(itemData)" 
          :class="{ 'list-item-selected': currentItemIdx === index }" 
          style="padding: 5px"
        >
          <!-- 内容区域 -->
          <div style="height: 1.5em; line-height: 1.5em; flex: 1; overflow: hidden; margin-right: 6px;">
        <a-skeleton avatar :title="false" :loading="!!itemData.loading" active>
              <a-list-item-meta>
            <template #title>
              <a-popover trigger="hover" :mouseEnterDelay="1" placement="topLeft">
                <template #title>{{ formatDate(itemData.createTime) }}</template>
                <template #content>
                      <div class="detail-style" style="max-height: 80vh; max-width: 90vw;">
                    <pre>{{ itemData.content }}</pre>
                  </div>
                </template>
                    <div style="margin-right: 6px; white-space: nowrap;">
                  <li>{{ (index + 1) }} . {{ itemData.content }}</li>
                </div>
              </a-popover>
            </template>
          </a-list-item-meta>
        </a-skeleton>
        </div>
          
          <!-- 操作区域 -->
          <div style="display: flex; justify-content: space-between; align-items: center; width: fit-content">
          <div>{{ format(itemData.createTime, 'short') }}</div>
          <a-dropdown :trigger="['click']">
              <more-outlined class="jump" @click.prevent style="cursor: pointer; color: black;" />
            <template #overlay>
              <a-menu>
                <a-menu-item @click="deleteItem(itemData)" key="0" style="color: #f5222d;">
                  <div>
                    <delete-outlined /><span style="margin-left: 8px;">删除</span>
                  </div>
                </a-menu-item>
                <a-menu-item @click="openDetail(itemData)" key="1">
                  <div>
                    <EditOutlined /><span style="margin-left: 8px;">详情</span>
                  </div>
                </a-menu-item>
                <a-menu-item @click="openGroupSelect(itemData)" key="2">
                  <div>
                    <GroupOutlined /><span style="margin-left: 8px;">分组</span>
                  </div>
                </a-menu-item>
              </a-menu>
            </template>
          </a-dropdown>
        </div>
      </div>
      </template>
    </virt-list>
    
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
    
    <!-- 详情模态框 -->
    <a-modal 
      v-model:open="detailModalOpen" 
      title="详情信息" 
      width="800px"
      :footer="null"
      @cancel="closeDetail"
    >
      <div class="detail-modal-content">
        <!-- 基本信息 -->
        <a-descriptions :column="1" bordered size="small">
          <a-descriptions-item label="ID">
            {{ detailItem.id }}
          </a-descriptions-item>
          <a-descriptions-item label="创建时间">
            {{ formatDate(detailItem.createTime) }}
          </a-descriptions-item>
          <a-descriptions-item label="相对时间">
            {{ format(detailItem.createTime, 'short') }}
          </a-descriptions-item>
          <a-descriptions-item label="内容长度">
            {{ detailItem.content ? detailItem.content.length : 0 }} 字符
          </a-descriptions-item>
        </a-descriptions>
        
        <!-- 内容区域 -->
        <div style="margin-top: 16px;">
          <div style="font-weight: bold; margin-bottom: 8px;">内容详情：</div>
          <a-textarea 
            v-model:value="detailItem.content" 
            :rows="15" 
            readonly
            style="font-family: 'Courier New', monospace; white-space: pre-wrap; word-break: break-all;"
          />
        </div>
        
        <!-- 操作按钮 -->
        <div style="margin-top: 16px; text-align: right;">
          <a-space>
            <a-button @click="copyDetailContent" type="primary">
              <template #icon><CopyOutlined /></template>
              复制内容
            </a-button>
            <a-button @click="deleteDetailItem" danger>
              <template #icon><DeleteOutlined /></template>
              删除
            </a-button>
            <a-button @click="closeDetail">
              关闭
            </a-button>
          </a-space>
        </div>
      </div>
    </a-modal>
  </div>
</template>

<script setup>
import { format, register } from 'timeago.js'
import { ref, onMounted, computed, nextTick, watchEffect, watch } from 'vue'
import { MoreOutlined, DeleteOutlined, EditOutlined, GroupOutlined, CopyOutlined, PlusOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { VirtList } from 'vue-virt-list'
import { containsIgnoreCase } from '../../utils/StringUtil'
import { showMessageShort } from '../../utils/MessageUtil'
import dbService from '../db_service'
import { copyToSystem } from '../cut_service'

// ==================== 时间格式化配置 ====================
/**
 * 自定义时间格式化函数
 * @param {number} number - 数字
 * @param {number} index - 索引
 * @param {number} totalSec - 总秒数
 * @returns {Array} 时间格式数组
 */
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
  ][index]
}

// 注册自定义时间格式
register('short', localeFunc)

// ==================== 响应式数据 ====================
const allCutList = ref([]) // 全量剪切版数据
const currentItemIdx = ref(-1) // 当前选中项的索引
const showItemList = ref([]) // 显示的项目列表
const initLoading = ref(true) // 初始化加载状态
const searchKey = ref('') // 搜索关键词
const listKey = ref(0) // 列表键值
let timeoutId = null // 搜索防抖定时器

// 分组相关数据
const groupList = ref([]) // 分组列表
const groupSelectOpen = ref(false) // 分组选择模态框显示状态
const groupSelectId = ref('') // 选中的分组ID
const currCutItem = ref({}) // 当前操作的剪切项
const quickNewGroupName = ref('') // 快捷新建分组名称

// 详情相关数据
const detailModalOpen = ref(false) // 详情模态框显示状态
const detailItem = ref({}) // 详情项数据

// 组件引用
const listRef = ref(null)
const scrollerRef = ref(null)
const virtListRef = ref(null)

// ==================== 生命周期 ====================
onMounted(async () => {
  // 获取全量数据
  await sendQueryCutList()
  initLoading.value = false
  
  // 设置焦点到列表容器
  nextTick(() => {
    if (scrollerRef.value) {
      scrollerRef.value.focus()
    }
  })
})

// ==================== 响应式监听 ====================
watchEffect(() => {
  // 根据搜索关键词过滤显示列表
  if (searchKey.value) {
    showItemList.value = allCutList.value.filter(filterData)
  } else {
    showItemList.value = allCutList.value
  }
})

// 监听showItemList变化，强制虚拟滚动组件重新渲染
watch(showItemList, () => {
  nextTick(() => {
    listKey.value++
  })
}, { deep: true })

// ==================== 数据操作 ====================
/**
 * 查询全部剪切列表
 */
const sendQueryCutList = async () => {
  try {
    let result = await dbService.fetchItems()
    allCutList.value = result || []
    showItemList.value = allCutList.value
    
    nextTick(() => {
      listKey.value++
    })
  } catch (error) {
    console.error('加载数据失败:', error)
  }
}

/**
 * 删除剪切项
 * @param {Object} remove - 要删除的项目
 */
const sendDeleteItem = (remove) => {
    dbService.removeItem(remove.id)
}

/**
 * 打开详情窗口
 * @param {Object} item - 项目数据
 */
const openDetail = (item) => {
  detailItem.value = { ...item }
  detailModalOpen.value = true
}

/**
 * 关闭详情窗口
 */
const closeDetail = () => {
  detailModalOpen.value = false
}

/**
 * 复制详情内容
 */
const copyDetailContent = () => {
  copyToSystem(detailItem.value.content)
  showMessageShort('内容已复制到剪贴板')
}

/**
 * 删除详情项
 */
const deleteDetailItem = () => {
  deleteItem(detailItem.value)
  closeDetail()
}

/**
 * 查询所有分组
 */
const queryGroups = async () => {
  const result = await dbService.fetchGroups()
  groupList.value = result || []
}

/**
 * 添加分组项目
 * @param {Object} groupItem - 分组项目数据
 */
const addGroupItem = async (groupItem) => {
  const result = await dbService.addGroupItem(groupItem.groupId, groupItem.content, groupItem.title)
  if (result) {
    showMessageShort('已收藏到分组')
  } else {
    showMessageShort('收藏失败')
  }
}

/**
 * 复制项目到系统剪贴板
 * @param {Object} item - 要复制的项目
 */
const sendCopyItem = (item) => {
  console.log('复制项目:', item)
  copyToSystem(item.content)
  showMessageShort('拷贝成功')
}

/**
 * 更新剪切板列表（从外部调用）
 * @param {Object} value - 新的剪切项
 */
const update = (value) => {
  console.log('更新剪切项:', value)
  
  // 添加新项目到列表开头
  let len = allCutList.value.unshift(value)
  if (len >= 300) {
    allCutList.value.pop()
  }
      
  // 强制触发虚拟滚动组件重新渲染
  nextTick(() => {
    // 更新listKey来强制重新渲染虚拟滚动组件
    listKey.value++
    
    // 重置选中状态到第一项（新添加的项目）
    currentItemIdx.value = -1
    
    // 延迟滚动到顶部显示新添加的项目
    setTimeout(() => {
      // 滚动到顶部
      if (virtListRef.value?.$el) {
        virtListRef.value.$el.scrollTo({
          top: 0,
          behavior: 'smooth'
        })
      }
    }, 200)
  })
}

// 将update函数暴露给全局
window.addCutItemToList = update

// ==================== 分组操作 ====================
/**
 * 打开分组选择模态框
 * @param {Object} item - 项目数据
 */
const openGroupSelect = async (item) => {
  currCutItem.value = item
  groupSelectId.value = ''
  quickNewGroupName.value = ''
  await queryGroups()
  groupSelectOpen.value = true
}

/**
 * 添加项目到分组
 */
const addItemToGroup = () => {
    groupSelectOpen.value = false
    let item = currCutItem.value
  if (!item) {
      return
    }
    let tmpGroupItem = {
    groupId: groupSelectId.value,
    title: item.content,
    content: item.content,
    }
    addGroupItem(tmpGroupItem)
}

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

// ==================== 工具函数 ====================
/**
 * 获取滚动目标元素
 * @returns {HTMLElement} 目标元素
 */
const getTarget = () => {
  return document.getElementById('cutItemBox')
}

/**
 * 过滤数据
 * @param {Object} item - 项目数据
 * @returns {boolean} 是否匹配搜索条件
 */
const filterData = (item) => {
  return containsIgnoreCase(item.content, searchKey.value)
}

/**
 * 删除列表中的某一项
 * @param {Object} remove - 要删除的项目
 */
const deleteItem = (remove) => {
  let index = allCutList.value.findIndex(item => item.id === remove.id)
  if (index !== -1) {
    sendDeleteItem(remove)
    allCutList.value.splice(index, 1)
    showMessageShort('删除成功')
  } else {
    showMessageShort('删除失败')
  }
}

/**
 * 格式化日期
 * @param {string} dateStr - 日期字符串
 * @returns {string} 格式化后的日期
 */
const formatDate = (dateStr) => {
  const date = new Date(dateStr)
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')

  return `${year}年${month}月${day}日，${hours}:${minutes}:${seconds}`
}

/**
 * 设置焦点到列表容器
 */
const abtainFocus = () => {
  if (virtListRef.value?.$el) {
    virtListRef.value.$el.focus()
  } else if (scrollerRef.value) {
  scrollerRef.value.focus()
  }
}

// ==================== 搜索功能 ====================
/**
 * 搜索功能
 * @param {string} key - 搜索关键词
 */
const search = (key) => {
  clearTimeout(timeoutId) // 取消之前的定时器
  timeoutId = setTimeout(() => {
    console.log('搜索关键词:', key)
    searchKey.value = key
    // 搜索时重置选中状态到第一项
    currentItemIdx.value = 0
  }, 500)
}

/**
 * 键盘导航
 * @param {KeyboardEvent} event - 键盘事件对象
 */
const handleKeyDown = (event) => {
  const key = event.key
  
  switch (key) {
    case 'ArrowUp':
      event.preventDefault()
      navigateUp()
      break
    case 'ArrowDown':
      event.preventDefault()
      navigateDown()
      break
    case 'Enter':
      event.preventDefault()
      if (currentItemIdx.value >= 0 && currentItemIdx.value < showItemList.value.length) {
        sendCopyItem(showItemList.value[currentItemIdx.value])
      }
      break
    case 'Home':
      event.preventDefault()
      navigateToFirst()
      break
    case 'End':
      event.preventDefault()
      navigateToLast()
      break
  }
}

/**
 * 向上导航
 */
const navigateUp = () => {
  if (showItemList.value.length === 0) return
  
  if (currentItemIdx.value > 0) {
    currentItemIdx.value--
  }
  
  // 立即检查并滚动，使用instant模式确保快速响应
  immediateScrollCheck()
}

/**
 * 向下导航
 */
const navigateDown = () => {
  if (showItemList.value.length === 0) return
  
  if (currentItemIdx.value < showItemList.value.length - 1) {
    currentItemIdx.value++
  }
  
  // 立即检查并滚动，使用instant模式确保快速响应
  immediateScrollCheck()
}

/**
 * 立即检查并滚动（不使用防抖）
 */
const immediateScrollCheck = () => {
  // 使用requestAnimationFrame确保在下一帧立即执行，提供最佳性能
  requestAnimationFrame(() => {
    // 每次都执行滚动检查，确保选中项始终在可视区域内
    scrollToSelectedItem(true) // 使用立即滚动模式，不使用动画
  })
}

/**
 * 跳转到第一项
 */
const navigateToFirst = () => {
  if (showItemList.value.length === 0) return
  currentItemIdx.value = 0
  scrollToSelectedItem()
}

/**
 * 跳转到最后一项
 */
const navigateToLast = () => {
  if (showItemList.value.length === 0) return
  currentItemIdx.value = showItemList.value.length - 1
  scrollToSelectedItem()
}

// ==================== 滚动控制 ====================
/**
 * 检测选中项是否在可视区域内
 * @returns {boolean} 是否可见
 */
const isSelectedItemVisible = () => {
  if (currentItemIdx.value < 0 || currentItemIdx.value >= showItemList.value.length) {
    return false
  }
  
  // 快速获取滚动容器
  const scrollContainer = virtListRef.value?.$el || document.getElementById('cutItemBox') || document.querySelector('.scroller')
  
  if (!scrollContainer) {
    return false
  }
  
  const itemHeight = 40
  const itemTop = currentItemIdx.value * itemHeight
  const itemBottom = itemTop + itemHeight
  const visibleTop = scrollContainer.scrollTop
  const visibleBottom = visibleTop + scrollContainer.clientHeight
  
  // 检查项目是否在可视区域内（允许部分可见）
  return itemTop < visibleBottom && itemBottom > visibleTop
}

/**
 * 滚动到选中项
 * @param {boolean} instant - 是否立即滚动（不使用动画）
 */
const scrollToSelectedItem = (instant = false) => {
  if (currentItemIdx.value < 0 || currentItemIdx.value >= showItemList.value.length) {
    return
  }
  
  // 获取滚动容器
  const scrollContainer = virtListRef.value?.$el || document.getElementById('cutItemBox') || document.querySelector('.scroller')
  
  if (!scrollContainer) {
    return
  }
  
  // 查找当前选中的DOM元素
  const selectedElement = document.querySelector('.list-item-selected')
  
  if (!selectedElement) {
    // 如果元素还未渲染（在虚拟滚动外），尝试使用虚拟滚动的方法
    if (virtListRef.value && typeof virtListRef.value.scrollToIndex === 'function') {
      try {
        virtListRef.value.scrollToIndex(currentItemIdx.value)
      } catch (error) {
        console.warn('scrollToIndex 调用失败:', error)
      }
    }
    return
  }
  
  // 使用 getBoundingClientRect 获取实际位置
  const containerRect = scrollContainer.getBoundingClientRect()
  const itemRect = selectedElement.getBoundingClientRect()
  
  // 计算选中项相对于容器的位置
  const offsetTop = itemRect.top - containerRect.top
  const offsetBottom = itemRect.bottom - containerRect.bottom
  
  // 设置缓冲区（避免紧贴边缘）
  const buffer = 10
  
  // 如果选中项在可视区域上方
  if (offsetTop < buffer) {
    scrollContainer.scrollTop += offsetTop - buffer
  }
  // 如果选中项在可视区域下方
  else if (offsetBottom > -buffer) {
    scrollContainer.scrollTop += offsetBottom + buffer
  }
}

// ==================== 组件暴露 ====================
/**
 * 暴露方法供父组件调用
 * @property {Function} search - 搜索功能
 * @property {Function} abtainFocus - 获取焦点
 * @property {Ref} currentItemIdx - 当前选中项索引
 * @property {Function} navigateToFirst - 跳转到第一项
 * @property {Function} navigateUp - 向上导航
 * @property {Function} navigateDown - 向下导航
 * @property {Function} navigateToLast - 跳转到最后一项
 */
defineExpose({ 
  search, 
  abtainFocus, 
  currentItemIdx, 
  navigateToFirst, 
  navigateUp, 
  navigateDown, 
  navigateToLast 
})
</script>

<style scoped>
/* ==================== 列表项操作按钮样式 ==================== */
.cut-list ::v-deep(.ant-list-item-action li) {
  padding: 0 !important;
}

.cut-list ::v-deep(.ant-list-item-action) {
  margin-left: 5px !important;
}

.cut-list ::v-deep(.ant-list-item) {
  padding-right: 3px !important;
}

/* ==================== 背景样式 ==================== */
.bg {
  background-color: rgb(221, 219, 219);
}

/* ==================== 列表项样式 ==================== */
.list-item {
  display: flex;
  height: 3em;
  align-items: center;
}

.list-item:hover {
  background-color: rgb(171, 225, 153);
}

.list-item-selected {
  background-color: rgb(171, 225, 153);
}

/* ==================== 焦点样式 ==================== */
.cut-list:focus {
  outline: none;
}

.list-item:focus {
  outline: none;
}

/* ==================== 交互样式 ==================== */
.click—class {
  cursor: pointer !important;
}

/* ==================== 详情样式 ==================== */
.detail-style {
  overflow-y: scroll;
  overflow-x: scroll;
}

/* ==================== 滚动容器样式 ==================== */
.scroller {
  height: 100%;
}

/* ==================== 详情模态框样式 ==================== */
.detail-modal-content {
  padding: 8px 0;
}

.detail-modal-content :deep(.ant-descriptions-item-label) {
  font-weight: 600;
  background-color: #fafafa;
}

.detail-modal-content :deep(.ant-textarea) {
  resize: vertical;
}

.detail-modal-content :deep(.ant-space) {
  gap: 8px !important;
}
</style>