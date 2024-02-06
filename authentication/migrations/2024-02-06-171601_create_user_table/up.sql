create table "user" (
  id uuid primary key default gen_random_uuid() not null,
  email text unique,
  email_verified boolean not null default false,
  locale text,
  oauth2_id text,
  oauth2_name text,
  oauth2_picture_url text
);
