# Arch — Master Data & Platform (Rust 2024)

ระบบ master data / reporting แบบ multi-region เขียนด้วย Rust: **Axum** (API) + **Leptos** (เว็บ WebAssembly) +
**Postgres/pgEdge** + **Ory Kratos** (ล็อกอิน) + **Valkey** (แคช) + **Redpanda** (event) + **Garage** (S3) + **Quickwit** (audit / event search)

> เอกสารนี้คือทางเข้า — กฎการเขียนโค้ดอยู่ที่ [`AGENTS.md`](./AGENTS.md), สถาปัตยกรรมอยู่ที่ [`docs/`](./docs/README.md)

## เริ่มใช้งาน

ต้องมี: **Nix (เปิด flakes)** และ **Podman (rootless) หรือ Docker** สำหรับ k3d — Rust, trunk, kubectl, k3d, kustomize, jq ฯลฯ มากับ `flake.nix`

```bash
cp .env.example .env        # 0. (ถ้ายังไม่มี .env) ค่า dev พร้อมใช้ รวมถึง KRATOS_WEBHOOK_SECRET ที่ล็อกอินต้องใช้
nix develop                 # 1. เข้า shell ที่มีเครื่องมือครบ (ตั้ง rustup + target wasm32 ให้)
make bootstrap              # 2. k3d cluster + Postgres/Valkey/Redpanda/Kratos/Garage/Quickwit + schema/seed + บัญชีล็อกอิน
make topics                 # 3. สร้าง Redpanda topics + ดัชนี event ใน Quickwit (bootstrap ไม่ทำให้)
make master                 # 4. api-master :8000   (เปิด terminal ใหม่; ต้องรันก่อนล็อกอิน — ดู "ล็อกอิน")
make web                    # 5. เว็บ :3000 (trunk)   → เปิด http://localhost:3000  (ครั้งแรกโหลด Tailwind CLI 3.4.17 ให้เอง ต้องมีเน็ต)
```

เพิ่มเมื่อต้องการฟีเจอร์รายงาน/แจ้งเตือน: `make report` (:8001) และ `make notification` (:8002) ใน terminal แยก

**ล็อกอิน (ค่า dev):** `admin@arch.local` / `Admin@2026` (สิทธิ์เต็ม) และ `test_abac@arch.local` / `Test@2026` (READ อย่างเดียว ผูกกับแผนก IT)
Kratos เรียก hook `login-guard` ไปที่ api-master (:8000) ก่อนอนุญาตล็อกอิน → **ถ้า api-master ไม่ได้รัน ล็อกอินจะได้ 500**

> ค่าทั้งหมดใน `.env` / `kube/` เป็นค่า **dev ที่เปิดเผยใน repo** — ห้ามใช้ขึ้นจริง ดู [`docs/guides/PRODUCTION_CHECKLIST.md`](./docs/guides/PRODUCTION_CHECKLIST.md)

## แผนที่ repo

```text
domain/                  entity ของ Sea-ORM (domain/src/entity) + value object — ไม่รู้จัก DB driver หรือ HTTP
application/
  core/                  ports (trait): cache, event, content(S3), audit, notification, read_cache ฯลฯ
  master/                use case ของ api-master:  features/<ระบบ>/<program>/{commands,queries,models,errors}
  report/                worker + use case รายงาน (RPPG000)
  notification/          รับ/เก็บ/ส่งแจ้งเตือน (NTPG000)
infrastructure/          adapter จริง: Postgres, Valkey, Redpanda, Garage S3, Quickwit, Kratos + middleware
handler/
  api/{master,report,notification}/   Axum: routers/ → เรียก Commands trait เท่านั้น
  web/package/           ชุด UI ใช้ร่วม (DataTable, DateRangePicker, layout, i18n, auth)
  web/application/master/  แอป Leptos: pages/<ระบบ>/<program>/{models,services,providers,components,screens}
sql/                     schema + seed (รันเรียงตามเลขหน้าชื่อไฟล์) — ดู sql/README.md
kube/arch/               manifest k3d/k8s (kustomize overlay `local`) + สคริปต์ utils/
docs/                    architecture/ (12 บท), programs/ (สเปกรายโปรแกรม), guides/ (คู่มือลงมือ)
```

**ทิศทาง dependency เข้าด้านในเท่านั้น:** `handler → application → domain`, `infrastructure` implement ports ของ `application-core` (กฎเต็มใน `AGENTS.md`)

## ใครคุยกับใคร (สรุป)

```text
เว็บ(Leptos) ──HTTP──► api-master ──► Valkey (read cache) ──► Postgres
   │                       │  └─ audit ──► Quickwit (audit-logs-v1)         ◄── ALPG000 ดู/ค้นหา
   │                       └─ report.requests ──► Redpanda ──► api-report ──► Garage (CSV)
   │                                                              └─ notifications ──► api-notification ──SSE──► เว็บ (กระดิ่ง)
   └─ Trunk dev proxy: /api/kratos→4433, /api/v1/notifications→8002, /api/v1/reports→8001, /api/v1→8000
      (สร้างจาก `.env` ตอน `make web`: WEB_PORT, KRATOS_PUBLIC_URL, API_*_URL — ดู `Trunk.toml.tmpl`)
```

พอร์ตทั้งหมด: [`docs/architecture/09_NETWORK_PORTS_AND_SECURITY_MATRIX.md`](./docs/architecture/09_NETWORK_PORTS_AND_SECURITY_MATRIX.md)

## คำสั่งที่ใช้บ่อย

| ต้องการ | คำสั่ง |
|---|---|
| ดูคำสั่งทั้งหมด | `make help` |
| สถานะ cluster / log pod | `make status` / `make logs APP=arch-database` |
| โหลด schema + seed ใหม่ (และล้างแคช) | `make seed` |
| ล้างแคช Valkey อย่างเดียว | `make cache-flush` |
| เช็กคอมไพล์ / ทดสอบ | `cargo check --workspace` / `cargo test --workspace --exclude master-app` |
| เช็กฝั่งเว็บ (wasm) | `cargo check -p master-app --target wasm32-unknown-unknown` |
| เข้า psql (ไม่มี psql ในเครื่อง) | `kubectl -n arch exec -it arch-database-0 -- psql -U postgres -d arch` |
| ปิด/เปิด cluster โดยเก็บข้อมูล | `make stop` / `make up` |
| **ลบ cluster พร้อมข้อมูลทั้งหมด** | `make down` |

> `master-app` (เว็บ) ไม่อยู่ในชุด `cargo test` ของ workspace — ตรวจด้วย wasm check และเปิดเบราว์เซอร์
> แนะนำตั้ง `CARGO_TARGET_DIR` ไว้นอก repo (เช่น `~/.cache/arch_cargo_target`) — โฟลเดอร์ `target/` โตได้หลายสิบ GB

## ต่อยอดต่อ

| ต้องการ | อ่าน |
|---|---|
| เพิ่มหน้าจอ/โปรแกรมใหม่ (DB → API → เว็บ → สิทธิ์ → i18n → เอกสาร) | [`docs/guides/ADD_A_PROGRAM.md`](./docs/guides/ADD_A_PROGRAM.md) |
| เพิ่ม service ใหม่ (worker อ่าน Redpanda / API แยก) | [`docs/guides/ADD_A_SERVICE.md`](./docs/guides/ADD_A_SERVICE.md) |
| มีอะไรพัง | [`docs/guides/TROUBLESHOOTING.md`](./docs/guides/TROUBLESHOOTING.md) |
| เปลี่ยน schema ฐานข้อมูล | [`sql/README.md`](./sql/README.md) |
| ก่อนขึ้นระบบจริง | [`docs/guides/PRODUCTION_CHECKLIST.md`](./docs/guides/PRODUCTION_CHECKLIST.md) |
| เพิ่มแคช / ดูว่าอะไรถูกแคช | [`docs/architecture/11_VALKEY_READ_CACHE.md`](./docs/architecture/11_VALKEY_READ_CACHE.md) |
| รายการโปรแกรมทั้งหมดและสเปก | [`docs/programs/00_INDEX.md`](./docs/programs/00_INDEX.md) |

ใช้ AI ช่วยเขียนโค้ด: ให้อ่าน [`AGENTS.md`](./AGENTS.md) ก่อน (กฎที่ห้ามละเมิด: ชั้น dependency, ห้ามตั้งชื่อไฟล์ generic, test อยู่ใน `tests/` เท่านั้น, UUIDv7, ฯลฯ)

## License

[MIT](./LICENSE) © 2026 attaporn_j <alienx1step@gmail.com> — โปรเจกต์ส่วนตัว ใช้/แก้ไข/แจกจ่ายต่อได้ตามเงื่อนไขของ MIT
บริการที่รันแยก (Garage, Quickwit, Redpanda, Ory Kratos, Valkey, Postgres) เป็นซอฟต์แวร์ของผู้อื่นภายใต้ license ของตัวเอง และไม่ได้รวมอยู่ใน license นี้
