import { AsyncPluginBase, LazyPlugin, NativeAbility } from "@ohos-rs/ability";
import type {
  BridgeContextRequirement,
  BridgeLifecycleEvent,
  BridgeTypedValue,
} from "@ohos-rs/ability";
import { WebviewPlugin } from "@ohos-rs/ability-plugin-webview";
import { abilityAccessCtrl, Permissions } from "@kit.AbilityKit";
import window from "@ohos.window";

// plugins example（examples/plugins）在 Rust 侧用 #[entry(plugins = [...])]
// 声明式注册了 'arkit.example.declarative'；@ohos-rs/ability 的桥接契约要求
// 每个 Rust 声明必须有匹配的 ArkTS 工厂（id / execution / requires 一致），
// 这一侧由宿主手工提供。其他 demo 不声明该插件，此候选不会被选中安装。
class DeclarativeDemoPlugin extends AsyncPluginBase {
  readonly id = "arkit.example.declarative";
  readonly requires: BridgeContextRequirement[] = [];

  onLifecycle(event: BridgeLifecycleEvent): void {
    console.info(`arkit declarative plugin lifecycle: ${event.kind}`);
  }

  async invokeAsync(_action: string, _request: BridgeTypedValue): Promise<BridgeTypedValue> {
    return { typeName: "std.void", value: undefined } as BridgeTypedValue;
  }
}

// moduleName 由 run.sh 按当前部署的 example 写入（对应 lib<crate>.so）。
// NativeAbility 据此加载模块并管理 bridge session 与 render owner 生命周期。
export default class EntryAbility extends NativeAbility {
  public moduleName: string = "shadcn_showcase";
  public defaultPage: boolean = false;

  // ArkTS 侧能力插件。webview 的 Rust facade 由 arkit 在 init 阶段自动注册
  // （`arkit/webview` feature），这里安装的 WebviewPlugin 只在 Rust 侧发起
  // bridge 调用时被使用；未启用 feature 的应用不受影响。
  public bridgePlugins = [
    new LazyPlugin(() => new WebviewPlugin()),
    new LazyPlugin(() => new DeclarativeDemoPlugin()),
  ];

  protected async loadWindowStageContent(windowStage: window.WindowStage): Promise<void> {
    try {
      if (this.moduleName === "camera_example") {
        const cameraPermission: Permissions = "ohos.permission.CAMERA";
        await abilityAccessCtrl.createAtManager()
          .requestPermissionsFromUser(this.context, [cameraPermission]);
      }
      await windowStage.loadContent("pages/Index");
    } catch (error) {
      const message = `arkit failed to create the main window stage: ${JSON.stringify(error)}`;
      console.error(message);
      throw new Error(message);
    }
  }
}
