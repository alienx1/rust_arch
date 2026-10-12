-- Data for db_language and db_language_label
-- Generated with UUIDv7 (Timestamp: 2026-04-22)

-- Languages: th (Thai), en (English)

INSERT INTO "db_language" ("id", "code", "active") VALUES
('019ca1eb-21c0-7000-8001-000000000001', 'th', true),
('019ca1eb-21c0-7000-8002-000000000002', 'en', true);

INSERT INTO "db_language_label" ("language_id", "display_language_id", "label") VALUES
-- Labels for Thai language
('019ca1eb-21c0-7000-8001-000000000001', '019ca1eb-21c0-7000-8001-000000000001', 'ไทย'),
('019ca1eb-21c0-7000-8001-000000000001', '019ca1eb-21c0-7000-8002-000000000002', 'Thai'),
-- Labels for English language
('019ca1eb-21c0-7000-8002-000000000002', '019ca1eb-21c0-7000-8001-000000000001', 'อังกฤษ'),
('019ca1eb-21c0-7000-8002-000000000002', '019ca1eb-21c0-7000-8002-000000000002', 'English');
