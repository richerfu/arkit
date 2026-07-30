import { DefaultXComponent } from "@ohos-rs/ability";

interface NativeBackPressModule {
  onBackPressIntercept?: () => boolean;
}

// moduleName 由 EntryAbility 写入 AppStorage（NativeAbility.onCreate 里
// AppStorage.setOrCreate("moduleName", this.moduleName)）。这里在 aboutToAppear
// 读出存入 @State，传给 DefaultXComponent——保持单一来源。
@Entry
@Component
struct Index {
  @State moduleName: string = "complex_cases";

  aboutToAppear(): void {
    const stored = AppStorage.get<string>("moduleName");
    if (stored) {
      this.moduleName = stored;
    }
  }

  onBackPress(): boolean {
    try {
      const nativeModule = loadNativeModule(`lib${this.moduleName}.so`) as NativeBackPressModule;
      return nativeModule.onBackPressIntercept?.() ?? false;
    } catch (error) {
      console.error(`arkit failed to dispatch back press: ${String(error)}`);
      return false;
    }
  }

  build() {
    Column() {
      DefaultXComponent({ moduleName: this.moduleName })
        .width("100%")
        .height("100%")
    }
    .width("100%")
    .height("100%")
  }
}
