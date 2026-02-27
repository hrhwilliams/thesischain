create table miner (
    id uuid default uuidv7() primary key,
    multiaddr text not null
)