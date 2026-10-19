# Troubleshooting Common Issues (Development)

Every issue listed below occurred during actual development of this project — organized by symptom.

## Start by Checking These 4 Commands

```bash
make status                                   # Cluster pods should be Running (ignoring old Completed/Error pods)
df -h /                                       # Disk usage over ~85% → see "Disk full" below
ss -ltnp | grep -E ':(8000|8001|8002|3000) '  # Check which local services are listening on their expected ports
kubectl -n arch exec deploy/arch-cache -- valkey-cli --scan --pattern 'arch:*' | head   # Inspect Valkey cache keys
```

## Cluster & Infrastructure

| Symptom | Cause | Fix |
|---|---|---|
| `make master` fails with `"Connection reset by peer"` / all pods evicted | **Disk full** → kubelet taints the node with `DiskPressure` and evicts all pods | Free disk space below ~85% and wait for pods to recover (`kubectl describe node \| grep -i pressure`). Common culprits: Cargo `target/` (tens of GBs — point `CARGO_TARGET_DIR` to `~/.cache/…` and clean periodically), `~/.local/share/containers`, unused Docker volumes |
| After host reboot or network change: `arch-identity` stuck in `Init:Error` / `Unknown`, Kratos fails (`lookup pgedge … connection refused`) | k3s CoreDNS failed on initial startup (`Unknown`) → cluster DNS resolution is broken | `kubectl -n kube-system delete pod -l k8s-app=kube-dns` followed by `kubectl -n arch delete pod -l app.kubernetes.io/name=arch-identity` |
| `arch-streaming-console` in `CrashLoopBackOff` during startup | Console pod is waiting for Redpanda to become ready | Wait a moment; it resolves automatically once Redpanda starts |
| Old `Completed` / `Error` pods cluttering `make status` | Superseded pods from earlier jobs/rollouts | `kubectl -n arch delete pod --field-selector=status.phase=Succeeded` (and `Failed`) |
| Report files missing (preview returns `503`/`410 REPORT_FILE_GONE`), topics missing, or Quickwit indexes missing | Earlier versions used `emptyDir` for Garage/Redpanda/Quickwit (now migrated to PVCs so data persists across restarts) | If running on an older cluster created before PVCs: run `make deploy` to switch to PVCs, then `make topics` |
| `make seed` stops with `"database was loaded from an older sql/01_ysql.sql"` | Database schema is older than `sql/01_ysql.sql` (files `00`–`50` only load once on an empty DB) | See [`sql/README.md`](../../sql/README.md): write an idempotent migration script, or recreate the database via `make down && make bootstrap` (**deletes all data**) |
| `psql` is not installed on the host | `psql` is available inside the database pod | `kubectl -n arch exec -it arch-database-0 -- psql -U postgres -d arch` (then run `set search_path=arch;`) |

## Sign-In & Permissions

| Symptom | Cause | Fix |
|---|---|---|
| Sign-in returns **HTTP 500** (or Kratos logs `login-guard … connection refused`) | Kratos calls the `login-guard` webhook on `api-master` (`:8000`) before issuing a session | Start `api-master` (`make master`) before signing in |
| Sign-in returns **HTTP 500** and `api-master` logs `login guard secret not configured` (`503`) | `KRATOS_WEBHOOK_SECRET` is missing from `.env` (the webhook fails closed when unconfigured) | Set `KRATOS_WEBHOOK_SECRET` in `.env` to match `X-Webhook-Secret` in Kratos' ConfigMap (see `.env.example`) |
| Signing in via `curl` returns **403 CSRF** | `csrf_token` contains `+ / =` characters that were not URL-encoded | Use `--data-urlencode "csrf_token=$CSRF"` |
| Sidebar menu or action buttons do not reflect newly updated permissions | Permissions and menus are cached for 60 seconds (changes made through the web UI invalidate the cache automatically) | If permissions were modified directly via SQL: run `make cache-flush` |
| User account sees an empty sidebar menu | User's permission group has no `READ` grants on any program, the group is inactive, or `su_user.active=false` | Check `SUPG003` / `SUPG004` / `SUPG005` |
| API returns `401 Missing 'ory_kratos_session' cookie` | Session cookie was not included in the request | Sign in via the web UI or pass `-b jar` when testing with `curl` |

## Web Frontend (Trunk / Leptos)

| Symptom | Cause | Fix |
|---|---|---|
| Code changes do not appear in the browser | Debug WASM binary is large and still compiling, or browser cached the old bundle | Wait until Trunk prints `"success"`, then hard reload (`Ctrl+Shift+R`) |
| Page reloads repeatedly on its own | Trunk live-reload triggering as builds finish | Normal during active compilation |
| API calls from the web UI return `404`/`502` | Misordered `[[proxy]]` rules or target backend service is not running (`Trunk.toml` is generated from `.env` during `make web` — re-run `make web` after editing `.env`) | Ensure longer prefixes appear first (`/api/v1/notifications/` → `8002`, `/api/v1/reports/` → `8001`, `/api/v1/` → `8000`) and check listening ports with `ss` |
| Report download fails with `"Invalid signature"` | Proxy followed a redirect to a pre-signed S3 URL and altered the `Host` header | The API returns JSON `{url}` for the browser to open directly — if accessing the web UI from another machine while `AWS_ENDPOINT_URL` is `localhost:3900`, update `AWS_ENDPOINT_URL` to a host reachable by the browser |
| `bg-primary/10` or `border-border/60` has no visible background/border color | Tailwind 3 cannot apply `/alpha` modifiers to CSS custom properties (`var(--…)`) | Use `bg-[color-mix(in_oklch,var(--primary)_12%,transparent)]` |
| Newly added Tailwind class has no effect | Tailwind scans `.rs` files statically at build time — classes constructed dynamically via `format!` are not detected | Write full class name literals or add them to `safelist` in `tailwind.config.js`; wait for Trunk build to finish and hard reload |
| First `make web` run is slow or fails downloading Tailwind | Trunk downloads Tailwind CLI `3.4.17` from GitHub once (cached in `~/.cache/trunk`) | If working offline, place a `tailwindcss` `3.4.17` binary in your `PATH` |
| Dropdown does not close when clicking outside | A `fixed inset-0` backdrop inside the topbar (which uses `backdrop-blur`) only covers the topbar area | Use `web_package::ui::component::dismiss_on_outside` |
| WASM build fails due to missing `web-sys` method/type | Corresponding `web-sys` Cargo feature is not enabled | Add the feature to `Cargo.toml` in `handler/web/package` and/or `handler/web/application/master` |

## i18n / Reports / Notifications / Audit Log

| Symptom | Cause | Fix |
|---|---|---|
| UI text still appears in Thai when English is selected | Key is missing from `db_label` (falling back to the default string in `t_or`) | Add the key to `sql/NN_i18n_*.sql` and run `make seed` |
| Clicking "Export CSV" does nothing | `api-report` (`:8001`) is not running or topic `report.requests` does not exist | Run `make report` and `make topics` |
| Password setup email is not received | `api-notification` is not running, topic `mail-outbox` is missing, or `mail.smtp_url` is invalid (check `api-notification` logs) | Run `make notification`, `make topics`, and verify `su_configuration` |
| Notification bell does not update in real time | `api-notification` (`:8002`) is not running or topic `notifications` is missing | Run `make notification` and `make topics` |
| Report job stays stuck in `running` | Worker crashed mid-job | Open `ALPG001` (Process Monitor): jobs stuck for over 15 minutes can be reprocessed |
| `Ctrl+C` on `api-report` / `api-notification` used to hang | Open SSE streams blocked graceful shutdown | Fixed (closes streams + 10s watchdog); pressing `Ctrl+C` a second time exits immediately |
| `ALPG000` is empty or shows `"audit service down"` | Quickwit pod is down or index has not been created yet | Check `kubectl -n arch get pod -l app.kubernetes.io/name=arch-logging`; `audit-logs-v1` is auto-created on first event; for Redpanda event indexes run `make event-indexes` |
| `ALPG000` does not show `"Event: …"` filter results | Redpanda topics or Quickwit Kafka sources have not been created | Run `make topics` and `make event-indexes` |

## Valkey Cache

| Symptom | Cause | Fix |
|---|---|---|
| Recently updated data still shows old values | Data was modified directly via SQL/seed (bypassing API write invalidation) | Run `make cache-flush` (`make seed` runs this automatically) |
| Suspect stale cache is causing unexpected query results | — | Set `READ_CACHE_ENABLED=false` in `.env`, restart `api-master`, and compare behavior |
| Want to inspect cache hits/misses | — | Enable log target `arch::cache` (`RUST_LOG="…,arch::cache=debug"`); see [`11_VALKEY_READ_CACHE.md`](../architecture/11_VALKEY_READ_CACHE.md) |

## Useful Debugging Commands

- View pod logs: `make logs APP=arch-identity` (requires `stern`) or `kubectl -n arch logs deploy/arch-identity -c kratos`
- Inspect Redpanda topics: `kubectl -n arch exec deploy/arch-streaming -- rpk topic list` / `rpk topic consume <topic> -n 5`
- List Valkey cache keys: `kubectl -n arch exec deploy/arch-cache -- valkey-cli --scan --pattern 'arch:*'`
- Query Quickwit events directly: `curl 'localhost:7280/api/v1/events-notifications-v1/search?query=*'`
- Tip: if `pkill -f <pattern>` kills your own shell, the pattern matched your shell's command line — find the exact PID via `ss -ltnp` instead.
