fn main() {
    let key = include_str!("src/license_public_key.hex").trim();
    if key.len() != 64 || key.chars().all(|c| c == '0') || !key.chars().all(|c| c.is_ascii_hexdigit()) {
        panic!("Configure a chave publica antes do build: bash prepare-license-keys.sh");
    }
    tauri_build::build()
}
