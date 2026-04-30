# ArkTS 接入

Arkit 应用在 ArkTS 工程中通过 `@ohos-rs/ability` 接入。业务侧不需要手写 `NodeContent`、`ContentSlot`、`init`、`render`。

## 安装依赖

在 OpenHarmony 工程中安装：

```sh
ohpm install @ohos-rs/ability
```

或在 `oh-package.json5` 中添加：

```json5
{
  "dependencies": {
    "@ohos-rs/ability": "^0.1.0"
  }
}
```

版本号以实际发布版本为准。

## 默认页面接入

`entry/src/main/ets/entryability/EntryAbility.ets`：

```ts
import { NativeAbility } from '@ohos-rs/ability'

export default class EntryAbility extends NativeAbility {
  public moduleName: string = 'counter'
}
```

`moduleName` 填 Rust 动态库的裸模块名：

| 构建产物 | `moduleName` |
| --- | --- |
| `libcounter.so` | `counter` |
| `libmy_app.so` | `my_app` |

不要写 `lib` 前缀，也不要写 `.so` 后缀。

## 自定义 ArkTS 页面

如果需要在 ArkTS 页面中混排按钮、标题或其他 ArkUI 组件，关闭默认页面：

```ts
import { NativeAbility } from '@ohos-rs/ability'
import { AbilityConstant, Want } from '@kit.AbilityKit'
import window from '@ohos.window'

export default class EntryAbility extends NativeAbility {
  public moduleName: string = 'counter'
  public defaultPage: boolean = false

  async onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): Promise<void> {
    super.onCreate(want, launchParam)
  }

  async onWindowStageCreate(windowStage: window.WindowStage): Promise<void> {
    super.onWindowStageCreate(windowStage)
    await windowStage.loadContent('pages/Index')
  }
}
```

`entry/src/main/ets/pages/Index.ets`：

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
    .padding(16)
  }
}
```

## 多模块

一个 Ability 需要初始化多个 Rust 动态库时，`moduleName` 可以是数组：

```ts
export default class EntryAbility extends NativeAbility {
  public moduleName: string[] = ['counter', 'analytics']
}
```

自定义页面中使用某个模块：

```ts
DefaultXComponent({ moduleName: 'counter' })
```

## 加载模式

默认使用异步加载：

```ts
export default class EntryAbility extends NativeAbility {
  public moduleName: string = 'counter'
  public loadMode: 'async' | 'sync' = 'async'
}
```

如需同步加载：

```ts
export default class EntryAbility extends NativeAbility {
  public moduleName: string = 'counter'
  public loadMode: 'async' | 'sync' = 'sync'
}
```

同步加载时，需要确保对应 native library 已按工程要求配置到运行包中。

## 覆写生命周期

如果覆写 `NativeAbility` 生命周期，必须调用 `super`：

```ts
async onWindowStageCreate(windowStage: window.WindowStage): Promise<void> {
  super.onWindowStageCreate(windowStage)
  await windowStage.loadContent('pages/Index')
}
```

## API

| API | 说明 |
| --- | --- |
| `NativeAbility` | ArkTS Ability 基类，负责加载 native 模块并转发生命周期。 |
| `moduleName` | Rust 动态库裸模块名。 |
| `defaultPage` | 是否使用默认承载页面，默认 `true`。 |
| `loadMode` | native 模块加载方式，默认 `async`。 |
| `DefaultXComponent` | 在自定义 ArkTS 页面中承载 Rust UI。 |
