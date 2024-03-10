use clap::CommandFactory;
use clap::ValueEnum;
use clap_complete::generate_to;
use clap_complete::Shell;
use clap_mangen::Man;
use std::{
    env,
    fs::File,
    io::Error,
    path::{Path, PathBuf},
};

include!("src/cli/arguments.rs");

fn build_shell_completion(outdir: &Path) -> Result<(), Error> {
    let mut cmd = Arguments::command();

    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, "http-diff", outdir)?;
    }

    Ok(())
}

fn build_manpages(outdir: &Path) -> Result<(), Error> {
    let app = Arguments::command();

    let file = Path::new(&outdir).join("http-diff.1");
    let mut file = File::create(&file)?;

    Man::new(app).render(&mut file)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/cli/arguments.rs");

    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let out_path = PathBuf::from(outdir);
    let mut path = out_path.ancestors().nth(4).unwrap().to_owned();
    path.push("assets");
    std::fs::create_dir_all(&path).unwrap();

    build_shell_completion(&path)?;
    build_manpages(&path)?;

    Ok(())
}
