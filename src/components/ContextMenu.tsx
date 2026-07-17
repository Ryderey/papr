import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type KeyboardEvent,
} from "react";
import Icon, { type IconName } from "./Icon";
import { clampToViewport, placeSubmenu } from "../lib/viewport";
import { useDismiss } from "../hooks/useDismiss";
import { useMenuKeyboard } from "../hooks/useMenuKeyboard";

export type MenuEntry =
  | {
      icon?: IconName;
      label: string;
      shortcut?: string;
      danger?: boolean;
      onClick: () => void;
    }
  | { separator: true }
  | {
      /** A row of colour swatches — for picking a tag colour. */
      swatches: { value: string; color: string }[];
      current: string;
      onPick: (value: string) => void;
    }
  | {
      icon?: IconName;
      label: string;
      submenu: MenuEntry[];
    };

interface Props {
  x: number;
  y: number;
  items: MenuEntry[];
  onClose: () => void;
}

/** Floating context menu, clamped inside the viewport — design `.ctx-menu`. */
export default function ContextMenu({ x, y, items, onClose }: Props) {
  const rootRef = useRef<HTMLDivElement>(null);
  const ref = useRef<HTMLDivElement>(null);
  const submenuRef = useRef<HTMLDivElement>(null);
  const closeTimer = useRef<number | null>(null);
  const [pos, setPos] = useState({ left: x, top: y });
  const [openSubmenu, setOpenSubmenu] = useState<number | null>(null);
  const [focusSubmenu, setFocusSubmenu] = useState(false);
  const [submenuPos, setSubmenuPos] = useState({ left: x, top: y });

  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    // The menu is anchored at the mouse cursor, so it can open anywhere. The
    // shared clamp pulls it back from the right/bottom edges and floors it at
    // the 8px margin, so a menu taller/wider than the window stays reachable.
    setPos(clampToViewport({ x, y, width: r.width, height: r.height, margin: 8 }));
  }, [x, y]);

  useLayoutEffect(() => {
    if (openSubmenu == null) return;
    const trigger = ref.current?.querySelector<HTMLElement>(
      `[data-submenu-index="${openSubmenu}"]`,
    );
    const submenu = submenuRef.current;
    if (!trigger || !submenu) return;
    const anchor = trigger.getBoundingClientRect();
    const size = submenu.getBoundingClientRect();
    setSubmenuPos(
      placeSubmenu({
        anchorLeft: anchor.left,
        anchorRight: anchor.right,
        anchorTop: anchor.top,
        width: size.width,
        height: size.height,
      }),
    );
    if (focusSubmenu) {
      submenu.querySelector<HTMLElement>('[role="menuitem"]')?.focus();
      setFocusSubmenu(false);
    }
  }, [openSubmenu, focusSubmenu]);

  useEffect(
    () => () => {
      if (closeTimer.current != null) window.clearTimeout(closeTimer.current);
    },
    [],
  );

  useDismiss(rootRef, onClose);

  // Focus management on open/close plus Arrow/Home/End/Enter navigation,
  // shared with the other role="menu" popovers.
  const onKeyDown = useMenuKeyboard(ref);
  const onSubmenuKeyDown = useMenuKeyboard(submenuRef, false, false);

  const cancelClose = () => {
    if (closeTimer.current != null) {
      window.clearTimeout(closeTimer.current);
      closeTimer.current = null;
    }
  };
  const closeSubmenu = () => {
    setOpenSubmenu(null);
    setFocusSubmenu(false);
  };
  const scheduleClose = () => {
    cancelClose();
    closeTimer.current = window.setTimeout(closeSubmenu, 120);
  };
  const openAt = (index: number, focus: boolean) => {
    cancelClose();
    setOpenSubmenu(index);
    setFocusSubmenu(focus);
  };

  const handleMainKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "ArrowRight") {
      const index = (document.activeElement as HTMLElement | null)?.dataset
        .submenuIndex;
      if (index != null) {
        e.preventDefault();
        openAt(Number(index), true);
        return;
      }
    }
    onKeyDown(e);
  };

  const handleSubmenuKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      const index = openSubmenu;
      closeSubmenu();
      ref.current
        ?.querySelector<HTMLElement>(`[data-submenu-index="${index}"]`)
        ?.focus();
      return;
    }
    onSubmenuKeyDown(e);
  };

  const renderEntry = (it: MenuEntry, i: number, nested = false) => {
    if ("separator" in it)
      return <div key={i} className="ctx-sep" role="separator" />;
    if ("swatches" in it)
      return (
        <div
          key={i}
          className="ctx-swatches"
          onMouseEnter={nested ? undefined : closeSubmenu}
        >
          {it.swatches.map((sw) => (
            <button
              key={sw.value}
              className={`ctx-swatch ${sw.value === it.current ? "on" : ""}`}
              role="menuitem"
              tabIndex={-1}
              style={{ background: sw.color }}
              aria-label={sw.value}
              aria-pressed={sw.value === it.current}
              onClick={() => {
                it.onPick(sw.value);
                onClose();
              }}
            />
          ))}
        </div>
      );
    if ("submenu" in it) {
      if (nested) return null;
      return (
        <div
          key={i}
          className="ctx-item ctx-item-submenu"
          role="menuitem"
          tabIndex={-1}
          aria-haspopup="menu"
          aria-expanded={openSubmenu === i}
          data-submenu-index={i}
          onMouseEnter={() => openAt(i, false)}
          onClick={() =>
            openSubmenu === i ? closeSubmenu() : openAt(i, false)
          }
        >
          <span className="ctx-ico">
            {it.icon && <Icon name={it.icon} size={13} />}
          </span>
          {it.label}
          <span className="ctx-submenu-arrow">
            <Icon name="chevron-right" size={12} />
          </span>
        </div>
      );
    }
    return (
      <div
        key={i}
        className="ctx-item"
        role="menuitem"
        tabIndex={-1}
        style={it.danger ? { color: "oklch(0.55 0.17 28)" } : undefined}
        onMouseEnter={nested ? undefined : closeSubmenu}
        onClick={() => {
          it.onClick();
          onClose();
        }}
      >
        <span className="ctx-ico">
          {it.icon && <Icon name={it.icon} size={13} />}
        </span>
        {it.label}
        {it.shortcut && <span className="ctx-shortcut">{it.shortcut}</span>}
      </div>
    );
  };

  const submenuEntry = openSubmenu == null ? null : items[openSubmenu];
  const submenuItems =
    submenuEntry && "submenu" in submenuEntry ? submenuEntry.submenu : null;

  return (
    <div
      className="ctx-menu-root"
      ref={rootRef}
      onMouseEnter={cancelClose}
      onMouseLeave={scheduleClose}
    >
      <div
        className="ctx-menu"
        ref={ref}
        role="menu"
        style={{ left: pos.left, top: pos.top }}
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleMainKeyDown}
      >
        {items.map((it, i) => renderEntry(it, i))}
      </div>
      {submenuItems && (
        <div
          className="ctx-menu ctx-submenu"
          ref={submenuRef}
          role="menu"
          style={{ left: submenuPos.left, top: submenuPos.top }}
          onClick={(e) => e.stopPropagation()}
          onKeyDown={handleSubmenuKeyDown}
        >
          {submenuItems.map((it, i) => renderEntry(it, i, true))}
        </div>
      )}
    </div>
  );
}
