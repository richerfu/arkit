import { NativeAbility } from "@ohos-rs/ability";
import Want from "@ohos.app.ability.Want";
import { AbilityConstant } from "@kit.AbilityKit";
import window from "@ohos.window";

// moduleName 由 run.sh 按当前部署的 example 写入（对应 lib<crate>.so）。
// NativeAbility 据此 Loadable.load 加载并调 init/render/destroy。
export default class EntryAbility extends NativeAbility {
  public moduleName: string = "chart_example";
  public defaultPage: boolean = false;

  async onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): Promise<void> {
    super.onCreate(want, launchParam);
  }

  async onWindowStageCreate(windowStage: window.WindowStage): Promise<void> {
    const win = windowStage.getMainWindowSync();
    await win.setWindowLayoutFullScreen(false);
    super.onWindowStageCreate(windowStage);
    await windowStage.loadContent("pages/Index");
  }
}
