create table "user" (
  "id" uuid primary key not null default gen_random_uuid(),
  "name" text
);

create table "auth_provider" (
  "id" uuid primary key not null default gen_random_uuid(),
  "user_id" uuid not null references "user"(id),
  "order" smallint not null,
  "provider_type" text not null,
  "provider_id" text not null,
  "email" text,
  "email_verified" boolean not null default false,
  "display_name" text,
  "user_name" text,
  "picture_url" text,
  "locale" text
);

create table "refresh_session" (
  "id" uuid primary key not null default gen_random_uuid(),
  "user_id" uuid unique not null references "user"(id),
  "issued_at" timestamptz not null,
  "expires_at" timestamptz not null,
  "count" bigint not null default 0,
  "invalidated" boolean not null default false
);
