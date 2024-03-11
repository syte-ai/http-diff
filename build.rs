use clap::CommandFactory;
use clap::ValueEnum;
use clap_complete::generate_to;
use clap_complete::Shell;
use clap_mangen::Man;
use std::io::Write;
use std::{
    env,
    fs::File,
    io::Error,
    path::{Path, PathBuf},
};

include!("src/cli/arguments.rs");

struct PackageMeta {
    name: String,
    version: String,
    description: String,
    authors: String,
}

impl PackageMeta {
    pub fn try_new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            name: env::var("CARGO_PKG_NAME")?,
            version: env::var("CARGO_PKG_VERSION")?,
            description: env::var("CARGO_PKG_DESCRIPTION")?,
            authors: env::var("CARGO_PKG_AUTHORS")?,
        })
    }
}

fn build_shell_completion(
    outdir: &Path,
    package_meta: &PackageMeta,
) -> Result<(), Error> {
    let mut cmd = Arguments::command();

    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, &package_meta.name, outdir)?;
    }

    Ok(())
}

fn build_manpages(
    outdir: &Path,
    package_meta: &PackageMeta,
) -> Result<(), Error> {
    let app = Arguments::command();

    let file = Path::new(&outdir).join(format!("{}.1", package_meta.name));
    let mut file = File::create(&file)?;

    Man::new(app).render(&mut file)?;

    Ok(())
}

fn build_control_file(
    outdir: &Path,
    package_meta: &PackageMeta,
) -> Result<(), Error> {
    let file_content = format!(
        "Package: {}\n\
         Version: {}\n\
         Architecture: amd64\n\
         Maintainer: {}\n\
         Description: {}\n\
         ",
        package_meta.name,
        package_meta.version,
        package_meta.authors,
        package_meta.description,
    );

    let mut file_path = PathBuf::from(outdir);
    file_path.push("controll");

    let mut file = File::create(&file_path)?;

    file.write_all(file_content.as_bytes())?;

    file.flush()?;

    Ok(())
}

fn build_desktop_file(
    outdir: &Path,
    package_meta: &PackageMeta,
) -> Result<(), Error> {
    let file_content = format!(
        "[Desktop Entry]\n\
        Name={}\n\
        Comment={}\n\
        Exec=/usr/bin/{}\n\
        Icon={}\n\
        Terminal=true\n\
        Type=Application\n\
        Encoding=UTF-8\n\
        Categories=Network;Application;\n\
        Name[en_US]={}\n\
        ",
        package_meta.name,
        package_meta.description,
        package_meta.name,
        package_meta.name,
        package_meta.name,
    );

    let mut file_path = PathBuf::from(outdir);
    file_path.push(format!("{}.desktop", package_meta.name));

    let mut file = File::create(&file_path)?;

    file.write_all(file_content.as_bytes())?;

    file.flush()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/cli/arguments.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let package_meta = PackageMeta::try_new()?;

    let outdir = match env::var_os("OUT_DIR") {
        None => {
            println!("cargo:warning=OUT_DIR variable was not found. Skipping creating assets");
            return Ok(());
        }
        Some(outdir) => outdir,
    };

    let out_path = PathBuf::from(outdir);
    let mut path = out_path.ancestors().nth(4).unwrap().to_owned();
    path.push("assets");
    std::fs::create_dir_all(&path).unwrap();

    build_shell_completion(&path, &package_meta)?;
    build_manpages(&path, &package_meta)?;

    build_control_file(&path, &package_meta)?;
    build_desktop_file(&path, &package_meta)?;

    Ok(())
}
