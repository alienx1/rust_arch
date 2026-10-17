-- Password e-mail templates in su_configuration (category mail_template), read by api-master on
-- every send (through the cache for a minute). Placeholders: {{username}} {{link}} {{hours}} {{app_name}}
-- (values are HTML-escaped in the .html templates). Setup / reset / forced / required each have a
-- subject, an HTML and a plain-text body; an empty or missing row falls back to the built-in text.
-- Idempotent: existing values are never overwritten, so edits survive a re-seed. Ids are v7-style
-- (PostgreSQL 17 has no uuidv7()).

INSERT INTO "su_configuration" ("id", "category", "key", "value", "description")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  t.category, t.key, t.value, t.description
FROM (VALUES
  ('mail_template', 'mail.template.app_name', $tpl$Arch$tpl$, $tpl$Name shown in the header of the password e-mails$tpl$),
  ('mail_template', 'mail.template.setup.subject', $tpl$ตั้งรหัสผ่านบัญชีของคุณ / Set your password$tpl$, $tpl$Subject of the 'setup' password e-mail$tpl$),
  ('mail_template', 'mail.template.setup.html', $tpl$<!DOCTYPE html>
<html lang="th"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ตั้งรหัสผ่านบัญชีของคุณ / Set your password</title></head>
<body style="margin:0;padding:0;background:#f3f4f6;font-family:'Segoe UI',Tahoma,'Noto Sans Thai',Arial,sans-serif;color:#111827;">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background:#f3f4f6;padding:24px 12px;"><tr><td align="center">
<table role="presentation" width="560" cellpadding="0" cellspacing="0" style="max-width:560px;width:100%;background:#ffffff;border-radius:12px;overflow:hidden;border:1px solid #e5e7eb;">
<tr><td style="background:#4f46e5;padding:20px 28px;color:#ffffff;font-size:18px;font-weight:600;letter-spacing:.3px;">{{app_name}}</td></tr>
<tr><td style="padding:28px 28px 8px;">
<h1 style="margin:0 0 10px;font-size:20px;line-height:1.3;color:#111827;">ยินดีต้อนรับสู่ {{app_name}}</h1>
<p style="margin:0 0 16px;font-size:15px;line-height:1.6;color:#374151;">บัญชีผู้ใช้ของคุณถูกสร้างเรียบร้อยแล้ว กรุณาตั้งรหัสผ่านก่อนเข้าใช้งานครั้งแรก</p>
<p style="margin:0 0 20px;font-size:14px;color:#374151;">ชื่อผู้ใช้ <span style="display:inline-block;padding:2px 8px;background:#eef2ff;border-radius:6px;font-family:Consolas,monospace;color:#3730a3;"><b>{{username}}</b></span></p>
<table role="presentation" cellpadding="0" cellspacing="0"><tr><td style="border-radius:8px;background:#4f46e5;"><a href="{{link}}" style="display:inline-block;padding:12px 26px;font-size:15px;font-weight:600;color:#ffffff;text-decoration:none;">ตั้งรหัสผ่าน</a></td></tr></table>
<p style="margin:14px 0 0;font-size:12.5px;color:#6b7280;">ลิงก์ใช้ได้ {{hours}} ชั่วโมง และใช้ได้เพียงครั้งเดียว</p>
</td></tr>
<tr><td style="padding:0 28px;"><hr style="border:none;border-top:1px solid #e5e7eb;margin:22px 0;"></td></tr>
<tr><td style="padding:0 28px 8px;">
<h2 style="margin:0 0 8px;font-size:16px;color:#111827;">Welcome to {{app_name}}</h2>
<p style="margin:0 0 14px;font-size:14px;line-height:1.6;color:#374151;">Your account has been created. Please set your password before you sign in for the first time.</p>
<p style="margin:0 0 6px;font-size:14px;color:#374151;">Username <span style="display:inline-block;padding:2px 8px;background:#eef2ff;border-radius:6px;font-family:Consolas,monospace;color:#3730a3;"><b>{{username}}</b></span></p>
<p style="margin:0;font-size:12.5px;color:#6b7280;">The link is valid for {{hours}} hours and works only once.</p>
</td></tr>
<tr><td style="padding:22px 28px 26px;">
<p style="margin:0 0 6px;font-size:12px;color:#6b7280;">ถ้าปุ่มกดไม่ได้ ให้คัดลอกลิงก์นี้ไปเปิดในเบราว์เซอร์ / If the button does not work, copy this link into your browser:</p>
<p style="margin:0;font-size:12px;word-break:break-all;"><a href="{{link}}" style="color:#4f46e5;">{{link}}</a></p>
</td></tr>
<tr><td style="background:#f9fafb;padding:16px 28px;font-size:12px;line-height:1.6;color:#6b7280;border-top:1px solid #e5e7eb;">
ถ้าคุณไม่ได้เป็นผู้ขอ โปรดเพิกเฉยต่ออีเมลนี้ และอย่าส่งต่อลิงก์ให้ผู้อื่น<br>If you did not expect this e-mail, ignore it and do not share the link with anyone.
</td></tr>
</table></td></tr></table></body></html>$tpl$, $tpl$HTML body of the 'setup' password e-mail. Placeholders: {{username}} {{link}} {{hours}} {{app_name}}$tpl$),
  ('mail_template', 'mail.template.setup.text', $tpl${{app_name}}

ยินดีต้อนรับสู่ {{app_name}}
บัญชีผู้ใช้ของคุณถูกสร้างเรียบร้อยแล้ว กรุณาตั้งรหัสผ่านก่อนเข้าใช้งานครั้งแรก

ชื่อผู้ใช้: {{username}}
ตั้งรหัสผ่าน (ลิงก์ใช้ได้ {{hours}} ชั่วโมง และใช้ได้เพียงครั้งเดียว):
{{link}}

ถ้าคุณไม่ได้เป็นผู้ขอ โปรดเพิกเฉยต่ออีเมลนี้ และอย่าส่งต่อลิงก์ให้ผู้อื่น

----------------------------------------

Welcome to {{app_name}}
Your account has been created. Please set your password before you sign in for the first time.

Username: {{username}}
Set your password (valid for {{hours}} hours, works only once):
{{link}}

If you did not expect this e-mail, ignore it and do not share the link with anyone.
$tpl$, $tpl$Plain-text body of the 'setup' password e-mail (same placeholders)$tpl$),
  ('mail_template', 'mail.template.reset.subject', $tpl$ตั้งรหัสผ่านใหม่ / Set a new password$tpl$, $tpl$Subject of the 'reset' password e-mail$tpl$),
  ('mail_template', 'mail.template.reset.html', $tpl$<!DOCTYPE html>
<html lang="th"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ตั้งรหัสผ่านใหม่ / Set a new password</title></head>
<body style="margin:0;padding:0;background:#f3f4f6;font-family:'Segoe UI',Tahoma,'Noto Sans Thai',Arial,sans-serif;color:#111827;">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background:#f3f4f6;padding:24px 12px;"><tr><td align="center">
<table role="presentation" width="560" cellpadding="0" cellspacing="0" style="max-width:560px;width:100%;background:#ffffff;border-radius:12px;overflow:hidden;border:1px solid #e5e7eb;">
<tr><td style="background:#4f46e5;padding:20px 28px;color:#ffffff;font-size:18px;font-weight:600;letter-spacing:.3px;">{{app_name}}</td></tr>
<tr><td style="padding:28px 28px 8px;">
<h1 style="margin:0 0 10px;font-size:20px;line-height:1.3;color:#111827;">ตั้งรหัสผ่านใหม่</h1>
<p style="margin:0 0 16px;font-size:15px;line-height:1.6;color:#374151;">ผู้ดูแลระบบส่งลิงก์ให้คุณตั้งรหัสผ่านใหม่ รหัสผ่านเดิมยังใช้งานได้จนกว่าคุณจะตั้งรหัสผ่านใหม่</p>
<p style="margin:0 0 20px;font-size:14px;color:#374151;">ชื่อผู้ใช้ <span style="display:inline-block;padding:2px 8px;background:#eef2ff;border-radius:6px;font-family:Consolas,monospace;color:#3730a3;"><b>{{username}}</b></span></p>
<table role="presentation" cellpadding="0" cellspacing="0"><tr><td style="border-radius:8px;background:#4f46e5;"><a href="{{link}}" style="display:inline-block;padding:12px 26px;font-size:15px;font-weight:600;color:#ffffff;text-decoration:none;">ตั้งรหัสผ่านใหม่</a></td></tr></table>
<p style="margin:14px 0 0;font-size:12.5px;color:#6b7280;">ลิงก์ใช้ได้ {{hours}} ชั่วโมง และใช้ได้เพียงครั้งเดียว</p>
</td></tr>
<tr><td style="padding:0 28px;"><hr style="border:none;border-top:1px solid #e5e7eb;margin:22px 0;"></td></tr>
<tr><td style="padding:0 28px 8px;">
<h2 style="margin:0 0 8px;font-size:16px;color:#111827;">Choose a new password</h2>
<p style="margin:0 0 14px;font-size:14px;line-height:1.6;color:#374151;">An administrator sent you a link to choose a new password. Your current password keeps working until you set a new one.</p>
<p style="margin:0 0 6px;font-size:14px;color:#374151;">Username <span style="display:inline-block;padding:2px 8px;background:#eef2ff;border-radius:6px;font-family:Consolas,monospace;color:#3730a3;"><b>{{username}}</b></span></p>
<p style="margin:0;font-size:12.5px;color:#6b7280;">The link is valid for {{hours}} hours and works only once.</p>
</td></tr>
<tr><td style="padding:22px 28px 26px;">
<p style="margin:0 0 6px;font-size:12px;color:#6b7280;">ถ้าปุ่มกดไม่ได้ ให้คัดลอกลิงก์นี้ไปเปิดในเบราว์เซอร์ / If the button does not work, copy this link into your browser:</p>
<p style="margin:0;font-size:12px;word-break:break-all;"><a href="{{link}}" style="color:#4f46e5;">{{link}}</a></p>
</td></tr>
<tr><td style="background:#f9fafb;padding:16px 28px;font-size:12px;line-height:1.6;color:#6b7280;border-top:1px solid #e5e7eb;">
ถ้าคุณไม่ได้เป็นผู้ขอ โปรดเพิกเฉยต่ออีเมลนี้ และอย่าส่งต่อลิงก์ให้ผู้อื่น<br>If you did not expect this e-mail, ignore it and do not share the link with anyone.
</td></tr>
</table></td></tr></table></body></html>$tpl$, $tpl$HTML body of the 'reset' password e-mail. Placeholders: {{username}} {{link}} {{hours}} {{app_name}}$tpl$),
  ('mail_template', 'mail.template.reset.text', $tpl${{app_name}}

ตั้งรหัสผ่านใหม่
ผู้ดูแลระบบส่งลิงก์ให้คุณตั้งรหัสผ่านใหม่ รหัสผ่านเดิมยังใช้งานได้จนกว่าคุณจะตั้งรหัสผ่านใหม่

ชื่อผู้ใช้: {{username}}
ตั้งรหัสผ่านใหม่ (ลิงก์ใช้ได้ {{hours}} ชั่วโมง และใช้ได้เพียงครั้งเดียว):
{{link}}

ถ้าคุณไม่ได้เป็นผู้ขอ โปรดเพิกเฉยต่ออีเมลนี้ และอย่าส่งต่อลิงก์ให้ผู้อื่น

----------------------------------------

Choose a new password
An administrator sent you a link to choose a new password. Your current password keeps working until you set a new one.

Username: {{username}}
Choose a new password (valid for {{hours}} hours, works only once):
{{link}}

If you did not expect this e-mail, ignore it and do not share the link with anyone.
$tpl$, $tpl$Plain-text body of the 'reset' password e-mail (same placeholders)$tpl$),
  ('mail_template', 'mail.template.forced.subject', $tpl$ต้องเปลี่ยนรหัสผ่านก่อนเข้าใช้งาน / Password change required$tpl$, $tpl$Subject of the 'forced' password e-mail$tpl$),
  ('mail_template', 'mail.template.forced.html', $tpl$<!DOCTYPE html>
<html lang="th"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ต้องเปลี่ยนรหัสผ่านก่อนเข้าใช้งาน / Password change required</title></head>
<body style="margin:0;padding:0;background:#f3f4f6;font-family:'Segoe UI',Tahoma,'Noto Sans Thai',Arial,sans-serif;color:#111827;">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background:#f3f4f6;padding:24px 12px;"><tr><td align="center">
<table role="presentation" width="560" cellpadding="0" cellspacing="0" style="max-width:560px;width:100%;background:#ffffff;border-radius:12px;overflow:hidden;border:1px solid #e5e7eb;">
<tr><td style="background:#4f46e5;padding:20px 28px;color:#ffffff;font-size:18px;font-weight:600;letter-spacing:.3px;">{{app_name}}</td></tr>
<tr><td style="padding:28px 28px 8px;">
<h1 style="margin:0 0 10px;font-size:20px;line-height:1.3;color:#111827;">ต้องเปลี่ยนรหัสผ่าน</h1>
<p style="margin:0 0 16px;font-size:15px;line-height:1.6;color:#374151;">ผู้ดูแลระบบกำหนดให้คุณเปลี่ยนรหัสผ่าน คุณออกจากระบบทุกอุปกรณ์แล้ว และจะเข้าใช้งานไม่ได้จนกว่าจะตั้งรหัสผ่านใหม่</p>
<p style="margin:0 0 20px;font-size:14px;color:#374151;">ชื่อผู้ใช้ <span style="display:inline-block;padding:2px 8px;background:#eef2ff;border-radius:6px;font-family:Consolas,monospace;color:#3730a3;"><b>{{username}}</b></span></p>
<table role="presentation" cellpadding="0" cellspacing="0"><tr><td style="border-radius:8px;background:#4f46e5;"><a href="{{link}}" style="display:inline-block;padding:12px 26px;font-size:15px;font-weight:600;color:#ffffff;text-decoration:none;">ตั้งรหัสผ่านใหม่</a></td></tr></table>
<p style="margin:14px 0 0;font-size:12.5px;color:#6b7280;">ลิงก์ใช้ได้ {{hours}} ชั่วโมง และใช้ได้เพียงครั้งเดียว</p>
</td></tr>
<tr><td style="padding:0 28px;"><hr style="border:none;border-top:1px solid #e5e7eb;margin:22px 0;"></td></tr>
<tr><td style="padding:0 28px 8px;">
<h2 style="margin:0 0 8px;font-size:16px;color:#111827;">Password change required</h2>
<p style="margin:0 0 14px;font-size:14px;line-height:1.6;color:#374151;">An administrator requires you to change your password. You have been signed out everywhere and cannot sign in until you set a new one.</p>
<p style="margin:0 0 6px;font-size:14px;color:#374151;">Username <span style="display:inline-block;padding:2px 8px;background:#eef2ff;border-radius:6px;font-family:Consolas,monospace;color:#3730a3;"><b>{{username}}</b></span></p>
<p style="margin:0;font-size:12.5px;color:#6b7280;">The link is valid for {{hours}} hours and works only once.</p>
</td></tr>
<tr><td style="padding:22px 28px 26px;">
<p style="margin:0 0 6px;font-size:12px;color:#6b7280;">ถ้าปุ่มกดไม่ได้ ให้คัดลอกลิงก์นี้ไปเปิดในเบราว์เซอร์ / If the button does not work, copy this link into your browser:</p>
<p style="margin:0;font-size:12px;word-break:break-all;"><a href="{{link}}" style="color:#4f46e5;">{{link}}</a></p>
</td></tr>
<tr><td style="background:#f9fafb;padding:16px 28px;font-size:12px;line-height:1.6;color:#6b7280;border-top:1px solid #e5e7eb;">
ถ้าคุณไม่ได้เป็นผู้ขอ โปรดเพิกเฉยต่ออีเมลนี้ และอย่าส่งต่อลิงก์ให้ผู้อื่น<br>If you did not expect this e-mail, ignore it and do not share the link with anyone.
</td></tr>
</table></td></tr></table></body></html>$tpl$, $tpl$HTML body of the 'forced' password e-mail. Placeholders: {{username}} {{link}} {{hours}} {{app_name}}$tpl$),
  ('mail_template', 'mail.template.forced.text', $tpl${{app_name}}

ต้องเปลี่ยนรหัสผ่าน
ผู้ดูแลระบบกำหนดให้คุณเปลี่ยนรหัสผ่าน คุณออกจากระบบทุกอุปกรณ์แล้ว และจะเข้าใช้งานไม่ได้จนกว่าจะตั้งรหัสผ่านใหม่

ชื่อผู้ใช้: {{username}}
ตั้งรหัสผ่านใหม่ (ลิงก์ใช้ได้ {{hours}} ชั่วโมง และใช้ได้เพียงครั้งเดียว):
{{link}}

ถ้าคุณไม่ได้เป็นผู้ขอ โปรดเพิกเฉยต่ออีเมลนี้ และอย่าส่งต่อลิงก์ให้ผู้อื่น

----------------------------------------

Password change required
An administrator requires you to change your password. You have been signed out everywhere and cannot sign in until you set a new one.

Username: {{username}}
Set a new password (valid for {{hours}} hours, works only once):
{{link}}

If you did not expect this e-mail, ignore it and do not share the link with anyone.
$tpl$, $tpl$Plain-text body of the 'forced' password e-mail (same placeholders)$tpl$),
  ('mail_template', 'mail.template.required.subject', $tpl$ต้องเปลี่ยนรหัสผ่านก่อนเข้าใช้งาน / Password change required$tpl$, $tpl$Subject of the 'required' password e-mail$tpl$),
  ('mail_template', 'mail.template.required.html', $tpl$<!DOCTYPE html>
<html lang="th"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ต้องเปลี่ยนรหัสผ่านก่อนเข้าใช้งาน / Password change required</title></head>
<body style="margin:0;padding:0;background:#f3f4f6;font-family:'Segoe UI',Tahoma,'Noto Sans Thai',Arial,sans-serif;color:#111827;">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background:#f3f4f6;padding:24px 12px;"><tr><td align="center">
<table role="presentation" width="560" cellpadding="0" cellspacing="0" style="max-width:560px;width:100%;background:#ffffff;border-radius:12px;overflow:hidden;border:1px solid #e5e7eb;">
<tr><td style="background:#4f46e5;padding:20px 28px;color:#ffffff;font-size:18px;font-weight:600;letter-spacing:.3px;">{{app_name}}</td></tr>
<tr><td style="padding:28px 28px 8px;">
<h1 style="margin:0 0 10px;font-size:20px;line-height:1.3;color:#111827;">ลิงก์ตั้งรหัสผ่านใหม่</h1>
<p style="margin:0 0 16px;font-size:15px;line-height:1.6;color:#374151;">คุณต้องตั้งรหัสผ่านใหม่ก่อนจึงจะเข้าใช้งานได้ นี่คือลิงก์ใหม่สำหรับตั้งรหัสผ่าน</p>
<p style="margin:0 0 20px;font-size:14px;color:#374151;">ชื่อผู้ใช้ <span style="display:inline-block;padding:2px 8px;background:#eef2ff;border-radius:6px;font-family:Consolas,monospace;color:#3730a3;"><b>{{username}}</b></span></p>
<table role="presentation" cellpadding="0" cellspacing="0"><tr><td style="border-radius:8px;background:#4f46e5;"><a href="{{link}}" style="display:inline-block;padding:12px 26px;font-size:15px;font-weight:600;color:#ffffff;text-decoration:none;">ตั้งรหัสผ่านใหม่</a></td></tr></table>
<p style="margin:14px 0 0;font-size:12.5px;color:#6b7280;">ลิงก์ใช้ได้ {{hours}} ชั่วโมง และใช้ได้เพียงครั้งเดียว</p>
</td></tr>
<tr><td style="padding:0 28px;"><hr style="border:none;border-top:1px solid #e5e7eb;margin:22px 0;"></td></tr>
<tr><td style="padding:0 28px 8px;">
<h2 style="margin:0 0 8px;font-size:16px;color:#111827;">Your new password link</h2>
<p style="margin:0 0 14px;font-size:14px;line-height:1.6;color:#374151;">You must set a new password before you can sign in. Here is a new link to do it.</p>
<p style="margin:0 0 6px;font-size:14px;color:#374151;">Username <span style="display:inline-block;padding:2px 8px;background:#eef2ff;border-radius:6px;font-family:Consolas,monospace;color:#3730a3;"><b>{{username}}</b></span></p>
<p style="margin:0;font-size:12.5px;color:#6b7280;">The link is valid for {{hours}} hours and works only once.</p>
</td></tr>
<tr><td style="padding:22px 28px 26px;">
<p style="margin:0 0 6px;font-size:12px;color:#6b7280;">ถ้าปุ่มกดไม่ได้ ให้คัดลอกลิงก์นี้ไปเปิดในเบราว์เซอร์ / If the button does not work, copy this link into your browser:</p>
<p style="margin:0;font-size:12px;word-break:break-all;"><a href="{{link}}" style="color:#4f46e5;">{{link}}</a></p>
</td></tr>
<tr><td style="background:#f9fafb;padding:16px 28px;font-size:12px;line-height:1.6;color:#6b7280;border-top:1px solid #e5e7eb;">
ถ้าคุณไม่ได้เป็นผู้ขอ โปรดเพิกเฉยต่ออีเมลนี้ และอย่าส่งต่อลิงก์ให้ผู้อื่น<br>If you did not expect this e-mail, ignore it and do not share the link with anyone.
</td></tr>
</table></td></tr></table></body></html>$tpl$, $tpl$HTML body of the 'required' password e-mail. Placeholders: {{username}} {{link}} {{hours}} {{app_name}}$tpl$),
  ('mail_template', 'mail.template.required.text', $tpl${{app_name}}

ลิงก์ตั้งรหัสผ่านใหม่
คุณต้องตั้งรหัสผ่านใหม่ก่อนจึงจะเข้าใช้งานได้ นี่คือลิงก์ใหม่สำหรับตั้งรหัสผ่าน

ชื่อผู้ใช้: {{username}}
ตั้งรหัสผ่านใหม่ (ลิงก์ใช้ได้ {{hours}} ชั่วโมง และใช้ได้เพียงครั้งเดียว):
{{link}}

ถ้าคุณไม่ได้เป็นผู้ขอ โปรดเพิกเฉยต่ออีเมลนี้ และอย่าส่งต่อลิงก์ให้ผู้อื่น

----------------------------------------

Your new password link
You must set a new password before you can sign in. Here is a new link to do it.

Username: {{username}}
Set a new password (valid for {{hours}} hours, works only once):
{{link}}

If you did not expect this e-mail, ignore it and do not share the link with anyone.
$tpl$, $tpl$Plain-text body of the 'required' password e-mail (same placeholders)$tpl$)
) AS t(category, key, value, description)
ON CONFLICT ("key") DO NOTHING;
