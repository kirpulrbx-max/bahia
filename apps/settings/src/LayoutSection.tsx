import { useState } from "react";
import {
  type Config,
  type DockSection,
  type MenuBarItem,
  type MinimizeStyle,
  ICON_SIZE,
  LAYOUT_RANGES,
  MAX_MAGNIFIED_SIZE,
} from "./config";
import { Card, Row, Segmented, Slider, SubRow, Toggle } from "./controls";
import type { Texts } from "./i18n";

/** Defaults, as `bahia_config::Config::default` has them. */
export const MENU_BAR_ORDER: readonly MenuBarItem[] = [
  "widgets", "taskView", "hiddenIcons", "inputLanguage", "accessories", "battery",
  "volume", "network", "bluetooth", "search", "privacy", "controlCenter", "clock",
];
export const DOCK_ORDER: readonly DockSection[] = [
  "start", "apps", "recents", "minimized", "shortcuts", "downloads", "trash",
];

/** Keeps the saved order and adds what it lacks (an item a newer version added). */
export function completeOrder<T>(saved: readonly T[] | undefined, all: readonly T[]): T[] {
  const order = (saved ?? []).filter((item, index, list) => all.includes(item) && list.indexOf(item) === index);
  all.forEach((item, index) => {
    if (order.includes(item)) return;
    const before = all.slice(0, index).reverse().find((other) => order.includes(other));
    order.splice(before === undefined ? 0 : order.indexOf(before) + 1, 0, item);
  });
  return order;
}

/** Moves the item at `from` to `to`, shifting the rest. */
export function moveItem<T>(list: readonly T[], from: number, to: number): T[] {
  const next = [...list];
  const [item] = next.splice(from, 1);
  next.splice(Math.max(0, Math.min(to, next.length)), 0, item);
  return next;
}

interface OrderListProps<T extends string> {
  label: string;
  items: readonly T[];
  name: (item: T) => string;
  /** Undefined for items that cannot be hidden. */
  shown: (item: T) => boolean | undefined;
  onToggle: (item: T, shown: boolean) => void;
  onReorder: (items: T[]) => void;
  t: Texts;
}

/**
 * A list the user arranges by dragging rows or with the arrow buttons
 * (keyboard and screen readers), with a switch for each item that can hide.
 */
function OrderList<T extends string>({ label, items, name, shown, onToggle, onReorder, t }: OrderListProps<T>) {
  const [dragging, setDragging] = useState<number | null>(null);
  const [over, setOver] = useState<number | null>(null);
  return (
    <ol className="order-list" aria-label={label}>
      {items.map((item, index) => {
        const visible = shown(item);
        return (
          <li
            key={item}
            className="order-row"
            data-dragging={dragging === index || undefined}
            data-over={over === index && dragging !== index ? (dragging !== null && dragging < index ? "after" : "before") : undefined}
            data-hidden={visible === false || undefined}
            draggable
            onDragStart={(event) => {
              setDragging(index);
              event.dataTransfer.effectAllowed = "move";
              event.dataTransfer.setData("text/plain", item);
            }}
            onDragOver={(event) => {
              if (dragging === null) return;
              event.preventDefault();
              event.dataTransfer.dropEffect = "move";
              if (over !== index) setOver(index);
            }}
            onDrop={(event) => {
              event.preventDefault();
              if (dragging !== null && dragging !== index) onReorder(moveItem(items, dragging, index));
              setDragging(null);
              setOver(null);
            }}
            onDragEnd={() => { setDragging(null); setOver(null); }}
          >
            <span className="order-grip" aria-hidden="true">
              <svg viewBox="0 0 12 16" width="10" height="14"><g fill="currentColor">
                <circle cx="3.5" cy="3" r="1.3" /><circle cx="8.5" cy="3" r="1.3" />
                <circle cx="3.5" cy="8" r="1.3" /><circle cx="8.5" cy="8" r="1.3" />
                <circle cx="3.5" cy="13" r="1.3" /><circle cx="8.5" cy="13" r="1.3" />
              </g></svg>
            </span>
            <span className="order-name">{name(item)}</span>
            <span className="order-buttons">
              <button type="button" className="order-move" aria-label={`${t.layMoveUp}: ${name(item)}`}
                disabled={index === 0} onClick={() => onReorder(moveItem(items, index, index - 1))}>
                <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true"><path d="m4 10 4-4 4 4" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" /></svg>
              </button>
              <button type="button" className="order-move" aria-label={`${t.layMoveDown}: ${name(item)}`}
                disabled={index === items.length - 1} onClick={() => onReorder(moveItem(items, index, index + 1))}>
                <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true"><path d="m4 6 4 4 4-4" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" /></svg>
              </button>
            </span>
            {visible === undefined
              ? <span className="order-toggle-spacer" />
              : <Toggle label={`${t.layShow}: ${name(item)}`} checked={visible} onChange={(on) => onToggle(item, on)} />}
          </li>
        );
      })}
    </ol>
  );
}

function menuBarName(t: Texts, item: MenuBarItem): string {
  switch (item) {
    case "widgets": return t.layMbWidgets;
    case "taskView": return t.layMbTaskView;
    case "hiddenIcons": return t.layMbHiddenIcons;
    case "inputLanguage": return t.layMbInputLanguage;
    case "accessories": return t.layMbAccessories;
    case "battery": return t.layMbBattery;
    case "volume": return t.layMbVolume;
    case "network": return t.layMbNetwork;
    case "bluetooth": return t.layMbBluetooth;
    case "search": return t.layMbSearch;
    case "privacy": return t.layMbPrivacy;
    case "controlCenter": return t.layMbControlCenter;
    case "clock": return t.layMbClock;
  }
}

function dockName(t: Texts, section: DockSection): string {
  switch (section) {
    case "start": return t.layDockStart;
    case "apps": return t.layDockApps;
    case "recents": return t.layDockRecents;
    case "minimized": return t.layDockMinimized;
    case "shortcuts": return t.layDockShortcuts;
    case "downloads": return t.layDockDownloads;
    case "trash": return t.layDockTrash;
  }
}

/** The existing switch behind each dock section that can hide. */
const DOCK_SWITCH: Partial<Record<DockSection, keyof Config>> = {
  start: "showStart",
  recents: "showRecents",
  minimized: "showMinimized",
  downloads: "showDownloads",
  trash: "showTrash",
};

/** The density presets' spacing, as `DockDensity::spacing` has it. */
const DENSITY_SPACING = { compact: [8, 6], normal: [12, 9], spacious: [16, 12] } as const;
/** Each preset's length, so a slider starts where the preset was. */
const GENIE_SPEED_MS = { normal: 500, fast: 350, faster: 250 } as const;
const SLIDE_SPEED_MS = { smooth: 375, normal: 250, fast: 167 } as const;

interface LayoutSectionProps {
  config: Config;
  t: Texts;
  update: (patch: Partial<Config>) => void;
  /** Windows' own animation switch is off: Bahia does not animate either. */
  animationsOff: boolean;
}

export function LayoutSection({ config, t, update, animationsOff }: LayoutSectionProps) {
  const menuBarOrder = completeOrder(config.menuBarOrder, MENU_BAR_ORDER);
  const menuBarHidden = config.menuBarHidden ?? [];
  const dockOrder = completeOrder(config.dockOrder, DOCK_ORDER);
  const [presetGap, presetPadding] = DENSITY_SPACING[config.dockDensity ?? "normal"];
  const ms = (value: number) => `${value}`;

  return (
    <>
      {animationsOff ? <p className="card-note row-hint layout-notice">{t.layAnimationsOff}</p> : null}

      <Card title={t.layMenuBarOrder}>
        <p className="card-note row-hint">{t.layMenuBarOrderHint}</p>
        <OrderList<MenuBarItem>
          label={t.layMenuBarOrder}
          items={menuBarOrder}
          name={(item) => menuBarName(t, item)}
          shown={(item) => !menuBarHidden.includes(item)}
          onToggle={(item, on) => update({
            menuBarHidden: on ? menuBarHidden.filter((hidden) => hidden !== item) : [...menuBarHidden, item],
          })}
          onReorder={(menuBarOrder) => update({ menuBarOrder })}
          t={t}
        />
      </Card>

      <Card title={t.layDockOrder}>
        <p className="card-note row-hint">{t.layDockOrderHint}</p>
        <OrderList<DockSection>
          label={t.layDockOrder}
          items={dockOrder}
          name={(section) => dockName(t, section)}
          shown={(section) => {
            const key = DOCK_SWITCH[section];
            return key ? (config[key] as boolean | undefined) ?? true : undefined;
          }}
          onToggle={(section, on) => {
            const key = DOCK_SWITCH[section];
            if (key) update({ [key]: on } as Partial<Config>);
          }}
          onReorder={(dockOrder) => update({ dockOrder })}
          t={t}
        />
      </Card>

      <Card title={t.laySizes}>
        <Row label={t.size}>
          <Slider label={t.size} value={config.iconSize} min={ICON_SIZE.min} max={ICON_SIZE.max}
            onChange={(iconSize) => update({ iconSize })} />
        </Row>
        <Row label={t.layGapAuto} hint={t.layAutoHint}>
          <Toggle label={t.layGapAuto} checked={config.dockGap === null}
            onChange={(auto) => update({ dockGap: auto ? null : presetGap })} />
        </Row>
        <SubRow label={t.layGap} open={config.dockGap !== null}>
          <Slider label={t.layGap} value={config.dockGap ?? presetGap}
            min={LAYOUT_RANGES.dockGap.min} max={LAYOUT_RANGES.dockGap.max}
            onChange={(dockGap) => update({ dockGap })} />
        </SubRow>
        <Row label={t.layPaddingAuto}>
          <Toggle label={t.layPaddingAuto} checked={config.dockPadding === null}
            onChange={(auto) => update({ dockPadding: auto ? null : presetPadding })} />
        </Row>
        <SubRow label={t.layPadding} open={config.dockPadding !== null}>
          <Slider label={t.layPadding} value={config.dockPadding ?? presetPadding}
            min={LAYOUT_RANGES.dockPadding.min} max={LAYOUT_RANGES.dockPadding.max}
            onChange={(dockPadding) => update({ dockPadding })} />
        </SubRow>
      </Card>

      <Card title={t.layMinimize}>
        <Row label={t.layMinimizeAnimate}>
          <Toggle label={t.layMinimizeAnimate} checked={config.genie} onChange={(genie) => update({ genie })} />
        </Row>
        <SubRow label={t.layStyle} open={config.genie}>
          <Segmented<MinimizeStyle>
            label={t.layStyle}
            value={config.minimizeStyle ?? "genie"}
            options={[
              { value: "genie", label: t.layStyleGenie },
              { value: "scale", label: t.layStyleScale },
            ]}
            onChange={(minimizeStyle) => update({ minimizeStyle })}
          />
        </SubRow>
        <SubRow label={t.layDuration} open={config.genie}>
          <Slider label={t.layDuration} unit="ms" format={ms}
            value={config.genieMs ?? GENIE_SPEED_MS[config.genieSpeed ?? "fast"]}
            min={LAYOUT_RANGES.genieMs.min} max={LAYOUT_RANGES.genieMs.max} step={10}
            onChange={(genieMs) => update({ genieMs })} />
        </SubRow>
      </Card>

      <Card title={t.magnification}>
        <Row label={t.magnification}>
          <Toggle label={t.magnification} checked={config.magnification}
            onChange={(magnification) => update({ magnification })} />
        </Row>
        <SubRow label={t.magnifiedSize} open={config.magnification}>
          <Slider label={t.magnifiedSize} value={config.magnifiedSize} min={config.iconSize} max={MAX_MAGNIFIED_SIZE}
            onChange={(magnifiedSize) => update({ magnifiedSize })} />
        </SubRow>
        <SubRow label={t.layMagnifyRange} hint={t.layMagnifyRangeHint} open={config.magnification}>
          <Slider label={t.layMagnifyRange} unit={t.layIcons} format={(value) => (value / 10).toFixed(1)}
            value={config.magnifyRange ?? 30}
            min={LAYOUT_RANGES.magnifyRange.min} max={LAYOUT_RANGES.magnifyRange.max}
            onChange={(magnifyRange) => update({ magnifyRange })} />
        </SubRow>
      </Card>

      <Card title={t.layBounce}>
        <Row label={t.bounceOnOpen}>
          <Toggle label={t.bounceOnOpen} checked={config.bounceOnOpen}
            onChange={(bounceOnOpen) => update({ bounceOnOpen })} />
        </Row>
        <Row label={t.layBounceAttention}>
          <Toggle label={t.layBounceAttention} checked={config.bounceOnAttention ?? true}
            onChange={(bounceOnAttention) => update({ bounceOnAttention })} />
        </Row>
        <Row label={t.layBounceHeight}>
          <Slider label={t.layBounceHeight} unit="%" value={config.bounceHeight ?? 50}
            min={LAYOUT_RANGES.bounceHeight.min} max={LAYOUT_RANGES.bounceHeight.max}
            onChange={(bounceHeight) => update({ bounceHeight })} />
        </Row>
        <Row label={t.layBounceDuration}>
          <Slider label={t.layBounceDuration} unit="ms" format={ms} value={config.bounceMs ?? 700}
            min={LAYOUT_RANGES.bounceMs.min} max={LAYOUT_RANGES.bounceMs.max} step={10}
            onChange={(bounceMs) => update({ bounceMs })} />
        </Row>
      </Card>

      <Card title={t.layDockSlide}>
        <Row label={t.layDockSlideAuto} hint={t.layDockSlideHint}>
          <Toggle label={t.layDockSlideAuto} checked={config.dockSlideMs === null}
            onChange={(auto) => update({
              dockSlideMs: auto ? null : SLIDE_SPEED_MS[config.dockAutoHideSpeed ?? "normal"],
            })} />
        </Row>
        <SubRow label={t.layDuration} open={config.dockSlideMs !== null}>
          <Slider label={t.layDuration} unit="ms" format={ms}
            value={config.dockSlideMs ?? SLIDE_SPEED_MS[config.dockAutoHideSpeed ?? "normal"]}
            min={LAYOUT_RANGES.dockSlideMs.min} max={LAYOUT_RANGES.dockSlideMs.max} step={10}
            onChange={(dockSlideMs) => update({ dockSlideMs })} />
        </SubRow>
      </Card>

      <Card>
        <Row label={t.layReset} hint={t.layResetHint}>
          <button type="button" className="push" onClick={() => update({
            menuBarOrder: [...MENU_BAR_ORDER],
            menuBarHidden: [],
            dockOrder: [...DOCK_ORDER],
            dockGap: null,
            dockPadding: null,
            minimizeStyle: "genie",
            genieMs: null,
            magnifyRange: 30,
            bounceHeight: 50,
            bounceMs: 700,
            bounceOnAttention: true,
            dockSlideMs: null,
          })}>{t.resetButton}</button>
        </Row>
      </Card>
    </>
  );
}
