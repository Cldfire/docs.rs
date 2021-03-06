use git2::Repository;
use std::{env, error::Error, fs::File, io::Write, path::Path};

fn main() {
    // Don't rerun anytime a single change is made
    println!("cargo:rerun-if-changed=templates/style/vendored.scss");
    println!("cargo:rerun-if-changed=templates/style/base.scss");
    println!("cargo:rerun-if-changed=templates/style/_rustdoc.scss");
    println!("cargo:rerun-if-changed=templates/style/_vars.scss");
    println!("cargo:rerun-if-changed=templates/style/_utils.scss");
    println!("cargo:rerun-if-changed=templates/style/_navbar.scss");
    println!("cargo:rerun-if-changed=templates/style/_themes.scss");
    println!("cargo:rerun-if-changed=vendor/");
    // TODO: are these right?
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");

    write_git_version();
    if let Err(sass_err) = compile_sass() {
        panic!("Error compiling sass: {}", sass_err);
    }
}

fn write_git_version() {
    let maybe_hash = get_git_hash();
    let git_hash = maybe_hash.as_deref().unwrap_or("???????");

    let build_date = time::strftime("%Y-%m-%d", &time::now_utc()).unwrap();
    let dest_path = Path::new(&env::var("OUT_DIR").unwrap()).join("git_version");

    let mut file = File::create(&dest_path).unwrap();
    write!(file, "({} {})", git_hash, build_date).unwrap();
}

fn get_git_hash() -> Option<String> {
    let repo = Repository::open(env::current_dir().unwrap()).ok()?;
    let head = repo.head().unwrap();

    head.target().map(|h| {
        let mut h = format!("{}", h);
        h.truncate(7);
        h
    })
}

fn compile_sass_file(
    name: &str,
    target: &str,
    include_paths: &[String],
) -> Result<(), Box<dyn Error>> {
    use sass_rs::{Context, Options, OutputStyle};

    const STYLE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/templates/style");

    let include_paths = {
        let mut paths = vec![STYLE_DIR.to_owned()];
        paths.extend_from_slice(include_paths);
        paths
    };

    // Compile base.scss
    let mut context = Context::new_file(format!("{}/{}.scss", STYLE_DIR, name))?;
    context.set_options(Options {
        output_style: OutputStyle::Compressed,
        include_paths,
        ..Default::default()
    });

    let css = context.compile()?;
    let dest_path = Path::new(&env::var("OUT_DIR")?).join(format!("{}.css", target));
    let mut file = File::create(&dest_path)?;
    file.write_all(css.as_bytes())?;

    Ok(())
}

fn compile_sass() -> Result<(), Box<dyn Error>> {
    // Compile base.scss -> style.css
    compile_sass_file("base", "style", &[])?;

    // Compile vendored.scss -> vendored.css
    compile_sass_file(
        "vendored",
        "vendored",
        &[concat!(env!("CARGO_MANIFEST_DIR"), "/vendor/pure-css/css").to_owned()],
    )?;

    Ok(())
}
