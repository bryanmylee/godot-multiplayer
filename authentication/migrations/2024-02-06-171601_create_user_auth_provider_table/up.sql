create table "user" (
  "id" uuid primary key default gen_random_uuid() not null,
  "name" text
);

create table "auth_provider" (
  "id" uuid primary key default gen_random_uuid() not null,
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
