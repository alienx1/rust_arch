# แก้ปัญหาที่เจอบ่อย (dev)

ทุกข้อด้านล่างเกิดขึ้นจริงระหว่างพัฒนาโปรเจกต์นี้ — เรียงตามอาการที่เห็น

## เริ่มจากตรวจ 4 อย่างนี้

```bash
make status                                   # pod ใน cluster ต้อง Running (ไม่นับ Completed/Error ของเก่า)
df -h /                                       # ดิสก์ใช้เกิน ~85% → ดูหัวข้อ "ดิสก์เต็ม"
ss -ltnp | grep -E ':(8000|8001|8002|3000) '  # service ที่ควรรัน มี process ฟังพอร์ตอยู่ไหม
kubectl -n arch exec deploy/arch-cache -- valkey-cli --scan --pattern 'arch:*' | head   # ถ้าสงสัยแคช
```

## cluster / โครงสร้างพื้นฐาน

| อาการ | สาเหตุ | แก้ |
|---|---|---|
| `make master` ขึ้น "Connection reset by peer" / pod โดนไล่หมด | **ดิสก์เต็ม** → kubelet ใส่ taint `DiskPressure` แล้ว evict pod ทั้งหมด | เคลียร์ดิสก์ให้ต่ำกว่า ~85% แล้วรอ pod กลับ (`kubectl describe node \| grep -i pressure`). ตัวใหญ่ที่เจอ: `target/` ของ cargo (หลายสิบ GB — ตั้ง `CARGO_TARGET_DIR` ไว้ที่ `~/.cache/…` แล้วลบได้), `~/.local/share/containers`, volume docker ที่ไม่ใช้ |
| หลัง restart เครื่อง/เน็ต: `arch-identity` ค้าง `Init:Error` / `Unknown`, Kratos ไม่ขึ้น (`lookup pgedge … connection refused`) | CoreDNS ของ k3s สตาร์ทไม่สำเร็จรอบแรก (`Unknown`) → DNS ใน cluster ใช้ไม่ได้ | `kubectl -n kube-system delete pod -l k8s-app=kube-dns` แล้ว `kubectl -n arch delete pod -l app.kubernetes.io/name=arch-identity` |
| `arch-streaming-console` `CrashLoopBackOff` ตอนเริ่ม | console รอ Redpanda พร้อม | รอสักครู่ จะหายเอง |
| pod `Completed` / `Error` เก่า ๆ เต็ม `make status` | pod รุ่นก่อนที่ถูกแทนที่แล้ว | `kubectl -n arch delete pod --field-selector=status.phase=Succeeded` (และ `Failed`) |
| ไฟล์รายงานหาย (พรีวิว 503/410 `REPORT_FILE_GONE`), topic หาย, ดัชนี Quickwit หาย | เดิม Garage/Redpanda/Quickwit ใช้ `emptyDir` → pod เกิดใหม่แล้วข้อมูลหาย (ตอนนี้ใช้ PVC แล้ว ข้อมูลอยู่รอด) | ถ้าเกิดกับ cluster ที่สร้างก่อนมี PVC: `make deploy` เพื่อใช้ PVC แล้ว `make topics` |
| `make seed` หยุดด้วย "database was loaded from an older sql/01_ysql.sql" | schema เก่ากว่าไฟล์ในโค้ด (ไฟล์ 00–50 โหลดครั้งเดียว) | ดู [`sql/README.md`](../../sql/README.md): เพิ่ม migration แบบ idempotent หรือสร้างฐานใหม่ `make down && make bootstrap` (**ลบข้อมูลทั้งหมด**) |
| ไม่มี `psql` ในเครื่อง | อยู่ใน pod | `kubectl -n arch exec -it arch-database-0 -- psql -U postgres -d arch` (ใช้ `set search_path=arch;`) |

## ล็อกอิน / สิทธิ์

| อาการ | สาเหตุ | แก้ |
|---|---|---|
| ล็อกอินแล้ว **500** (หรือ Kratos log `login-guard … connection refused`) | Kratos เรียก hook ไป api-master `:8000` ก่อนอนุญาต | `make master` ให้ api-master รันก่อนล็อกอิน |
| ล็อกอินแล้ว **500** และ api-master log ว่า `login guard secret not configured` (503) | ไม่มี `KRATOS_WEBHOOK_SECRET` ใน `.env` (hook ปิดตายเมื่อไม่ได้ตั้งค่า) | ตั้งค่าให้ตรงกับ `X-Webhook-Secret` ใน configmap ของ Kratos (ดู `.env.example`) |
| `curl` ล็อกอินแล้ว **403 CSRF** | ส่ง `csrf_token` แบบไม่ encode (มีตัว `+ / =`) ทำให้เพี้ยน | ใช้ `--data-urlencode "csrf_token=$CSRF"` |
| เมนู/ปุ่มไม่ตรงกับสิทธิ์ที่เพิ่งแก้ | สิทธิ์และเมนูถูกแคช 60 วินาที (แต่การแก้ผ่านเว็บล้างให้ทันที) | ถ้าแก้ด้วย SQL ตรง ๆ: `make cache-flush` |
| บัญชีไม่มีเมนูเลย | กลุ่มของผู้ใช้ไม่มี READ ที่โปรแกรมไหนเลย / กลุ่มปิดอยู่ / ผู้ใช้ `active=false` | ดูที่ SUPG003 / SUPG004 / SUPG005 |
| API ตอบ 401 `Missing 'ory_kratos_session' cookie` | ไม่ได้ส่งคุกกี้ session | ล็อกอินผ่านเว็บ หรือใส่ `-b jar` ใน curl |

## เว็บ (Trunk / Leptos)

| อาการ | สาเหตุ | แก้ |
|---|---|---|
| เปลี่ยนโค้ดแล้วหน้าไม่เปลี่ยน | wasm (debug) ใหญ่ ใช้เวลา build; หรือเบราว์เซอร์แคช | รอ trunk ขึ้น "success", แล้ว hard reload (Ctrl+Shift+R) |
| หน้ารีโหลดเองซ้ำ ๆ | trunk live-reload ตอน build เสร็จ | ปกติ |
| API ผ่านเว็บได้ 404/502 | ลำดับ `[[proxy]]` ผิด หรือ service นั้นไม่ได้รัน (config สร้างจาก `.env` ตอน `make web` — แก้ `.env` แล้วต้องรัน `make web` ใหม่) | ตัวที่ prefix ยาวกว่าอยู่ก่อน (`/api/v1/notifications/` → 8002, `/api/v1/reports/` → 8001, `/api/v1/` → 8000) แล้วเช็กพอร์ตด้วย `ss` |
| ดาวน์โหลดรายงานได้ "Invalid signature" | proxy ตาม redirect ของ presigned URL | API ตอบ JSON `{url}` ให้เบราว์เซอร์เปิดเอง (ทำอยู่แล้ว) — ถ้าเปิดเว็บจากเครื่องอื่น `AWS_ENDPOINT_URL` ยังเป็น `localhost:3900` ต้องตั้งเป็น URL ที่เบราว์เซอร์เข้าถึงได้ |
| สไตล์ `bg-primary/10`, `border-border/60` ไม่ขึ้น | Tailwind ทำ alpha กับตัวแปรสีธีม (`var(--…)`) ไม่ได้ | ใช้ `bg-[color-mix(in_oklch,var(--primary)_12%,transparent)]` |
| คลาส Tailwind ใหม่ไม่มีผล | CSS ถูกคอมไพล์ตอน build จากชื่อคลาสที่ปรากฏเต็ม ๆ ใน `.rs` — ถ้าประกอบด้วย `format!` ให้เขียนชื่อเต็มหรือใส่ `safelist` ใน `tailwind.config.js`; รอ trunk build เสร็จแล้ว hard reload |
| `make web` ครั้งแรกช้า/ล้มเหลวตอนโหลด Tailwind | trunk ต้องดาวน์โหลด Tailwind CLI 3.4.17 จาก GitHub ครั้งเดียว (เก็บใน `~/.cache/trunk`) — ถ้าออฟไลน์ ให้วาง binary `tailwindcss` เวอร์ชัน 3.4.17 ไว้ใน `PATH` |
| dropdown ไม่ปิดเมื่อคลิกที่อื่น | ชั้นโปร่งใส `fixed inset-0` ใน topbar (มี `backdrop-blur`) คลุมแค่ topbar | ใช้ `web_package::ui::component::dismiss_on_outside` |
| คอมไพล์ wasm ไม่ผ่านเพราะ web-sys | ไม่ได้เปิด feature | เพิ่มใน `Cargo.toml` ของ `handler/web/package` / `handler/web/application/master` |

## ข้อความ / รายงาน / แจ้งเตือน / audit

| อาการ | สาเหตุ | แก้ |
|---|---|---|
| ข้อความหน้าจอเป็นภาษาไทยทั้งที่เลือกอังกฤษ | ขาด key ใน `db_label` (ใช้ข้อความไทยสำรองจาก `t_or`) | เพิ่มใน `sql/NN_i18n_*.sql` แล้ว `make seed` |
| กด export แล้วไม่มีอะไรเกิดขึ้น | `api-report` (8001) ไม่ได้รัน หรือ topic `report.requests` ไม่มี | `make report`, `make topics` |
| ไม่ได้รับอีเมลตั้งรหัสผ่าน | `api-notification` ไม่ได้รัน หรือไม่มี topic `mail-outbox` หรือ `mail.smtp_url` ไม่ถูกต้อง (ดู log ของ api-notification) | `make notification`, `make topics`, ตรวจ `su_configuration` |
| กระดิ่งไม่เด้ง | `api-notification` (8002) ไม่ได้รัน / ไม่มี topic `notifications` | `make notification`, `make topics` |
| งานรายงาน `running` ค้าง | worker ตายกลางทาง | ดู ALPG001 (Process Monitor): งานค้างเกิน 15 นาทีกด "ประมวลผลใหม่" ได้ |
| Ctrl+C แล้ว api-report ไม่ยอมหยุด | SSE ที่เปิดค้าง | แก้แล้ว (ปิด stream + watchdog 10 วินาที); Ctrl+C ซ้ำออกทันที |
| ALPG000 ว่าง / "audit service down" | Quickwit ล่ม หรือดัชนียังไม่ถูกสร้าง | `kubectl -n arch get pod -l app.kubernetes.io/name=arch-logging`; ดัชนี `audit-logs-v1` ถูกสร้างเองตอนมี event แรก; ดัชนี event ของ Redpanda: `make event-indexes` |
| ALPG000 ไม่เห็น "Event: …" | ยังไม่ได้ `make topics` / `make event-indexes` | รันทั้งสอง |

## แคช Valkey

| อาการ | สาเหตุ | แก้ |
|---|---|---|
| ข้อมูลที่เพิ่งแก้ยังเป็นค่าเก่า | แก้ด้วย SQL/seed (ไม่ผ่าน API จึงไม่มีการล้าง) | `make cache-flush` (`make seed` เรียกให้) |
| สงสัยว่าแคชทำให้ผลเพี้ยน | — | `READ_CACHE_ENABLED=false` ใน `.env` แล้ว restart api-master เทียบผล |
| อยากดูว่า hit/miss | — | log target `arch::cache` (`RUST_LOG="…,arch::cache=debug"`); รายละเอียดที่ [`11_VALKEY_READ_CACHE.md`](../architecture/11_VALKEY_READ_CACHE.md) |

## เคล็ดลับตอน debug

- ดู log ของ pod: `make logs APP=arch-identity` (ต้องมี `stern`) หรือ `kubectl -n arch logs deploy/arch-identity -c kratos`
- ดู topic: `kubectl -n arch exec deploy/arch-streaming -- rpk topic list` / `rpk topic consume <topic> -n 5`
- ดู key แคช: `kubectl -n arch exec deploy/arch-cache -- valkey-cli --scan --pattern 'arch:*'`
- ค้นอีเวนต์ใน Quickwit: `curl 'localhost:7280/api/v1/events-notifications-v1/search?query=*'`
- ถ้าสั่ง `pkill -f <คำ>` แล้ว shell ตัวเองตายด้วย แปลว่าคำนั้นอยู่ในบรรทัดคำสั่งของคุณเอง — หา PID จาก `ss -ltnp` แทน
