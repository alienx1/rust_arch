-- UI texts of NTPG000 (Notifications: the bell and the list). Idempotent; ids are v7-style
-- (PostgreSQL 17 has no uuidv7()).

WITH t(lang, category, key, value) AS (VALUES
  ('th', 'ntpg000', 'title', 'การแจ้งเตือน (Notifications)'),
  ('en', 'ntpg000', 'title', 'Notifications'),
  ('th', 'ntpg000', 'description', 'สิ่งที่ระบบแจ้งให้คุณทราบ ส่งถึงทุกหน้าจอที่เปิดอยู่ผ่านไอคอนกระดิ่ง • รหัสโปรแกรม: NTPG000'),
  ('en', 'ntpg000', 'description', 'What the system tells you, pushed to every open screen through the bell • Program code: NTPG000'),
  ('th', 'ntpg000', 'table.title', 'การแจ้งเตือน'),
  ('en', 'ntpg000', 'table.title', 'Notifications'),
  ('th', 'ntpg000', 'table.description', 'สิ่งที่ระบบแจ้งให้คุณทราบ (เช่น รายงานที่สร้างเสร็จ) • ส่งแบบเรียลไทม์ไปทุกหน้าจอที่คุณเปิดอยู่'),
  ('en', 'ntpg000', 'table.description', 'What the system tells you (e.g. a finished report) • pushed in real time to every screen you have open'),
  ('th', 'ntpg000', 'column.time', 'เวลา'),
  ('en', 'ntpg000', 'column.time', 'Time'),
  ('th', 'ntpg000', 'column.what', 'การแจ้งเตือน'),
  ('en', 'ntpg000', 'column.what', 'Notification'),
  ('th', 'ntpg000', 'column.status', 'สถานะ'),
  ('en', 'ntpg000', 'column.status', 'Status'),
  ('th', 'ntpg000', 'column.actions', 'จัดการ'),
  ('en', 'ntpg000', 'column.actions', 'Actions'),
  ('th', 'ntpg000', 'status.unread', 'ใหม่'),
  ('en', 'ntpg000', 'status.unread', 'New'),
  ('th', 'ntpg000', 'status.read', 'อ่านแล้ว'),
  ('en', 'ntpg000', 'status.read', 'Read'),
  ('th', 'ntpg000', 'state.loading', 'กำลังโหลดการแจ้งเตือน...'),
  ('en', 'ntpg000', 'state.loading', 'Loading notifications...'),
  ('th', 'ntpg000', 'state.empty_title', 'ยังไม่มีการแจ้งเตือน'),
  ('en', 'ntpg000', 'state.empty_title', 'No notifications yet'),
  ('th', 'ntpg000', 'state.empty_text', 'เมื่อมีเรื่องที่ต้องแจ้ง จะแสดงที่นี่และที่ไอคอนกระดิ่ง'),
  ('en', 'ntpg000', 'state.empty_text', 'When there is something to tell you it shows here and on the bell'),
  ('th', 'ntpg000', 'action.open', 'เปิด'),
  ('en', 'ntpg000', 'action.open', 'Open'),
  ('th', 'ntpg000', 'error.load', 'ไม่สามารถโหลดการแจ้งเตือนได้'),
  ('en', 'ntpg000', 'error.load', 'Could not load the notifications'),
  ('th', 'ntpg000', 'bell.label', 'การแจ้งเตือน'),
  ('en', 'ntpg000', 'bell.label', 'Notifications'),
  ('th', 'ntpg000', 'bell.title', 'การแจ้งเตือน'),
  ('en', 'ntpg000', 'bell.title', 'Notifications'),
  ('th', 'ntpg000', 'bell.mark_read', 'อ่านทั้งหมดแล้ว'),
  ('en', 'ntpg000', 'bell.mark_read', 'Mark all as read'),
  ('th', 'ntpg000', 'bell.empty', 'ยังไม่มีการแจ้งเตือน'),
  ('en', 'ntpg000', 'bell.empty', 'No notifications yet'),
  ('th', 'ntpg000', 'bell.view_all', 'ดูการแจ้งเตือนทั้งหมด'),
  ('en', 'ntpg000', 'bell.view_all', 'View all notifications')
)
INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  l."id", t.category, t.key, t.value
FROM t JOIN "db_language" l ON l."code" = t.lang
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
