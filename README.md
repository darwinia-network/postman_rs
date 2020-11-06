### postman in rust

```bash
git clone https://github.com/darwinia-network/postman_rs.git
cd postman_rs
mkdir ~/.postman_rs
cp config.toml.example ~/.postman_rs/config.toml
# fill in the correct params
cargo install --path .
postman_rs
```