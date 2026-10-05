\set ON_ERROR_STOP on


CREATE TABLE users (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    telegram_user_id BIGINT NOT NULL UNIQUE,
    username TEXT,
    display_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TYPE ChannelStatus AS ENUM (
    'active',
    'paused'
);

CREATE TABLE channels (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    telegram_chat_id BIGINT NOT NULL UNIQUE,
    username TEXT,
    title TEXT NOT NULL,
    status ChannelStatus NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TYPE ChannelOwnerRole AS ENUM (
    'claimer',
    'manager',
    'owner'
);

CREATE TABLE channel_owners (
    channel_id BIGINT NOT NULL
        REFERENCES channels(id),
    user_id BIGINT NOT NULL
        REFERENCES users(id),
    role ChannelOwnerRole NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (channel_id, user_id)
);

CREATE TABLE channel_pages (
    channel_id BIGINT PRIMARY KEY
        REFERENCES channels(id) ON DELETE CASCADE,
    slug TEXT NOT NULL UNIQUE,
    display_title TEXT,
    description TEXT,
    is_published BOOLEAN NOT NULL DEFAULT false,

    crawl_checkpoint_message_id BIGINT,
    last_crawl_started_at TIMESTAMPTZ,
    last_crawl_completed_at TIMESTAMPTZ,
    last_crawl_error TEXT,
    last_crawl_retry_count INT NOT NULL DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE channel_categories (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    channel_id BIGINT NOT NULL
        REFERENCES channels(id) ON DELETE CASCADE,

    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (channel_id, id),
    UNIQUE (channel_id, name)
);

CREATE TABLE channel_authors (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    channel_id BIGINT NOT NULL
        REFERENCES channels(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (channel_id, id),
    UNIQUE (channel_id, name)
);

CREATE TABLE channel_page_recordings (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    channel_id BIGINT NOT NULL
        REFERENCES channel_pages(channel_id) ON DELETE CASCADE,
    author_id BIGINT,
    category_id BIGINT,

    telegram_message_id BIGINT NOT NULL,
    telegram_remote_file_id TEXT NOT NULL
        CHECK (telegram_remote_file_id <> ''),
    title TEXT,
    caption TEXT,
    duration_seconds INTEGER
        CHECK (duration_seconds IS NULL OR duration_seconds >= 0),
    file_size_bytes BIGINT
        CHECK (file_size_bytes IS NULL OR file_size_bytes >= 0),
    storage_key TEXT,
    published_at TIMESTAMPTZ NOT NULL,
    downloaded_at TIMESTAMPTZ,
    download_attempts INTEGER NOT NULL DEFAULT 0
        CHECK (download_attempts >= 0),
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (channel_id, telegram_message_id),

    FOREIGN KEY (channel_id, author_id)
        REFERENCES channel_authors(channel_id, id),
    FOREIGN KEY (channel_id, category_id)
      REFERENCES channel_categories (channel_id, id)
);

CREATE INDEX channel_page_recordings_published_idx
    ON channel_page_recordings (channel_id, published_at DESC);
