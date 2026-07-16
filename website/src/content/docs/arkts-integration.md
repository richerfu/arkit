---
title: ArkTS 工程接入
description: "NativeAbility、XComponent 与生命周期。"
---

# ArkTS 工程接入

Arkit native 模块通过 `@ohos-rs/ability` 接入 ArkTS。业务侧不需要手写 `NodeContent`、`ContentSlot` 或 N-API 生命周期函数。

## 安装 Ability 包

```sh
ohpm install @ohos-rs/ability
```

也可以在 `oh-package.json5` 声明：

```json5
{
  dependencies: {
    "@ohos-rs/ability": "^0.1.0",
  },
}
```

版本号应与当前 OpenHarmony 工程使用的 binding 代际保持一致。

## 默认承载页面

`entry/src/main/ets/entryability/EntryAbility.ets`：

```ts
import { NativeAbility } from "@ohos-rs/ability";

export default class EntryAbility extends NativeAbility {
  public moduleName: string = "counter";
}
```

`moduleName` 是 Rust 动态库裸模块名：

| 构建产物        | moduleName |
| --------------- | ---------- |
| `libcounter.so` | `counter`  |
| `libmy_app.so`  | `my_app`   |

不要写 `lib` 前缀或 `.so` 后缀。默认页面会创建承载 native UI 的 XComponent，并调用模块的 `init/render`。

## 自定义 ArkTS 页面

需要和 ArkTS 组件混排时关闭默认页面：

```ts
import { NativeAbility } from "@ohos-rs/ability";
import { AbilityConstant, Want } from "@kit.AbilityKit";
import window from "@ohos.window";

export default class EntryAbility extends NativeAbility {
  public moduleName: string = "counter";
  public defaultPage: boolean = false;

  async onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): Promise<void> {
    super.onCreate(want, launchParam);
  }

  async onWindowStageCreate(windowStage: window.WindowStage): Promise<void> {
    super.onWindowStageCreate(windowStage);
    await windowStage.loadContent("pages/Index");
  }
}
```

自定义页面用 `DefaultXComponent` 挂载：

```ts
import { DefaultXComponent } from '@ohos-rs/ability'

const MODULE_NAME = 'counter'

@Entry
@Component
struct Index {
  build() {
    Column() {
      Text('Arkit Counter')
        .fontSize(24)
        .margin({ bottom: 12 })

      DefaultXComponent({ moduleName: MODULE_NAME })
        .width('100%')
        .layoutWeight(1)
    }
    .width('100%')
    .height('100%')
  }
}
```

## 多模块

一个 Ability 初始化多个 Rust 动态库时使用数组：

```ts
export default class EntryAbility extends NativeAbility {
  public moduleName: string[] = ["counter", "analytics"];
}
```

页面中的每个 `DefaultXComponent` 仍指定单个模块名。每个 Arkit root 拥有独立 VirtualDom、renderer、窗口 context 和动画 host。

## 加载模式

默认异步加载 native 模块：

```ts
public loadMode: 'async' | 'sync' = 'async'
```

确实需要同步初始化时：

```ts
public loadMode: 'async' | 'sync' = 'sync'
```

同步模式要求 native library 已正确进入运行包，且初始化成本不会阻塞 Ability 启动。

## 生命周期规则

覆盖 `NativeAbility` 生命周期时必须调用 `super`：

```ts
async onWindowStageCreate(windowStage: window.WindowStage): Promise<void> {
  super.onWindowStageCreate(windowStage)
  await windowStage.loadContent('pages/Index')
}
```

`super` 负责 helper、window callbacks、avoid area、keyboard、back press 和 native module 生命周期。漏掉它会导致页面能加载但安全区、异步唤醒或销毁不完整。

## 接口对照

| API                 | 说明                                          |
| ------------------- | --------------------------------------------- |
| `NativeAbility`     | 加载 native 模块并转发生命周期的 Ability 基类 |
| `moduleName`        | Rust 动态库裸模块名或名称数组                 |
| `defaultPage`       | 是否使用默认承载页，默认 `true`               |
| `loadMode`          | `async` 或 `sync`，默认 `async`               |
| `DefaultXComponent` | 在自定义 ArkTS 页面中承载某个 Arkit root      |

## 常见问题

- 白屏但无链接错误：先确认 `DefaultXComponent` 的实际尺寸不为零。
- 返回键无效：确认 lifecycle `super` 被调用，并在 Router tree 中调用了 `use_back_handler()`。
- 安全区重复：非 edge-to-edge ArkTS 宿主可能已经避让系统栏；Arkit 会按 XComponent `content_rect` 求交，但业务不要再硬编码状态栏高度。
- WebView helper 不可用：必须通过 `NativeAbility`/`DefaultXComponent` 的标准 render 流程进入，不能只手动调用动态库 export。
