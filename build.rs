// Ensure web-ui/dist/ exists for rust-embed at compile time.
// In CI or fresh checkouts where `npm run build` hasn't been run yet,
// we create a minimal placeholder so the derive macro doesn't fail.
use std::fs;
use std::path::Path;

fn main() {
    let dist = Path::new("web-ui/dist");
    if !dist.exists() {
        fs::create_dir_all(dist.join("assets")).unwrap();
        fs::write(
            dist.join("index.html"),
            r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"><title>Folio</title></head>
<body><p>Web UI not built. Run <code>cd web-ui &amp;&amp; npm install &amp;&amp; npx vite build</code></p></body>
</html>"#,
        )
        .unwrap();
    }
    println!("cargo:rerun-if-changed=web-ui/dist");
}
