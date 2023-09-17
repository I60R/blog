# Build instruction

1. Install `MySql` and launch with `systemctl start mysql`
2. Create user `user` with password `user`
3. Execute `source .env`
4. Install `sqlx-cli`
5. Execute `sqlx database create`
6. Execute `sqlx migrate run`
7. Execute `cargo run`
