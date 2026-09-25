import {
  AccessibilityElement,
  AccessibilityEvent,
  AccessibilityExtensionAbility,
  FocusDirection,
} from "@kit.AccessibilityKit";
import { hilog } from "@kit.PerformanceAnalysisKit";

const LOG_DOMAIN: number = 0x0000;
const LOG_TAG: string = "ArkitA11yE2E";

/**
 * Small screen-reader-style service used only by the QEMU acceptance suite.
 *
 * The public OpenHarmony QEMU image exposes AccessibilityManagerService but
 * does not bundle a user-facing screen reader. This extension drives the same
 * focus and performAction path without replacing or patching the image.
 */
export default class AccessibilityE2ETestExtension extends AccessibilityExtensionAbility {
  private focusedElement: AccessibilityElement | undefined = undefined;

  onConnect(): void {
    hilog.info(LOG_DOMAIN, LOG_TAG, "connected");
  }

  onDisconnect(): void {
    hilog.info(LOG_DOMAIN, LOG_TAG, "disconnected");
    this.focusedElement = undefined;
  }

  onAccessibilityEvent(event: AccessibilityEvent): void {
    const eventType: string = `${event.eventType}`;
    hilog.info(LOG_DOMAIN, LOG_TAG, "event=%{public}s", eventType);
    switch (eventType) {
      case "right":
        this.moveFocus("forward");
        break;
      case "left":
        this.moveFocus("backward");
        break;
      case "down":
        this.performFocusedAction("click");
        break;
      case "up":
        this.performFocusedAction("longClick");
        break;
      case "hoverEnter":
        if (event.target !== undefined) {
          this.focusHoveredTarget(event.target);
        }
        break;
      default:
        break;
    }
  }

  private moveFocus(direction: FocusDirection): void {
    this.moveFocusAsync(direction).catch((error: Error): void => {
      hilog.error(LOG_DOMAIN, LOG_TAG, "moveFocus failed: %{public}s", error.message);
    });
  }

  private async moveFocusAsync(direction: FocusDirection): Promise<void> {
    let current: AccessibilityElement;
    try {
      current = await this.context.getFocusElement(true);
    } catch {
      current = await this.context.getWindowRootElement();
    }
    const next: AccessibilityElement = await current.findElement("focusDirection", direction);
    await next.performAction("accessibilityFocus");
    this.focusedElement = next;
  }

  private focusHoveredTarget(target: AccessibilityElement): void {
    this.focusHoveredTargetAsync(target).catch((error: Error): void => {
      hilog.error(LOG_DOMAIN, LOG_TAG, "focusHoveredTarget failed: %{public}s", error.message);
    });
  }

  private async focusHoveredTargetAsync(target: AccessibilityElement): Promise<void> {
    const actionable: AccessibilityElement | undefined = await this.findActionable(target, 0);
    if (actionable === undefined) {
      throw new Error("hovered accessibility subtree has no actionable element");
    }
    await actionable.performAction("accessibilityFocus");
    this.focusedElement = actionable;
    const actions: string[] = await actionable.actionNames();
    let accessibilityText: string = "";
    try {
      accessibilityText = await actionable.attributeValue("accessibilityText");
    } catch {}
    hilog.info(
      LOG_DOMAIN,
      LOG_TAG,
      "focused a11y=%{public}s actions=%{public}s",
      accessibilityText,
      actions.join(","),
    );
  }

  private async findActionable(
    element: AccessibilityElement,
    depth: number,
  ): Promise<AccessibilityElement | undefined> {
    if (depth > 8) {
      return undefined;
    }
    try {
      const actions: string[] = await element.actionNames();
      if (actions.includes("click") || actions.includes("longClick")) {
        return element;
      }
    } catch {}

    let children: AccessibilityElement[];
    try {
      children = await element.attributeValue("children");
    } catch {
      return undefined;
    }
    for (const child of children) {
      const target: AccessibilityElement | undefined = await this.findActionable(child, depth + 1);
      if (target !== undefined) {
        return target;
      }
    }
    return undefined;
  }

  private performFocusedAction(action: "click" | "longClick"): void {
    this.performFocusedActionAsync(action).catch((error: Error): void => {
      hilog.error(LOG_DOMAIN, LOG_TAG, "%{public}s failed: %{public}s", action, error.message);
    });
  }

  private async performFocusedActionAsync(action: "click" | "longClick"): Promise<void> {
    let element: AccessibilityElement;
    try {
      element = await this.context.getFocusElement(true);
    } catch {
      if (this.focusedElement === undefined) {
        throw new Error("no accessibility-focused element");
      }
      element = this.focusedElement;
    }
    const actions: string[] = await element.actionNames();
    if (!actions.includes(action)) {
      throw new Error(`focused element does not support ${action}`);
    }
    await element.performAction(action);
    hilog.info(LOG_DOMAIN, LOG_TAG, "performed=%{public}s", action);
  }
}
