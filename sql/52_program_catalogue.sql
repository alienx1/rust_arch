-- Systems, modules and programs the application actually routes (drives the sidebar menu).
--
-- Every path matches the SPA router in handler/web/application/master exactly, and every code is
-- a screen that exists: add a row here when a route lands, not ahead of it (a program with no
-- route is a dead sidebar entry). Icons are icondata_ai const names minus the "Ai" prefix.
-- Permissions on these programs are NOT granted here: 53_ derives the ADMIN grants from
-- db_program, and SUPG003 grants everything else.
--
-- Hand-assigned ids: 019ca1ec- (DB system/module), 019ca1ed- (SY/SU/AL + programs),
-- 019ca1ee- (DBPG006). Idempotent (ON CONFLICT), safe to re-run.

------------------------------------------------------
-- 1. Systems and modules
------------------------------------------------------
INSERT INTO "db_system" ("id", "code", "active") VALUES
('019ca1ec-0000-7000-8007-000000000001', 'DB', true) ON CONFLICT ("id") DO NOTHING;
INSERT INTO "db_system_label" ("system_id", "language_id", "label") VALUES
('019ca1ec-0000-7000-8007-000000000001', '019ca1eb-21c0-7000-8001-000000000001', 'ข้อมูลหลัก'),
('019ca1ec-0000-7000-8007-000000000001', '019ca1eb-21c0-7000-8002-000000000002', 'Master Data') ON CONFLICT ("system_id","language_id") DO NOTHING;
INSERT INTO "db_module" ("id", "system_id", "code", "active") VALUES
('019ca1ec-0000-7000-8008-000000000001', '019ca1ec-0000-7000-8007-000000000001', 'DBMT', true) ON CONFLICT ("id") DO NOTHING;
INSERT INTO "db_module_label" ("module_id", "language_id", "label") VALUES
('019ca1ec-0000-7000-8008-000000000001', '019ca1eb-21c0-7000-8001-000000000001', 'ทะเบียนข้อมูลหลัก'),
('019ca1ec-0000-7000-8008-000000000001', '019ca1eb-21c0-7000-8002-000000000002', 'Master Data Maintenance') ON CONFLICT ("module_id","language_id") DO NOTHING;
INSERT INTO "db_system" ("id","code","active") VALUES ('019ca1ed-0000-7000-8001-000000000001','SY',true) ON CONFLICT ("id") DO NOTHING;
INSERT INTO "db_system_label" ("system_id","language_id","label") VALUES
('019ca1ed-0000-7000-8001-000000000001','019ca1eb-21c0-7000-8001-000000000001','ระบบ'),('019ca1ed-0000-7000-8001-000000000001','019ca1eb-21c0-7000-8002-000000000002','System') ON CONFLICT ("system_id","language_id") DO NOTHING;
INSERT INTO "db_system" ("id","code","active") VALUES ('019ca1ed-0000-7000-8001-000000000002','SU',true) ON CONFLICT ("id") DO NOTHING;
INSERT INTO "db_system_label" ("system_id","language_id","label") VALUES
('019ca1ed-0000-7000-8001-000000000002','019ca1eb-21c0-7000-8001-000000000001','ความปลอดภัย'),('019ca1ed-0000-7000-8001-000000000002','019ca1eb-21c0-7000-8002-000000000002','Security') ON CONFLICT ("system_id","language_id") DO NOTHING;
INSERT INTO "db_system" ("id","code","active") VALUES ('019ca1ed-0000-7000-8001-000000000003','AL',true) ON CONFLICT ("id") DO NOTHING;
INSERT INTO "db_system_label" ("system_id","language_id","label") VALUES
('019ca1ed-0000-7000-8001-000000000003','019ca1eb-21c0-7000-8001-000000000001','ตรวจสอบ'),('019ca1ed-0000-7000-8001-000000000003','019ca1eb-21c0-7000-8002-000000000002','Audit') ON CONFLICT ("system_id","language_id") DO NOTHING;
INSERT INTO "db_module" ("id","system_id","code","active") VALUES ('019ca1ed-0000-7000-8002-000000000001','019ca1ed-0000-7000-8001-000000000001','SYMT',true) ON CONFLICT ("id") DO NOTHING;
INSERT INTO "db_module_label" ("module_id","language_id","label") VALUES
('019ca1ed-0000-7000-8002-000000000001','019ca1eb-21c0-7000-8001-000000000001','ระบบงานหลัก'),('019ca1ed-0000-7000-8002-000000000001','019ca1eb-21c0-7000-8002-000000000002','System Core') ON CONFLICT ("module_id","language_id") DO NOTHING;
INSERT INTO "db_module" ("id","system_id","code","active") VALUES ('019ca1ed-0000-7000-8002-000000000002','019ca1ed-0000-7000-8001-000000000002','SUMT',true) ON CONFLICT ("id") DO NOTHING;
INSERT INTO "db_module_label" ("module_id","language_id","label") VALUES
('019ca1ed-0000-7000-8002-000000000002','019ca1eb-21c0-7000-8001-000000000001','จัดการสิทธิ์'),('019ca1ed-0000-7000-8002-000000000002','019ca1eb-21c0-7000-8002-000000000002','Security Maintenance') ON CONFLICT ("module_id","language_id") DO NOTHING;
INSERT INTO "db_module" ("id","system_id","code","active") VALUES ('019ca1ed-0000-7000-8002-000000000003','019ca1ed-0000-7000-8001-000000000003','ALMT',true) ON CONFLICT ("id") DO NOTHING;
INSERT INTO "db_module_label" ("module_id","language_id","label") VALUES
('019ca1ed-0000-7000-8002-000000000003','019ca1eb-21c0-7000-8001-000000000001','ตรวจสอบระบบ'),('019ca1ed-0000-7000-8002-000000000003','019ca1eb-21c0-7000-8002-000000000002','Audit Maintenance') ON CONFLICT ("module_id","language_id") DO NOTHING;

------------------------------------------------------
-- 2. Programs
------------------------------------------------------
-- Folders first: db_program.program_id is self-referencing, so a child cannot
-- be inserted before its parent row is committed.
INSERT INTO "db_program" ("id","system_id","module_id","program_id","code","path","icon","sort_order","active") VALUES
('019ca1ed-0000-7000-8003-000000000001','019ca1ed-0000-7000-8001-000000000001','019ca1ed-0000-7000-8002-000000000001',NULL,'SYPG000','/','DashboardOutlined',1,true),
('019ca1ed-0000-7000-8003-000000000010','019ca1ec-0000-7000-8007-000000000001','019ca1ec-0000-7000-8008-000000000001',NULL,'DBPG000','/db/dbpg000','DatabaseOutlined',2,true),
('019ca1ed-0000-7000-8003-000000000020','019ca1ed-0000-7000-8001-000000000002','019ca1ed-0000-7000-8002-000000000002',NULL,'SUPG000',NULL,'LockOutlined',3,true),
('019ca1ed-0000-7000-8003-000000000030','019ca1ed-0000-7000-8001-000000000003','019ca1ed-0000-7000-8002-000000000003',NULL,'ALPG000_GROUP',NULL,'AuditOutlined',4,true),
('019ca1ed-0000-7000-8003-000000000040','019ca1ed-0000-7000-8001-000000000003','019ca1ed-0000-7000-8002-000000000003',NULL,'RPPG000','/rp/rppg000','FileDoneOutlined',5,true),
('019ca1ed-0000-7000-8003-000000000041','019ca1ed-0000-7000-8001-000000000003','019ca1ed-0000-7000-8002-000000000003',NULL,'NTPG000','/nt/ntpg000','BellOutlined',6,true)
ON CONFLICT ("id") DO UPDATE SET "path"=EXCLUDED."path","icon"=EXCLUDED."icon","sort_order"=EXCLUDED."sort_order","active"=true;
-- Menu rule (SUPG002 get_menu_tree + the sidebar): a folder (a program with children) is only a
-- group header and never a link, even when it has a path. DBPG000 is the folder "ข้อมูลหลัก" AND
-- the organizations main screen, so the entry that opens that screen is DBPG001 ("กำหนดองค์กรและ
-- หน่วยงาน"), whose path points at it; DBPG001's own detail forms live under /db/dbpg001/org and
-- /db/dbpg001/company and are reached from that list.
INSERT INTO "db_program" ("id","system_id","module_id","program_id","code","path","icon","sort_order","active") VALUES
('019ca1ed-0000-7000-8003-000000000011','019ca1ec-0000-7000-8007-000000000001','019ca1ec-0000-7000-8008-000000000001','019ca1ed-0000-7000-8003-000000000010','DBPG001','/db/dbpg000','ApartmentOutlined',1,true),
('019ca1ed-0000-7000-8003-000000000012','019ca1ec-0000-7000-8007-000000000001','019ca1ec-0000-7000-8008-000000000001','019ca1ed-0000-7000-8003-000000000010','DBPG002','/db/dbpg002','BankOutlined',2,true),
('019ca1ed-0000-7000-8003-000000000013','019ca1ec-0000-7000-8007-000000000001','019ca1ec-0000-7000-8008-000000000001','019ca1ed-0000-7000-8003-000000000010','DBPG003','/db/dbpg003','IdcardOutlined',3,true),
('019ca1ed-0000-7000-8003-000000000014','019ca1ec-0000-7000-8007-000000000001','019ca1ec-0000-7000-8008-000000000001','019ca1ed-0000-7000-8003-000000000010','DBPG004','/db/dbpg004','GroupOutlined',4,true),
('019ca1ed-0000-7000-8003-000000000015','019ca1ec-0000-7000-8007-000000000001','019ca1ec-0000-7000-8008-000000000001','019ca1ed-0000-7000-8003-000000000010','DBPG005','/db/dbpg005','GlobalOutlined',5,true),
('019ca1ee-0000-7000-8003-000000000001','019ca1ec-0000-7000-8007-000000000001','019ca1ec-0000-7000-8008-000000000001','019ca1ed-0000-7000-8003-000000000010','DBPG006','/db/dbpg006','TranslationOutlined',6,true),
('019ca1ed-0000-7000-8003-000000000021','019ca1ed-0000-7000-8001-000000000002','019ca1ed-0000-7000-8002-000000000002','019ca1ed-0000-7000-8003-000000000020','SUPG001','/su/supg001','ClusterOutlined',1,true),
('019ca1ed-0000-7000-8003-000000000022','019ca1ed-0000-7000-8001-000000000002','019ca1ed-0000-7000-8002-000000000002','019ca1ed-0000-7000-8003-000000000020','SUPG002','/su/supg002','MenuOutlined',2,true),
('019ca1ed-0000-7000-8003-000000000023','019ca1ed-0000-7000-8001-000000000002','019ca1ed-0000-7000-8002-000000000002','019ca1ed-0000-7000-8003-000000000020','SUPG003','/su/supg003','SafetyOutlined',3,true),
('019ca1ed-0000-7000-8003-000000000024','019ca1ed-0000-7000-8001-000000000002','019ca1ed-0000-7000-8002-000000000002','019ca1ed-0000-7000-8003-000000000020','SUPG004','/su/supg004','UserOutlined',4,true),
('019ca1ed-0000-7000-8003-000000000025','019ca1ed-0000-7000-8001-000000000002','019ca1ed-0000-7000-8002-000000000002','019ca1ed-0000-7000-8003-000000000020','SUPG005','/su/supg005','SafetyCertificateOutlined',5,true),
('019ca1ed-0000-7000-8003-000000000031','019ca1ed-0000-7000-8001-000000000003','019ca1ed-0000-7000-8002-000000000003','019ca1ed-0000-7000-8003-000000000030','ALPG000','/al/alpg000','FileSearchOutlined',1,true),
('019ca1ed-0000-7000-8003-000000000032','019ca1ed-0000-7000-8001-000000000003','019ca1ed-0000-7000-8002-000000000003','019ca1ed-0000-7000-8003-000000000030','ALPG001','/al/alpg001','SyncOutlined',2,true)
ON CONFLICT ("id") DO UPDATE SET "path"=EXCLUDED."path","icon"=EXCLUDED."icon","sort_order"=EXCLUDED."sort_order","active"=true;

INSERT INTO "db_program_label" ("program_id","language_id","label") VALUES
('019ca1ed-0000-7000-8003-000000000001','019ca1eb-21c0-7000-8001-000000000001','ภาพรวมระบบ'),
('019ca1ed-0000-7000-8003-000000000001','019ca1eb-21c0-7000-8002-000000000002','Dashboard'),
('019ca1ed-0000-7000-8003-000000000010','019ca1eb-21c0-7000-8001-000000000001','ข้อมูลหลัก'),
('019ca1ed-0000-7000-8003-000000000010','019ca1eb-21c0-7000-8002-000000000002','Master Data'),
('019ca1ed-0000-7000-8003-000000000011','019ca1eb-21c0-7000-8001-000000000001','กำหนดองค์กรและหน่วยงาน'),
('019ca1ed-0000-7000-8003-000000000011','019ca1eb-21c0-7000-8002-000000000002','Organizations'),
('019ca1ed-0000-7000-8003-000000000012','019ca1eb-21c0-7000-8001-000000000001','กำหนดสาขา / แผนก'),
('019ca1ed-0000-7000-8003-000000000012','019ca1eb-21c0-7000-8002-000000000002','Branches & Departments'),
('019ca1ed-0000-7000-8003-000000000013','019ca1eb-21c0-7000-8001-000000000001','ทะเบียนพนักงาน'),
('019ca1ed-0000-7000-8003-000000000013','019ca1eb-21c0-7000-8002-000000000002','Employees'),
('019ca1ed-0000-7000-8003-000000000014','019ca1eb-21c0-7000-8001-000000000001','ข้อมูลอ้างอิงระบบ'),
('019ca1ed-0000-7000-8003-000000000014','019ca1eb-21c0-7000-8002-000000000002','Reference Data'),
('019ca1ed-0000-7000-8003-000000000015','019ca1eb-21c0-7000-8001-000000000001','จังหวัด / อำเภอ / ตำบล'),
('019ca1ed-0000-7000-8003-000000000015','019ca1eb-21c0-7000-8002-000000000002','Geography'),
('019ca1ee-0000-7000-8003-000000000001','019ca1eb-21c0-7000-8001-000000000001','ข้อความแปลภาษา'),
('019ca1ee-0000-7000-8003-000000000001','019ca1eb-21c0-7000-8002-000000000002','i18n Labels'),
('019ca1ed-0000-7000-8003-000000000020','019ca1eb-21c0-7000-8001-000000000001','กำหนดสิทธิ์'),
('019ca1ed-0000-7000-8003-000000000020','019ca1eb-21c0-7000-8002-000000000002','Security'),
('019ca1ed-0000-7000-8003-000000000021','019ca1eb-21c0-7000-8001-000000000001','จัดการระบบ / โมดูล'),
('019ca1ed-0000-7000-8003-000000000021','019ca1eb-21c0-7000-8002-000000000002','Systems & Modules'),
('019ca1ed-0000-7000-8003-000000000022','019ca1eb-21c0-7000-8001-000000000001','จัดการเมนู'),
('019ca1ed-0000-7000-8003-000000000022','019ca1eb-21c0-7000-8002-000000000002','Menus & Programs'),
('019ca1ed-0000-7000-8003-000000000023','019ca1eb-21c0-7000-8001-000000000001','แฟ้มสิทธิ์การเข้าใช้งาน'),
('019ca1ed-0000-7000-8003-000000000023','019ca1eb-21c0-7000-8002-000000000002','Permission Matrix'),
('019ca1ed-0000-7000-8003-000000000024','019ca1eb-21c0-7000-8001-000000000001','กำหนดผู้ใช้งาน'),
('019ca1ed-0000-7000-8003-000000000024','019ca1eb-21c0-7000-8002-000000000002','User Accounts'),
('019ca1ed-0000-7000-8003-000000000025','019ca1eb-21c0-7000-8001-000000000001','สถานะเซสชัน (IAM)'),
('019ca1ed-0000-7000-8003-000000000025','019ca1eb-21c0-7000-8002-000000000002','IAM & Session Monitor'),
('019ca1ed-0000-7000-8003-000000000030','019ca1eb-21c0-7000-8001-000000000001','ตรวจสอบ'),
('019ca1ed-0000-7000-8003-000000000030','019ca1eb-21c0-7000-8002-000000000002','Audit'),
('019ca1ed-0000-7000-8003-000000000031','019ca1eb-21c0-7000-8001-000000000001','บันทึกการตรวจสอบ'),
('019ca1ed-0000-7000-8003-000000000031','019ca1eb-21c0-7000-8002-000000000002','Audit Log'),
('019ca1ed-0000-7000-8003-000000000032','019ca1eb-21c0-7000-8001-000000000001','ติดตามกระบวนการ'),
('019ca1ed-0000-7000-8003-000000000032','019ca1eb-21c0-7000-8002-000000000002','Process Monitor'),
('019ca1ed-0000-7000-8003-000000000040','019ca1eb-21c0-7000-8001-000000000001','รายงานของฉัน'),
('019ca1ed-0000-7000-8003-000000000040','019ca1eb-21c0-7000-8002-000000000002','My Reports'),
('019ca1ed-0000-7000-8003-000000000041','019ca1eb-21c0-7000-8001-000000000001','การแจ้งเตือน'),
('019ca1ed-0000-7000-8003-000000000041','019ca1eb-21c0-7000-8002-000000000002','Notifications')
ON CONFLICT ("program_id","language_id") DO UPDATE SET "label"=EXCLUDED."label";
