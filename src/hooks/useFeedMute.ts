import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import * as api from "../api";
import type { Feed } from "../types";
import type { MenuEntry } from "../components/ContextMenu";
import { isFeedMuted, MUTE_DURATIONS } from "../lib/feedMute";
import { reportError, toast, toastAction } from "../toast";

export function useFeedMute() {
  const qc = useQueryClient();
  const { t } = useTranslation();

  const setMuted = useCallback(
    async function setMuted(feed: Feed, minutes: number | null) {
      try {
        await api.setFeedMuted(feed.id, minutes);
        qc.invalidateQueries({ queryKey: ["feeds"] });
        qc.invalidateQueries({ queryKey: ["articles"] });
        qc.invalidateQueries({ queryKey: ["counts"] });

        if (minutes == null) {
          toast.show(t("feedMute.toastUnmuted", { feed: feed.title }));
          return;
        }
        const durationKey = MUTE_DURATIONS.find(
          (duration) => duration.minutes === minutes,
        )?.labelKey;
        toastAction(
          t("feedMute.toastMuted", {
            feed: feed.title,
            duration: durationKey ? t(durationKey) : String(minutes),
          }),
          {
            label: t("common.undo"),
            run: () => {
              void setMuted(feed, null);
            },
          },
        );
      } catch (e) {
        reportError(e);
      }
    },
    [qc, t],
  );

  const menuItems = useCallback(
    (feed: Feed): MenuEntry[] => {
      const isMuted = isFeedMuted(feed.mutedUntil, Math.floor(Date.now() / 1000));
      const durations: MenuEntry[] = MUTE_DURATIONS.map((duration) => ({
        label: t(duration.labelKey),
        onClick: () => {
          void setMuted(feed, duration.minutes);
        },
      }));
      if (isMuted) {
        return [
          {
            icon: "eye",
            label: t("feedMute.unmute"),
            onClick: () => {
              void setMuted(feed, null);
            },
          },
          {
            icon: "eye-off",
            label: t("feedMute.changeDuration"),
            submenu: durations,
          },
        ];
      }
      return [
        {
          icon: "eye-off",
          label: t("feedMute.action"),
          submenu: durations,
        },
      ];
    },
    [setMuted, t],
  );

  return { setMuted, menuItems };
}
