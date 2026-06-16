# Focus Timer (专注时间) Design

## Overview

Add a "专注" (Focus) tab to the existing CutPage tab bar, providing a Pomodoro-style timer. Users can set focus duration and rest duration, choose single or loop mode, and receive system notifications when focus/rest periods start and end.

## Requirements

- New "专注" tab in CutPage
- Single focus mode: one focus session then stop
- Loop focus mode: focus → rest → focus → rest … until manually stopped (infinite loop)
- Preset durations (focus: 15/25/30/45/60 min, rest: 5/10/15 min) + custom input
- System notifications on focus start, focus end, rest end
- Timer runs in background; tab title shows countdown when active
- No history records saved (pure timer, no database)

## Architecture

### Approach: Pure Frontend Timer (Option A)

Timer logic runs entirely in Vue frontend with `setInterval`. No database tables, no Rust changes. Notifications use `@tauri-apps/plugin-notification` (same pattern as `todo_notifier.js`).

Rationale: No persistence needed; `setInterval` precision is sufficient for minute-level countdowns; consistent with existing notification pattern.

### State Machine

```
idle → focusing → resting → focusing → resting → … → idle
                ↘ paused → focusing (resume)
any state → idle (stop)
```

States: `idle | focusing | resting | paused`

## Files

| File | Change | Purpose |
|------|--------|---------|
| `src/components/FocusPage.vue` | New | Focus tab UI component |
| `src/focus_timer.js` | New | Timer service: state, countdown, notifications |
| `src/components/CutPage.vue` | Modify | Add "专注" tab, dynamic tab title with countdown |

No database changes. No Rust changes.

## focus_timer.js — Timer Service

### Reactive State (Vue `reactive()`)

```js
{
  state: 'idle' | 'focusing' | 'resting' | 'paused',
  remainingSeconds: 0,
  currentRound: 0,        // rounds completed (loop mode)
  totalFocusSeconds: 0,   // cumulative focus seconds this session
}
```

### API

| Method | Description |
|--------|-------------|
| `start(config)` | Start focus session. `config: { focusMinutes, restMinutes, mode: 'once' \| 'loop' }` |
| `pause()` | Pause countdown (only in `focusing` state) |
| `resume()` | Resume countdown |
| `stop()` | Stop entire session, reset to `idle` |
| `skipRest()` | Skip current rest, enter next focus round |

### Countdown Logic

- `setInterval(1000)` decrements `remainingSeconds` each tick
- When `remainingSeconds` reaches 0, transition state:
  - `focusing` → `resting` (or `idle` if single mode)
  - `resting` → `focusing` (increment `currentRound`)
- Pause: `clearInterval`, save remaining time
- Resume: restart `setInterval` with remaining time
- Stop: `clearInterval`, reset all state

### Notifications

| Event | Title | Body |
|-------|-------|------|
| Focus starts | 🍅 专注开始 | 开始专注，加油！ |
| Focus ends (enter rest) | ✅ 专注完成 | 已专注 X 分钟，休息一下！ |
| Rest ends (next round) | 🍅 休息结束 | 准备开始新一轮专注！ |
| Single focus completes | ✅ 专注完成 | 已专注 X 分钟！ |
| Manual stop | — | No notification (user-initiated) |

Uses `@tauri-apps/plugin-notification` with `sendNotification()`, same as `todo_notifier.js`.

## FocusPage.vue — UI Component

### Idle State (Setup View)

```
┌─────────────────────────────┐
│         专注时间             │
│                             │
│   专注时长                  │
│   [15] [25] [30] [45] [60]  │  ← Preset buttons, selected highlighted
│   自定义: [____] 分钟        │
│                             │
│   休息时长                  │
│   [5] [10] [15]             │  ← Preset buttons
│   自定义: [____] 分钟        │
│                             │
│   专注模式                  │
│   ○ 单次专注  ● 循环专注     │  ← Radio group
│                             │
│        [ 🍅 开始专注 ]       │  ← Primary button
│                             │
└─────────────────────────────┘
```

### Focusing State (Countdown View)

```
┌─────────────────────────────┐
│                             │
│         24:58               │  ← Large countdown display
│       专注中...              │  ← Status text
│                             │
│   ● ● ● ○                  │  ← Round indicators (loop mode)
│                             │
│   [ ⏸ 暂停 ]  [ ⏹ 停止 ]   │  ← Action buttons
│                             │
└─────────────────────────────┘
```

### Resting State

```
┌─────────────────────────────┐
│                             │
│         04:32               │  ← Large countdown display
│       休息中...              │  ← Status text (light green background)
│                             │
│   [ ⏹ 跳过休息 ]            │  ← Skip to next focus round
│   [ ⏹ 结束专注 ]            │  ← End entire session
│                             │
└─────────────────────────────┘
```

### Interaction Details

- **Pause**: Only available during `focusing`. Button toggles to "继续" when paused.
- **Skip rest**: Available during `resting` in loop mode. Starts next focus round immediately.
- **Stop**: Shows `Modal.confirm` before stopping. In loop mode, displays total focus time in confirmation.
- **Rest duration**: Only shown when mode is "循环专注". Hidden for single mode (no rest needed after single focus).

## CutPage.vue Integration

### Tab Addition

Add new `a-tab-pane` after "待办":

```html
<a-tab-pane key="focusList" tab="专注">
  <focus-page></focus-page>
</a-tab-pane>
```

### Dynamic Tab Title

When `focus_timer.state !== 'idle'`, the tab title shows the countdown:

```
[文本] [图片] [收藏] [待办] [专注 24:58] 📌 🔍
```

Implementation: computed property bound to `a-tab-pane`'s `tab` attribute, derived from `focus_timer.state` and `focus_timer.remainingSeconds`.

Format: `专注 ${mm}:${ss}` when active, `专注` when idle.

## Error Handling

- **Notification permission denied**: Silently skip notifications; timer still works
- **Timer drift**: Acceptable for minute-level countdowns; no correction needed
- **App restart**: Timer resets (no persistence by design)

## Out of Scope

- History/statistics persistence
- Custom notification sounds
- Auto-start focus from todo items
- Break activity suggestions
