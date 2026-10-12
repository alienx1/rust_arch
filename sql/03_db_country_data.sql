-- Data for db_country and db_country_label
-- Generated with UUIDv7 (Timestamp: 2026-04-22)

-- Reference Language IDs:
-- th: 019ca1eb-21c0-7000-8001-000000000001
-- en: 019ca1eb-21c0-7000-8002-000000000002

-- Country: Thailand (THA)
INSERT INTO "db_country" ("id", "code", "active") VALUES
('019ca1eb-2a30-7000-8000-000000000066', 'THA', true);

INSERT INTO "db_country_label" ("country_id", "language_id", "label") VALUES
('019ca1eb-2a30-7000-8000-000000000066', '019ca1eb-21c0-7000-8001-000000000001', 'ประเทศไทย'),
('019ca1eb-2a30-7000-8000-000000000066', '019ca1eb-21c0-7000-8002-000000000002', 'Thailand');
