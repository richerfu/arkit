import { LazyPlugin, NativeAbility } from "@ohos-rs/ability";
import { WebviewPlugin } from "@ohos-rs/ability-plugin-webview";
import Want from "@ohos.app.ability.Want";
import { AbilityConstant, abilityAccessCtrl, Permissions } from "@kit.AbilityKit";
import window from "@ohos.window";

// moduleName 由 run.sh 按当前部署的 example 写入（对应 lib<crate>.so）。
// NativeAbility 据此 Loadable.load 加载并调 init/render/destroy。
export default class EntryAbility extends NativeAbility {
  public moduleName: string = "shadcn_showcase";
  public defaultPage: boolean = false;

  // ArkTS 侧能力插件。webview 的 Rust facade 由 arkit 在 init 阶段自动注册
  // （`arkit/webview` feature），这里安装的 WebviewPlugin 只在 Rust 侧发起
  // bridge 调用时被使用；未启用 feature 的应用不受影响。
  public bridgePlugins = [new LazyPlugin(() => new WebviewPlugin())];

  async onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): Promise<void> {
    super.onCreate(want, launchParam);
  }

  async onWindowStageCreate(windowStage: window.WindowStage): Promise<void> {
    try {
      super.onWindowStageCreate(windowStage);
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
