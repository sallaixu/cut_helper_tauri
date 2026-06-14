import dbService from './db_service';
import { sendNotification, isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';

// 定时器映射: todoId -> [timerId1, timerId2, timerId3]
const timerMap = new Map();

/**
 * 请求通知权限
 */
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

/**
 * 发送系统通知
 */
function notify(title, body) {
  try {
    sendNotification({ title, body });
  } catch (error) {
    console.error('发送通知失败:', error);
  }
}

/**
 * 为单个 todo 注册定时器
 */
function registerTimers(todo) {
  // 先清除该 todo 已有的定时器
  unregisterTimers(todo.id);

  if (todo.status === 'done') return;

  const timers = [];
  const now = Date.now();

  // 提前提醒
  if (todo.startTime && todo.notifyAdvance > 0) {
    const advanceTime = new Date(todo.startTime).getTime() - todo.notifyAdvance * 60000;
    const delay = advanceTime - now;
    if (delay > 0) {
      timers.push(setTimeout(() => {
        notify('⏰ 即将开始', `${todo.title}（${todo.notifyAdvance}分钟后）`);
      }, delay));
    }
  }

  // 开始提醒
  if (todo.startTime && todo.notifyStart) {
    const delay = new Date(todo.startTime).getTime() - now;
    if (delay > 0) {
      timers.push(setTimeout(() => {
        notify('📋 待办开始', todo.title);
      }, delay));
    }
  }

  // 结束提醒
  if (todo.endTime && todo.notifyEnd) {
    const delay = new Date(todo.endTime).getTime() - now;
    if (delay > 0) {
      timers.push(setTimeout(() => {
        notify('✅ 待办结束', todo.title);
      }, delay));
    }
  }

  if (timers.length > 0) {
    timerMap.set(todo.id, timers);
  }
}

/**
 * 清除单个 todo 的所有定时器
 */
function unregisterTimers(todoId) {
  const timers = timerMap.get(todoId);
  if (timers) {
    timers.forEach(clearTimeout);
    timerMap.delete(todoId);
  }
}

/**
 * 初始化：加载所有未完成的 todo 并注册定时器
 */
async function init() {
  await ensurePermission();

  try {
    const todos = await dbService.fetchTodoItems();
    const activeTodos = todos.filter(t => t.status !== 'done');
    activeTodos.forEach(registerTimers);
    console.log(`通知调度器已初始化，注册了 ${activeTodos.length} 个待办通知`);
  } catch (error) {
    console.error('通知调度器初始化失败:', error);
  }
}

/**
 * 注册单个 todo 的通知（添加/修改时调用）
 */
function register(todo) {
  registerTimers(todo);
}

/**
 * 注销单个 todo 的通知（删除/完成时调用）
 */
function unregister(todoId) {
  unregisterTimers(todoId);
}

/**
 * 全量刷新：清除所有定时器，重新从数据库加载
 */
async function refreshAll() {
  // 清除所有定时器
  for (const timers of timerMap.values()) {
    timers.forEach(clearTimeout);
  }
  timerMap.clear();

  // 重新加载
  await init();
}

export default { init, register, unregister, refreshAll };
