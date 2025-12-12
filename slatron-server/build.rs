fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(feature = "embed-ui")]
    {
        use std::env;
        use std::fs::File;
        use std::path::Path;
        use std::process::Command;
        use walkdir::WalkDir;

        println!("cargo:rerun-if-changed=../slatron-ui/src");
        println!("cargo:rerun-if-changed=../slatron-ui/package.json");
        println!("cargo:rerun-if-changed=../slatron-ui/vite.config.ts");

        let ui_dir = Path::new("../slatron-ui");
        if !ui_dir.exists() {
            panic!("UI directory not found at ../slatron-ui");
        }

        // 1. Install dependencies
        let status = Command::new("npm")
            .arg("install")
            .current_dir(ui_dir)
            .status()
            .expect("Failed to run npm install");

        if !status.success() {
            panic!("npm install failed");
        }

        // 2. Build Frontend
        let status = Command::new("npm")
            .arg("run")
            .arg("build")
            .current_dir(ui_dir)
            .status()
            .expect("Failed to run npm run build");

        if !status.success() {
            panic!("npm run build failed");
        }

        // 3. Zip static folder (output of vite build)
        // Vite config outputs to ../slatron-server/static
        let dist_dir = env::current_dir().unwrap().join("static");

        if !dist_dir.exists() {
            panic!("Dist directory not found at {:?}", dist_dir);
        }

        let out_dir = env::var("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("ui.zip");
        let file = File::create(&dest_path).unwrap();

        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);

        let walk_dir = WalkDir::new(&dist_dir);
        // We want paths inside zip to be relative to dist_dir, e.g. "index.html"
        // But WalkDir gives full paths.
        // Actually simplest is to make paths relative to dist_dir.

        for entry in walk_dir.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path == dist_dir {
                continue;
            }
            let name = path.strip_prefix(&dist_dir).unwrap();
            let name_str = name.to_str().unwrap();

            if path.is_dir() {
                let _ = zip.add_directory(name_str, options);
            } else {
                zip.start_file(name_str, options).unwrap();
                let mut f = File::open(path).unwrap();
                std::io::copy(&mut f, &mut zip).unwrap();
            }
        }
        zip.finish().unwrap();
    }
}
