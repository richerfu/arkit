# shadcn_showcase

路径：`examples/shadcn_showcase`

展示 `arkit::shadcn` facade 下的 Dioxus 组件、主题预设、controlled/uncontrolled state、overlay 与交互事件。showcase 当前集中在 `src/lib.rs`，便于作为单一设备验收入口。

## 当前交互契约

- Tabs、页面切换和其他 subtree replacement 通过 runtime event queue 进入 Dioxus，原生 callback 不允许重入正在执行的 render。
- Menubar、DropdownMenu、ContextMenu 的 checkbox/radio 是受控 entry；菜单保持打开时必须立即刷新选择标记，submenu 展开状态不得被重置。
- ContextMenu 只由真实 `onlongpress` 触发。短按无响应，单指长按约 500ms 后打开一次。
- HoverCard 相对 trigger 使用 center anchor；卡片内容使用 start alignment。不要把“锚点居中”和“内容居中”混为同一布局属性。
- Grid/List 切换必须改变 native projection；`grid_column_template` 等 collection 属性必须由 renderer 编码，不能静默忽略。

## 构建与设备验收

```bash
cd examples/shadcn_showcase
ohrs build --arch aarch

cd ../..
app/run.sh shadcn_showcase all
```

`ohrs build` 成功后才能打 HAP。一次只安装这一个 example，等待设备交互验收完成后再切换下一个。host `cargo check` 不是 OpenHarmony 构建验证。
