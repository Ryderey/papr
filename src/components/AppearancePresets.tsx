import type { CSSProperties } from "react";
import { useTranslation } from "react-i18next";
import { useUi, READER_FONTS } from "../store";
import { ACCENTS, APPEARANCE_PRESETS, matchingAppearancePreset } from "../lib/appearance";
import Icon from "./Icon";

export default function AppearancePresets() {
  const { t } = useTranslation();
  const appearance = useUi();
  const selected = matchingAppearancePreset(appearance);

  return (
    <section className="settings-group appearance-presets" aria-labelledby="appearance-presets-title">
      <div className="appearance-presets-heading">
        <h3 id="appearance-presets-title">{t("settings.appearance.presetsTitle")}</h3>
        <span className="appearance-current">
          {selected
            ? t("settings.appearance.presetCurrent", { name: t(`settings.appearance.presets.${selected}.name`) })
            : t("settings.appearance.presetCustom")}
        </span>
      </div>
      <p className="appearance-presets-intro">{t("settings.appearance.presetsDesc")}</p>
      <div className="appearance-preset-grid">
        {APPEARANCE_PRESETS.map(({ id, settings }) => {
          const palette = ACCENTS[settings.accent];
          const dark = settings.theme === "dark";
          const name = t(`settings.appearance.presets.${id}.name`);
          const description = t(`settings.appearance.presets.${id}.desc`);
          const style = {
            "--accent": dark ? palette.dAccent : palette.accent,
            "--accent-soft": dark ? palette.dSoft : palette.soft,
            "--accent-ink": dark ? palette.dInk : palette.ink,
            "--preview-font": READER_FONTS[settings.readerFont].stack,
            "--preview-size": `${settings.readerSize / 2}px`,
            "--preview-leading": String(settings.readerLeading / 100),
          } as CSSProperties;

          return (
            <button
              key={id}
              type="button"
              className={`appearance-preset ${selected === id ? "on" : ""}`}
              aria-pressed={selected === id}
              aria-label={t("settings.appearance.presetApply", { name })}
              aria-describedby={`appearance-preset-${id}-desc`}
              onClick={() => appearance.applyAppearancePreset(id)}
            >
              <span
                className="appearance-preview"
                data-theme={settings.theme}
                data-dark-shade={settings.darkShade}
                data-density={settings.density}
                data-view-mode={settings.viewMode}
                style={style}
                aria-hidden="true"
              >
                <span className="appearance-preview-sidebar">
                  <span className="appearance-preview-brand">Papr</span>
                  <span className="appearance-preview-nav active" />
                  <span className="appearance-preview-nav" />
                  <span className="appearance-preview-nav short" />
                </span>
                <span className="appearance-preview-list">
                  <span className="appearance-preview-list-heading" />
                  <span className="appearance-preview-item active"><i /><i /></span>
                  <span className="appearance-preview-item"><i /><i /></span>
                  <span className="appearance-preview-item"><i /><i /></span>
                </span>
                <span className="appearance-preview-reader">
                  <span className="appearance-preview-kicker">{t("settings.appearance.previewKicker")}</span>
                  <span className="appearance-preview-title">{t("settings.appearance.previewTitle")}</span>
                  <span className="appearance-preview-rule" />
                  <span className="appearance-preview-body">{t("settings.appearance.previewBody")}</span>
                </span>
                <span className="appearance-preview-mode">{t(`settings.appearance.${settings.theme}`)}</span>
              </span>
              <span className="appearance-preset-caption">
                <span className="appearance-preset-title">
                  {name}
                  {selected === id && <Icon name="check" size={14} />}
                </span>
                <span className="appearance-preset-desc" id={`appearance-preset-${id}-desc`}>{description}</span>
              </span>
            </button>
          );
        })}
      </div>
    </section>
  );
}
