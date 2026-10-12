-- API error message templates (db_label, category 'error'; key = the API error code).
--
-- The api-master error-localization middleware replaces the English `message` of an error body
-- ({success:false, code, message}) with the template error.<code> in the Accept-Language language
-- (th | en, default th). `{detail}` is replaced by the original English message; templates for
-- INTERNAL_ERROR / DATABASE_ERROR / *_PROVIDER_* deliberately omit it so internals are never shown.
--
-- Ids are v7-style uuids generated here (PostgreSQL 17 has no uuidv7()); the statement is idempotent
-- (ON CONFLICT (category, key, language_id) DO NOTHING). Other db_label rows are not touched.

INSERT INTO "db_label" ("id", "language_id", "category", "key", "value")
SELECT
    encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp())*1000)::bigint) FROM 3) FROM 1 FOR 6),52,1),53,1),'hex')::uuid,
    l.id,
    'error',
    t.key,
    CASE l.code WHEN 'th' THEN t.th ELSE t.en END
FROM (VALUES
    ('NOT_FOUND',               'ไม่พบข้อมูล ({detail})',                                                  'Not found ({detail})'),
    ('VALIDATION_ERROR',        'ข้อมูลไม่ถูกต้อง ({detail})',                                              'Invalid data ({detail})'),
    ('IN_USE',                  'ลบไม่ได้: ยังมีข้อมูลอื่นอ้างอิงอยู่ — ให้ปิดใช้งานแทน ({detail})',          'Cannot delete: other records still reference it — switch it off instead ({detail})'),
    ('DUPLICATE_CODE',          'รหัสนี้ถูกใช้แล้ว ({detail})',                                              'This code is already in use ({detail})'),
    ('DUPLICATE',               'ข้อมูลซ้ำ ({detail})',                                                     'Duplicate entry ({detail})'),
    ('VERSION_CONFLICT',        'ข้อมูลถูกแก้ไขโดยผู้อื่นแล้ว กรุณาโหลดใหม่แล้วลองอีกครั้ง',                  'The record was changed by someone else. Reload it and try again'),
    ('NOT_ALLOWED',             'ไม่อนุญาตให้ทำรายการนี้ ({detail})',                                        'This action is not allowed ({detail})'),
    ('FORBIDDEN',               'คุณไม่มีสิทธิ์ทำรายการนี้ ({detail})',                                      'You do not have permission to do this ({detail})'),
    ('UNAUTHORIZED',            'กรุณาเข้าสู่ระบบ',                                                          'Please sign in'),
    ('USER_NOT_FOUND',          'ไม่พบผู้ใช้ ({detail})',                                                   'User not found ({detail})'),
    ('LANGUAGE_NOT_FOUND',      'ไม่พบภาษา ({detail})',                                                     'Language not found ({detail})'),
    ('LABEL_NOT_FOUND',         'ไม่พบข้อความแปลภาษา ({detail})',                                            'Label not found ({detail})'),
    ('IDENTITY_PROVIDER_ERROR', 'ระบบยืนยันตัวตนขัดข้อง กรุณาลองใหม่ภายหลัง',                                'The identity provider is unavailable. Please try again later'),
    ('KRATOS_ERROR',            'ระบบยืนยันตัวตนขัดข้อง กรุณาลองใหม่ภายหลัง',                                'The identity store is unavailable. Please try again later'),
    ('DATABASE_ERROR',          'ฐานข้อมูลขัดข้อง กรุณาลองใหม่ภายหลัง',                                      'A database error occurred. Please try again later'),
    ('INTERNAL_ERROR',          'เกิดข้อผิดพลาดภายในระบบ กรุณาลองใหม่ภายหลัง',                                'An internal error occurred. Please try again later'),
    ('BAD_REQUEST',             'คำขอไม่ถูกต้อง ({detail})',                                                'Bad request ({detail})'),
    ('UNPROCESSABLE_ENTITY',    'ข้อมูลที่ส่งมาไม่ถูกต้อง ({detail})',                                        'The submitted data is invalid ({detail})'),
    ('UNSUPPORTED_MEDIA_TYPE',  'ไม่รองรับรูปแบบข้อมูลที่ส่งมา',                                             'Unsupported content type'),
    ('METHOD_NOT_ALLOWED',      'ไม่รองรับวิธีเรียกนี้',                                                     'Method not allowed'),
    ('PAYLOAD_TOO_LARGE',       'ข้อมูลที่ส่งมามีขนาดใหญ่เกินไป',                                            'The request is too large'),
    ('SERVICE_UNAVAILABLE',     'ระบบยังไม่พร้อมให้บริการ กรุณาลองใหม่ภายหลัง',                              'The service is unavailable. Please try again later')
) AS t(key, th, en)
CROSS JOIN "db_language" l
WHERE l.code IN ('th', 'en')
ON CONFLICT ("category", "key", "language_id") DO NOTHING;
