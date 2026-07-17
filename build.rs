use std::{env, fs, path::Path};

fn sanitise_icon_name(name: &str) -> String {
    if let Some(name) = name.strip_prefix('1') {
        format!("onek{name}")
    } else if let Some(name) = name.strip_prefix('2') {
        format!("twok{name}")
    } else if let Some(name) = name.strip_prefix('3') {
        format!("threek{name}")
    } else if let Some(name) = name.strip_prefix('4') {
        format!("fourk{name}")
    } else if let Some(name) = name.strip_prefix('5') {
        format!("fivek{name}")
    } else if let Some(name) = name.strip_prefix('6') {
        format!("sixk{name}")
    } else if let Some(name) = name.strip_prefix('7') {
        format!("sevenk{name}")
    } else if let Some(name) = name.strip_prefix('8') {
        format!("eightk{name}")
    } else if let Some(name) = name.strip_prefix('9') {
        format!("ninek{name}")
    } else {
        match name {
            "loop" => "looped".to_string(),
            "try" => "tried".to_string(),
            name => name.to_string(),
        }
    }
}

fn generate_material_icons_style(name: &str) {
    let source_path = Path::new("./src/assets/material-design-icons/svg").join(name);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(name).with_extension("rs");

    let mut code = String::new();

    for entry in fs::read_dir(source_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let include_path = path
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("\\", "\\\\");

        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let func_name = sanitise_icon_name(file_stem);

        code.push_str(&format!(
            "crate::generate_svg!({func_name}, \"{include_path}\");\n",
        ));
    }

    fs::write(dest_path, code).unwrap();
}

fn main() {
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-ObjC");
    };

    generate_material_icons_style("filled");
    generate_material_icons_style("outlined");
    generate_material_icons_style("round");
}
