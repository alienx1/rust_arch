# เช็กลิสต์ก่อนขึ้นระบบจริง

repo นี้ตั้งค่าไว้สำหรับ **พัฒนาบนเครื่อง** (k3d, node เดียว, ค่า dev เปิดเผยใน repo) — ยังไม่ใช่ production-ready
เอกสารนี้ไล่สิ่งที่ต้องเปลี่ยน/ตัดสินใจ พร้อมบอกว่าต้องแก้ที่ไหน (ทำเครื่องหมาย ☐ เมื่อเสร็จ)

> สถานะปัจจุบัน: มี overlay เดียวคือ `kube/arch/overlays/local`; `make deploy ENV=staging|production` **ยังใช้ไม่ได้** (ต้องสร้าง `kube/arch/overlays/<env>/` เอง); service ของแอป (`api-*`) รันด้วย `cargo` บนเครื่อง ไม่มี Dockerfile/manifest

## 1. ความลับและรหัสผ่าน (ทุกตัวต้องเปลี่ยน และไม่เก็บใน repo)

| ☐ | ค่า | อยู่ที่ไหนตอนนี้ | หมายเหตุ |
|---|---|---|---|
| ☐ | รหัส Postgres `postgres` | `.env` (`DATABASE_URL`, `POSTGRES_PASSWORD`), `kube/arch/shared/security/*.yaml`, **DSN ของ Kratos ใน configmap** (`arch_identity/base/configmap.yaml`) | ใช้ Secret/Vault; ใช้ role แยกต่อ service ที่สิทธิ์น้อยที่สุด (ตอนนี้ทุกตัวใช้ superuser `postgres`) |
| ☐ | `KRATOS_SECRET_COOKIE`, `KRATOS_SECRET_CIPHER` | `.env`, `shared-credentials-secret.yaml` | สุ่มใหม่ ≥32 ตัวอักษร; การเปลี่ยนจะทำให้ session เดิมใช้ไม่ได้ |
| ☐ | **webhook `login-guard`** (`login-guard-dev-secret-…`) | `arch_identity/base/configmap.yaml` **และ** `KRATOS_WEBHOOK_SECRET` ของ api-master | สองฝั่งต้องตรงกัน; ย้ายไป Secret |
| ☐ | Garage: access key/secret (`GKa4c1e…`), `GARAGE_ADMIN_TOKEN`, `GARAGE_RPC_SECRET` | `.env`, `arch_storage/base/configmap.yaml` (รวมสคริปต์ bootstrap ที่ฝัง key/secret) | ย้ายไป Secret; ออก key ใหม่ให้แต่ละ service; admin API ไม่ต้องเปิดออกนอก cluster |
| ☐ | `REPORT_API_KEY` | `.env` | เปลี่ยนหรือเอาออกถ้าไม่ได้ใช้ |
| ☐ | บัญชีล็อกอิน dev: `admin@arch.local / Admin@2026`, `test_abac@arch.local / Test@2026` | `make seed-admin`, `make seed-test-users`, `sql/53_admin_seed.sql`, `sql/54_abac_test_seed.sql` | **ห้ามรัน `seed-test-users` / `54_` ใน production**; สร้าง admin จริงด้วยรหัสของตัวเอง แล้วลบ/ปิดบัญชี dev |
| ☐ | ตรวจว่าไม่มีความลับหลุดใน git history | — | repo มี `.gitignore` แล้ว (ไม่ track `.env`, `.kube/`, `target/`, `dist/`) แต่ยังไม่ได้อยู่ใน git — ก่อน `git init`/commit ครั้งแรกให้ตรวจ `git status` ว่าไม่มีความลับหลุดเข้าไป; ถ้าเคย commit `.env` จริงให้ถือว่ารั่วและเปลี่ยนทั้งหมด |

## 2. การยืนยันตัวตน (Ory Kratos)

| ☐ | รายการ | ที่แก้ |
|---|---|---|
| ☐ | ปิด `dev: true` | `kratos.yml` ใน `arch_identity/base/configmap.yaml` |
| ☐ | `base_url`, `ui_url`, `allowed_return_urls`, CORS ให้เป็นโดเมนจริง (ตอนนี้ `localhost`) | เดียวกัน + `CORS_ALLOWED_ORIGINS` ใน `.env` |
| ☐ | **ตัดสินใจเรื่องสมัครเอง (registration flow เปิดอยู่)**: ปิดถ้าบัญชีต้องสร้างผ่าน SUPG004 เท่านั้น | `selfservice.flows.registration` |
| ☐ | คุกกี้ `Secure` + `SameSite` เหมาะสม ภายใต้ HTTPS | Kratos `serve.public` / cookies |
| ☐ | Kratos **admin API (4434)** เข้าได้เฉพาะภายใน cluster | ตอนนี้ k3d map ออกเครื่อง — ดูหัวข้อ 4 |
| ☐ | นโยบายรหัสผ่าน / MFA / อีเมล verification-recovery ผ่าน SMTP จริง | `kratos.yml` (ตอนนี้ลิงก์กู้คืนออกผ่าน SUPG004 เท่านั้น) |
| ☐ | api-master ต้องรันก่อน/ตลอดเวลาที่มีคนล็อกอิน (hook `login-guard`) — วางให้ HA และมี health check | deployment ของ api-master |

## 3. เครือข่ายและ TLS

| ☐ | รายการ | หมายเหตุ |
|---|---|---|
| ☐ | HTTPS ทุกจุดที่ผู้ใช้เข้า (Traefik/Ingress + cert) | ตอนนี้ HTTP ล้วน |
| ☐ | เปิดเฉพาะพอร์ตที่จำเป็น | `k3s-up.sh` map ออกเครื่อง: 5432, 6379, 9092, 9644, 8080, 4433, **4434**, 3900, **3903**, 7280, 7281 — production ควรเหลือแค่ทางเข้าเว็บ/API/Kratos public |
| ☐ | NetworkPolicy ให้ DB/Valkey/Redpanda/Quickwit/Garage ถูกเรียกจาก service ที่ได้รับอนุญาตเท่านั้น | ดู `docs/architecture/09_…` |
| ☐ | แทน proxy ของ Trunk ด้วย reverse proxy จริง (path เดียวกัน: `/api/kratos`, `/api/v1/notifications`, `/api/v1/reports`, `/api/v1`) | `Trunk.toml.tmpl` / `gen-trunk-config.sh` ใช้ตอน dev เท่านั้น |
| ☐ | CORS จำกัดเฉพาะโดเมนจริง และ `CORS_ALLOW_CREDENTIALS` ตรวจแล้ว | `.env` |
| ☐ | Rate limit / WAF หน้า API (ตัวแอปไม่มี) | gateway |
| ☐ | เชื่อ header `X-Forwarded-For` เฉพาะจาก proxy ที่ไว้ใจ (audit เก็บ IP จาก header นี้) | `infrastructure/src/middleware/audit_trail.rs` |

## 4. ข้อมูลและความทนทาน

| ☐ | รายการ | สถานะปัจจุบัน | ต้องทำ |
|---|---|---|---|
| ☐ | **สำรองข้อมูล** Postgres, Garage, Quickwit | ไม่มี; PVC เป็น `local-path` (ผูกกับ node เดียว) | backup อัตโนมัติ + ทดสอบกู้คืน (`pg_dump`/WAL, snapshot bucket) |
| ☐ | Postgres HA / multi-region (pgEdge Spock) | node เดียว | ตาม `docs/architecture/04_MULTI_REGION_TOPOLOGY.md` |
| ☐ | Redpanda | โหมด `dev-container` node เดียว RAM 1 GB | โหมด production, replication ≥3, SASL/TLS |
| ☐ | Garage | `replication_factor = 1`, node เดียว | ≥3 node/zone, lifecycle ของไฟล์รายงาน (ตอนนี้ไม่ลบเอง) |
| ☐ | Quickwit | metastore/ดัชนีบนดิสก์เดียว | ใช้ object storage/Postgres เป็น metastore; **ดัชนี `audit-logs-v1` ไม่มี retention** (ดัชนี `events-*` เก็บ 90 วัน) |
| ☐ | การเปลี่ยน schema | `sql/01_ysql.sql` โหลดครั้งเดียว | ใช้ migration แบบ idempotent — ดู [`sql/README.md`](../../sql/README.md) |
| ☐ | การลบข้อมูลเก่า | `nt_notification` (แจ้งเตือน), `rp_report_job` + ไฟล์ใน Garage, `al_data_change` ที่ค้าง ไม่มี retention | กำหนดนโยบาย + job ลบ |

## 5. ข้อมูลส่วนบุคคลและ audit

- audit เก็บ **IP, user agent, header/body ของ request และ response** (ปิดบังความลับ เช่น คุกกี้/รหัสผ่านเป็น `***`) และ before/after ของทุกแถวที่แก้ — ทบทวนกับนโยบาย PDPA/ความเป็นส่วนตัว และกำหนดอายุเก็บ (☐)
- ปิด/เปิด: `AUDIT_LOG_ENABLED` — ปิดแล้วหน้า ALPG000 จะว่างและไม่มีหลักฐานการตรวจสอบ
- สิทธิ์ดู audit (`ALPG000`) และสั่ง reprocess (`ALPG001`) ควรจำกัดผู้ดูแลเท่านั้น ☐

## 6. แอปพลิเคชัน

| ☐ | รายการ | ที่แก้ |
|---|---|---|
| ☐ | สร้าง image + manifest ของ `api-master`, `api-report`, `api-notification` (resources, probes `/livez` `/readyz`, graceful shutdown ≥ 10 วินาที) | ใหม่ |
| ☐ | build เว็บแบบ release (`trunk build --release`) แล้วเสิร์ฟเป็นไฟล์สถิต; ตั้ง `APP_ENV` ให้หน้า `/demo` ถูกซ่อน (เปิดเฉพาะ localhost/dev ผ่าน `is_dev_env()`) | `handler/web/application/master` |
| ☑ | Tailwind: CSS ถูกคอมไพล์ตอน build ด้วย **Tailwind CLI 3.4.17** (trunk ดาวน์โหลดเวอร์ชันที่ pin ใน `Trunk.toml.tmpl`) ไม่ได้รันในเบราว์เซอร์และไม่โหลดอะไรจากภายนอกอีก; ฟอนต์ Inter เสิร์ฟเองจาก `handler/web/package/fonts/` (A08). เหลือ: ใส่ Content-Security-Policy (ตอนนี้ไม่มี Tailwind ในเบราว์เซอร์แล้ว จึงเข้มได้มากขึ้น แต่ยังมี inline script สั้น ๆ ของธีม/trunk ต้องจัดการด้วย nonce/hash) | `tailwind.config.js`, `index.html` |
| ☐ | URL ที่เบราว์เซอร์เข้าถึงสำหรับไฟล์ใน Garage: presigned URL สร้างจาก `AWS_ENDPOINT_URL` — ต้องเป็นโดเมนสาธารณะของ S3 (ยังไม่มีตัวแปรแยก "public URL") | `infrastructure` (s3 presign) |
| ☐ | หลาย instance: `NOTIFICATION_CONSUMER_GROUP` ต่อ instance ต้องไม่ซ้ำกัน; worker รายงานใช้ group ร่วมกัน | `.env`/deployment |
| ☐ | หลาย region: แคช Valkey แยกต่อ region → ส่งอีเวนต์ล้างแคชผ่าน Redpanda (ตอนนี้พึ่ง TTL) | `docs/architecture/11_VALKEY_READ_CACHE.md` |
| ☐ | ระดับ log: ตั้ง `RUST_LOG` ที่เหมาะสม (ค่าเริ่มต้นของ dev เปิด debug ของ `api_master` และ `arch::cache`) | env |
| ☐ | เปลี่ยนค่า dev toggles ที่ไม่ควรเปิดจริง: `READ_CACHE_ENABLED`, `CACHE_STATS_SECS` | `.env` |

## 7. กระบวนการ

- ☐ CI: build + `cargo test --workspace --exclude master-app` + wasm check + `kubeconform` + `cargo audit` (ตอนนี้ไม่มี pipeline; repo ยังไม่ได้อยู่ใน git)
- ☐ รูปแบบโค้ด: เพิ่ม `rustfmt.toml` / `clippy.toml` แล้วบังคับใน CI
- ☐ แผนรับมือเหตุ: ใครเป็น on-call, วิธีกู้คืน, runbook (เริ่มจาก [`TROUBLESHOOTING.md`](./TROUBLESHOOTING.md))
- ☐ ทดสอบโหลด/ความจุ: audit, export ระดับล้านแถว (ตอนนี้วัดเฉพาะขั้นดึงจาก Quickwit)
