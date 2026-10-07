import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import * as api from "../api";
import { errorText } from "../lib/errors";
import { useArticleActions } from "../hooks/articleActions";

function sameSchedule(a: api.GitHubSchedule, b: api.GitHubSchedule) {
  return a.enabled === b.enabled && a.upload_delay_secs === b.upload_delay_secs &&
    a.cloud_interval_minutes === b.cloud_interval_minutes &&
    a.background_interval_minutes === b.background_interval_minutes;
}

export default function GitHubSyncSection({ otherConnected, onToast }: { otherConnected: boolean; onToast: (message: string) => void }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const actions = useArticleActions();
  const status = useQuery({ queryKey: ["github-sync-status"], queryFn: api.githubStatus, refetchInterval: 10_000 });
  const schedule = useQuery({ queryKey: ["github-sync-schedule"], queryFn: api.githubSchedule });
  const [owner, setOwner] = useState("");
  const [repo, setRepo] = useState("");
  const [branch, setBranch] = useState("");
  const [token, setToken] = useState("");
  const [preview, setPreview] = useState<api.GitHubPreview | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [scheduleDraft, setScheduleDraft] = useState<api.GitHubSchedule | null>(null);
  const editedSchedule = scheduleDraft ?? schedule.data;
  const scheduleChanged = !!schedule.data && !!editedSchedule && !sameSchedule(editedSchedule, schedule.data);
  const connected = status.data?.profile;
  const run = async (operation: () => Promise<void>) => {
    setBusy(true); setError(null);
    try { await operation(); } catch (error) { setError(errorText(error)); }
    finally { setBusy(false); await queryClient.invalidateQueries({ queryKey: ["github-sync-status"] }); }
  };
  const sync = () => run(async () => { const report = await api.githubSyncNow(); if (!report.unchanged) { actions.refreshAfterBulk(); await queryClient.invalidateQueries({ queryKey: ["folders"] }); await queryClient.invalidateQueries({ queryKey: ["feeds"] }); } onToast(t("githubSync.completed")); });
  const updateSchedule = (next: api.GitHubSchedule) => run(async () => {
    await queryClient.cancelQueries({ queryKey: ["github-sync-schedule"] });
    await api.githubSetSchedule(next);
    const saved = await api.githubSchedule();
    // Reopening settings during the write may have started another old read.
    await queryClient.cancelQueries({ queryKey: ["github-sync-schedule"] });
    queryClient.setQueryData(["github-sync-schedule"], saved);
    if (!sameSchedule(saved, next)) throw new Error(t("githubSync.scheduleSaveFailed"));
    setScheduleDraft(null);
    onToast(t("githubSync.scheduleSaved"));
  });
  return <div className="settings-group">
    <h3 className="settings-group-title">GitHub</h3>
    <p className="modal-hint">{t("githubSync.scope")}</p>
    {status.isPending && <p>{t("githubSync.loading")}</p>}
    {(error || status.error) && <p role="alert">{error || errorText(status.error)}</p>}
    {status.data?.last_error_code && <p role="alert">{errorText({ code: status.data.last_error_code })}</p>}
    {connected ? <>
      <p>{connected.owner}/{connected.repo} · {connected.branch}</p>
      {schedule.error && <p role="alert">{errorText(schedule.error)}</p>}
      {editedSchedule && <div style={{ display: "flex", flexDirection: "column", gap: 8, marginBottom: 12 }}>
        <label><input type="checkbox" checked={editedSchedule.enabled} disabled={busy} onChange={(e) => setScheduleDraft({ ...editedSchedule, enabled: e.target.checked })} /> {t("githubSync.automatic")}</label>
        <label>{t("githubSync.uploadDelay")} <select className="modal-input" aria-label={t("githubSync.uploadDelay")} disabled={busy || !editedSchedule.enabled} value={editedSchedule.upload_delay_secs} onChange={(e) => setScheduleDraft({ ...editedSchedule, upload_delay_secs: Number(e.target.value) })}>
          {[10, 30, 60, 120].map((n) => <option key={n} value={n}>{t("githubSync.seconds", { count: n })}</option>)}
        </select></label>
        <label>{t("githubSync.cloudInterval")} <select className="modal-input" aria-label={t("githubSync.cloudInterval")} disabled={busy || !editedSchedule.enabled} value={editedSchedule.cloud_interval_minutes} onChange={(e) => setScheduleDraft({ ...editedSchedule, cloud_interval_minutes: Number(e.target.value) })}>
          {[5, 10, 15, 30, 60].map((n) => <option key={n} value={n}>{t("githubSync.minutes", { count: n })}</option>)}
        </select></label>
        <p className="modal-hint">{t(scheduleChanged ? "githubSync.scheduleUnsaved" : schedule.data?.enabled ? "githubSync.scheduleHint" : "githubSync.manualOnly")}</p>
        <div><button className="s-btn primary" disabled={busy || !scheduleChanged} onClick={() => void updateSchedule(editedSchedule)}>{t("common.save")}</button></div>
      </div>}
      <p>{t("githubSync.pending", { count: status.data?.pending ?? 0 })}</p>
      <p>{t("githubSync.lastSuccess", { time: status.data?.last_success_at || t("githubSync.never") })}</p>
      {!!status.data?.rejected && <p>{t("githubSync.rejected", { count: status.data.rejected })}</p>}
      {status.data?.uncertain_publication && <p>{t("githubSync.uncertain")}</p>}
      {status.data?.retry_at && <p>{t("githubSync.retryAt", { time: status.data.retry_at })}</p>}
      <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
        <button className="s-btn primary" disabled={busy || status.data?.busy} onClick={sync}>{t("settings.sync.syncNow")}</button>
        {(busy || status.data?.busy) && <button className="s-btn" onClick={() => void api.githubCancelSync().catch((e) => setError(errorText(e)))}>{t("githubSync.cancel")}</button>}
        <button className="s-btn" disabled={busy || status.data?.busy} onClick={() => void run(async () => { await api.githubDisconnect(); setToken(""); setScheduleDraft(null); onToast(t("settings.sync.disconnected")); })}>{t("settings.sync.disconnect")}</button>
      </div>
      <div style={{ display: "flex", gap: 8, marginTop: 12 }}>
        <input className="modal-input" type="password" autoComplete="off" aria-label={t("githubSync.token")} placeholder={t("githubSync.token")} value={token} onChange={(e) => setToken(e.target.value)} disabled={busy} />
        <button className="s-btn" disabled={busy || !token.trim()} onClick={() => void run(async () => { await api.githubUpdateCredential(token); setToken(""); onToast(t("githubSync.credentialUpdated")); })}>{t("githubSync.updateToken")}</button>
      </div>
    </> : otherConnected ? <p>{t("githubSync.otherBackend")}</p> : status.data && <>
      <p className="modal-hint">{t("githubSync.setup")}</p>
      <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
        <input className="modal-input" aria-label={t("githubSync.owner")} placeholder={t("githubSync.owner")} value={owner} disabled={busy || !!preview} onChange={(e) => setOwner(e.target.value)} />
        <input className="modal-input" aria-label={t("githubSync.repo")} placeholder={t("githubSync.repo")} value={repo} disabled={busy || !!preview} onChange={(e) => setRepo(e.target.value)} />
        <input className="modal-input" aria-label={t("githubSync.branch")} placeholder={t("githubSync.branch")} value={branch} disabled={busy || !!preview} onChange={(e) => setBranch(e.target.value)} />
        <input className="modal-input" type="password" autoComplete="off" aria-label={t("githubSync.token")} placeholder={t("githubSync.token")} value={token} disabled={busy || !!preview} onChange={(e) => setToken(e.target.value)} />
      </div>
      {preview ? <>
        <p>{t("githubSync.previewCounts", { localFeeds: preview.local_feeds, remoteFeeds: preview.remote_feeds, localArticles: preview.local_articles, remoteArticles: preview.remote_articles })}</p>
        <p>{t("githubSync.previewExcluded", { feeds: preview.excluded_feeds, articles: preview.excluded_articles, warnings: preview.warning_count })}</p>
        <p className="modal-hint">{t("githubSync.previewHint")}</p>
        <button className="s-btn primary" disabled={busy} onClick={() => void run(async () => { await api.githubConnect(preview, token); setToken(""); setPreview(null); onToast(t("settings.sync.connected")); })}>{t("githubSync.confirm")}</button>{" "}
        <button className="s-btn" disabled={busy} onClick={() => setPreview(null)}>{t("githubSync.cancel")}</button>
      </> : <button className="s-btn primary" style={{ marginTop: 12 }} disabled={busy || !owner.trim() || !repo.trim() || !token.trim()} onClick={() => void run(async () => setPreview(await api.githubPreview(owner.trim(), repo.trim(), branch, token)))}>{t("githubSync.preview")}</button>}
    </>}
  </div>;
}
