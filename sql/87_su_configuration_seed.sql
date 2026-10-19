-- Security settings in su_configuration (read by api-master: login brute-force lock, password mails).
-- Idempotent: existing values are never overwritten, so edits survive a re-seed. Ids are v7-style
-- (PostgreSQL 17 has no uuidv7()).

INSERT INTO "su_configuration" ("id", "category", "key", "value", "description")
SELECT
  encode(set_bit(set_bit(overlay(uuid_send(gen_random_uuid()) placing substring(int8send((extract(epoch FROM clock_timestamp()) * 1000)::bigint) FROM 3) FROM 1 FOR 6), 52, 1), 53, 1), 'hex')::uuid,
  t.category, t.key, t.value, t.description
FROM (VALUES
  ('security', 'security.login.max_failed_attempts', '5', 'Failed sign-ins (within the window below) after which the account is locked'),
  ('security', 'security.login.attempt_window_minutes', '15', 'Minutes in which failed sign-ins are counted'),
  ('security', 'security.login.lock_minutes', '15', 'Minutes an account stays locked after too many failed sign-ins'),
  ('mail', 'mail.smtp_url', '', 'SMTP server for password e-mails, e.g. smtp://host:1025, smtps://user:pass@host:465 or smtp://host:587?tls=required. Empty = no mail is sent (the admin gets the link instead)'),
  ('mail', 'mail.from', 'no-reply@arch.local', 'Sender of the e-mails, e.g. Arch Master <no-reply@arch.local>'),
  ('security', 'security.password.min_length', '8', 'Shortest new password (Ory Kratos refuses anything under 8; at most 128)'),
  ('security', 'security.password.require_uppercase', '1', 'A new password needs an uppercase letter (1 = yes, 0 = no)'),
  ('security', 'security.password.require_lowercase', '1', 'A new password needs a lowercase letter (1 = yes, 0 = no)'),
  ('security', 'security.password.require_digit', '1', 'A new password needs a digit (1 = yes, 0 = no)'),
  ('security', 'security.password.require_symbol', '0', 'A new password needs a symbol such as ! @ # (1 = yes, 0 = no)'),
  ('security', 'security.password.forbid_identifier', '1', 'A new password may not contain the user name or the e-mail name (1 = yes, 0 = no)'),
  ('security', 'security.password.mail_resend_minutes', '5', 'Least minutes between two password emails sent because the account tried to sign in while a password change is required')
) AS t(category, key, value, description)
ON CONFLICT ("key") DO NOTHING;
