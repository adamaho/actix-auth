create extension if not exists "uuid-ossp";

create table keys (
  id uuid primary key
);

create table users (
  id serial primary key,
  email varchar(40) not null unique,
  password varchar(100) not null,
  key_id uuid not null unique references keys(id),
  created_at timestamp not null default current_timestamp
);

insert into keys (id) values(uuid_generate_v4());


-- create table scoreboards (
--   id serial primary key,
--   name varchar(40) not null,
--   description text not null
-- );

-- create table scoreboards_owners (
--   scoreboard_id integer not null references scoreboards(id) on delete cascade,
--   user_id uuid not null references users(id) on delete cascade,
--   owner boolean not null default false,
--   unique(scoreboard_id, user_id, owner),
--   primary key (user_id, scoreboard_id)
-- );

-- create table scores (
--   id serial primary key,
--   scoreboard_id integer not null references scoreboards(id) on delete cascade,
--   user_id uuid not null references users(id) on delete cascade,
--   score integer not null default 0,
--   unique (user_id, scoreboard_id)
-- );