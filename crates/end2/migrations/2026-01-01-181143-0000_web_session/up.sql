create table web_session (
    id uuid default uuidv7() primary key,
    blob jsonb not null default '{}'
)