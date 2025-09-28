use std::{env, fs};
use std::path::PathBuf;
use std::process::Command;
use anyhow::Error;
use typebinder::contexts::type_solving::TypeSolvingContextBuilder;
use typebinder::error::TsExportError;
use typebinder::exporters::file::FileExporter;
use typebinder::macros::context::MacroSolvingContext;
use typebinder::path_mapper::PathMapper;
use typebinder::pipeline::Pipeline;
use typebinder::step_spawner::mod_reader::RustModuleReader;
use regex::Regex;

pub fn main() -> Result<(), Error> {
    return Ok(());
    println!("cargo:rerun-if-changed=src/schema");

    let solving_context = TypeSolvingContextBuilder::default()
        .add_default_solvers()
        .finish();

    let macro_context = MacroSolvingContext::default();

    // Generate schema types
    let rust_module = fs::canonicalize("src/schema/mod.rs")?;
    let schema_dir = "../site-react/src/schema";
    fs::create_dir_all(schema_dir)?;
    let schema_dir = fs::canonicalize(schema_dir)?;

    // Clean out the schemas
    let _ = fs::remove_dir_all(&schema_dir);

    if let Err(error) = (Pipeline {
        pipeline_step_spawner: RustModuleReader::try_new(rust_module)
            .map_err(|err| Error::msg(format!("Error: {err}")))?,
        exporter: FileExporter::new(schema_dir),
        path_mapper: PathMapper::default(),
    }
        .launch(&solving_context, &macro_context))
    {
        match error {
            TsExportError::MalformedInput | TsExportError::SynError(_) => {
                // The Rust compiler itself will catch these and throw them better.
            }
            error => {
                panic!("Error during type export: {error}");
            }
        }
    }

    fix_type_paths(PathBuf::from("../site-react/src/schema"))?;

    if !run_command(&["pnpm", "check"])? {
        panic!("Typechecking website failed. Run pnpm check manually to see more info.");
    }

    Ok(())
}

fn fix_type_paths(path: PathBuf) -> Result<(), Error> {
    if fs::metadata(&path)?.is_dir() {
        for file in fs::read_dir(&path)? {
            fix_type_paths(file?.path())?;
        }
    } else {
        let content = fs::read_to_string(&path)?;
        // Replace all backend::schema and subpaths with @/schema
        let re = Regex::new(r#"backend::schema([a-zA-Z0-9_:]*)"#)?;
        let replaced = re.replace_all(&content, |caps: &regex::Captures| {
            if &caps[1] == "" {
                "@/schema".to_string()
            } else {
                format!("@/schema{}", &caps[1].replace("::", "/"))
            }
        });
        fs::write(&path, replaced
            .replace("from \"uuid\";", "from \"@/lib/env\";")
        )?;
    }
    Ok(())
}

fn run_command(args: &[&str]) -> Result<bool, Error> {
        let output = if cfg!(target_os = "windows") {
            let mut command = Command::new("cmd");
            command
                .args(&["/C", &format!("{}", args.join(" "))]);
            command
        } else {
            let mut command = Command::new(args[0]);
            command.args(&args[1..]);
            command
        }
            .current_dir("../site-react")
            .output()?;

        if env::var("BUILD_DEBUG").is_ok() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                println!("cargo:warning={}", line);
            }
            for line in String::from_utf8_lossy(&output.stderr).lines() {
                println!("cargo:warning={}", line);
            }
        }
        Ok(output.status.success())
    }