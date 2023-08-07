wget -O src/schema.rs https://ldtk.io/files/quicktype/LdtkJson.rs
sed -z -i -r 's/(#\[serde\(rename\s=\s\"[a-zA-Z_]+\"\)\]\s+)([a-zA-Z_][a-zA-Z0-9_]*\:)/\1pub \2/g' src/schema.rs
