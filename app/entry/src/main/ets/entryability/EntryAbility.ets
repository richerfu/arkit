import { LazyPlugin, NativeAbility } from "@ohos-rs/ability";
import { WebviewPlugin } from "@ohos-rs/ability-plugin-webview";
import { abilityAccessCtrl, Permissions } from "@kit.AbilityKit";
import window from "@ohos.window";

// moduleName 由 run.sh 按当前部署的 example 写入（对应 lib<crate>.so）。
// NativeAbility 据此加载模块并管理 bridge session 与 render owner 生命周期。
export default class EntryAbility extends NativeAbility {
  public moduleName: string = "colorui_showcase";
  public defaultPage: boolean = false;

  // ArkTS 侧能力插件。webview 的 Rust facade 由 arkit 在 init 阶段自动注册
  // （`arkit/webview` feature），这里安装的 WebviewPlugin 只在 Rust 侧发起
  // bridge 调用时被使用；未启用 feature 的应用不受影响。
  public bridgePlugins = [new LazyPlugin(() => new WebviewPlugin())];

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
