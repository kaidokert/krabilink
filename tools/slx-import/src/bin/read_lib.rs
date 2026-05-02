use clap::Parser;
mod simulink_file;
use simulink_file::*;
use std::collections::HashMap;
use std::io::Read;
use walkdir::WalkDir;

use serde;

#[derive(Parser)]
struct Args {
    #[arg(short, long, group = "input")]
    file: Option<String>,

    #[arg(short, long, group = "input")]
    root_dir: Option<String>,

    #[arg(short, long, default_value = "info")]
    log_level: log::LevelFilter,
}

fn sanitize(name: &str) -> String {
    // replace any character outside of ascii 32-127 with a space
    name.chars()
        .map(|c| if c.is_ascii_graphic() { c } else { ' ' })
        .collect()
}

struct SubSystem {
    name: String,
    system: System,
}
impl SubSystem {
    fn new(name: String, system: System) -> Self {
        Self { name, system }
    }
}

struct CompleteModel {
    systems: HashMap<String, SubSystem>,
}
impl CompleteModel {
    fn new() -> Self {
        Self {
            systems: HashMap::new(),
        }
    }
}

const KNOWN_BLOCKS: [&str; 20] = [
    "Constant",
    "Gain",
    "Sum",
    "InitialCondition",
    "Scope",
    "Display",
    "DashboardScope",
    "Step",
    "Sin",
    "Integrator",
    "Abs",
    "Product",
    "Sqrt",
    "MinMax",
    "Ground",
    "RelationalOperator",
    "Terminator",
    "Inport",
    "Outport",
    "Saturate",
];

fn load_library(path_prefix: String, slx_file_path: String) -> Option<CompleteModel> {
    let mut model = CompleteModel::new();
    log::debug!("file: {}", slx_file_path);
    let mut zip = zip::ZipArchive::new(
        std::fs::File::open(&slx_file_path).expect(&format!("Failed to open file: {}", slx_file_path)),
    )
    .expect(&format!("Failed to read ZIP archive: {}", slx_file_path));
    let sysref = {
        let mut file = zip
            .by_name("simulink/blockdiagram.xml")
            .expect("ZIP missing simulink/blockdiagram.xml");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Failed to read blockdiagram.xml from ZIP");
        let model_info: Result<ModelInformation, serde_xml_rs::Error> =
            serde_xml_rs::from_str(&content);
        if model_info.is_err() {
            log::error!("Failed to parse model info: {}", model_info.err().unwrap());
            return None;
        }
        let model_info = model_info.unwrap();
        let sysref = model_info.Library.items.iter().find_map(|item| match item {
            LibraryItem::System(sysref) => Some(sysref.sysref.clone()),
            _ => None,
        });
        let model_sysref = model_info.Model.items.iter().find_map(|item| match item {
            LibraryItem::System(sysref) => Some(sysref.sysref.clone()),
            _ => None,
        });
        log::info!(
            "prefix: path_prefix {}, library_sysref: {:?} model_sysref: {:?}",
            path_prefix,
            sysref,
            model_sysref
        );
        if let Some(inner_sysref) = sysref {
            inner_sysref
        } else {
            if model_sysref.is_some() {
                model_sysref.expect("model_sysref was None after is_some check")
            } else {
                log::error!("This isn't a model or a library file");
                return None;
            }
        }
    };

    let mut sysrefs = vec![(sysref, path_prefix)];

    while let Some((sysref, path)) = sysrefs.pop() {
        let sys_path = format!("simulink/systems/{}.xml", sysref);
        let mut file = zip
            .by_name(&sys_path)
            .expect(&format!("ZIP missing {}", sys_path));
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect(&format!("Failed to read {} from ZIP", sys_path));
        let sys = serde_xml_rs::from_str::<System>(&content)
            .expect(&format!("Failed to parse XML in {}", sys_path));
        let subsys = SubSystem::new(sysref, sys);

        for item in subsys.system.items.iter().filter(|item| match item {
            SysItem::Block(sysref) => sysref.block_type == "SubSystem",
            _ => false,
        }) {
            match item {
                SysItem::Block(sysref) => {
                    log::debug!("item: {}/{:?}", sanitize(&path), sanitize(&sysref.name));
                    for item in sysref.items.iter() {
                        match item {
                            BlockItem::System(sysref_inner) => {
                                //log::info!("sysref: {:?}", sysref_inner.sys_ref);
                                sysrefs.push((
                                    sysref_inner.sys_ref.clone(),
                                    path.to_string() + "/" + &sysref.name,
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        // Move subsystem to model
        model.systems.insert(path, subsys);
    }
    Some(model)
}

fn check_for_known_blocks(
    filename: &str,
    model: &CompleteModel,
    reference_models: &HashMap<String, CompleteModel>,
) {
    // find root system from model, it's one where system.name equals system_root
    let root_system = model.systems.iter().find_map(|(key, sys)| {
        if sys.name == "system_root" {
            Some(key)
        } else {
            None
        }
    });
    log::info!("root_system: {:?}", root_system);

    let mut all_blocks_good = true;

    // loop over all blocks in all systems
    for (key, sys) in model.systems.iter() {
        // log::info!("item: {:?} {}", key , sys.name);
        for item in sys.system.items.iter() {
            match item {
                SysItem::Block(sysref) => {
                    match sysref.block_type.as_str() {
                        "SubSystem" => {
                            for items in sysref.items.iter() {
                                match items {
                                    BlockItem::System(sysref_inner) => {
                                        let submodel_name = sysref_inner.sys_ref.clone();
                                        log::debug!("system: submodel {}", submodel_name);
                                        // Now look this up in the passed submodels within CompleteModel
                                        let submodel = model
                                            .systems
                                            .iter()
                                            .filter(|(key, sys)| sys.name == sysref_inner.sys_ref)
                                            .nth(0);
                                        if let Some((subname, subsys)) = submodel {
                                            log::info!("resolved submodel: {:?}", subname);
                                        } else {
                                            all_blocks_good = false;
                                            log::error!("submodel not found {}", submodel_name);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        "Reference" => {
                            for items in sysref.items.iter() {
                                match items {
                                    BlockItem::P(ref_p) => {
                                        if ref_p.name == "SourceBlock" {
                                            log::debug!(
                                                "p reference:  {} = {}",
                                                ref_p.name,
                                                ref_p.value
                                            );
                                            // First part before slash is the ref_model key
                                            let ref_model_key = ref_p
                                                .value
                                                .split('/')
                                                .nth(0)
                                                .expect(&format!("No '/' in SourceBlock value: {}", ref_p.value));
                                            // we need to find ref_p.value in reference_models
                                            let ref_model =
                                                reference_models.iter().find_map(|(key, model)| {
                                                    log::debug!(
                                                        "key: [{} ] : [ {} ] ",
                                                        key,
                                                        ref_model_key
                                                    );
                                                    if key == ref_model_key {
                                                        Some(model)
                                                    } else {
                                                        None
                                                    }
                                                });
                                            if let Some(ref_m) = ref_model {
                                                for (key, sys) in ref_m.systems.iter() {
                                                    //log::info!("ref_model: {} [ {} ] ", key, sys.name);
                                                    if key == &ref_p.value {
                                                        log::info!(
                                                            "found ref_model: {} [ {} ] ",
                                                            key,
                                                            sys.name
                                                        );
                                                        match sanitize(&key).as_str() {
                                                            "simulink/Logic and Bit Operations/Compare To Zero" => (),
                                                            "simulink/Logic and Bit Operations/Bitwise Operator" => (),
                                                            "simulink/Logic and Bit Operations/Compare To Constant" => (),
                                                            "simulink/Sources/Ramp" => (),
                                                            _ => {
                                                                all_blocks_good = false;
                                                                log::error!("A ref we don't like {} ", key);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {
                            if !KNOWN_BLOCKS.contains(&sysref.block_type.as_str()) {
                                all_blocks_good = false;
                                log::error!("A block we don't like: {}", sysref.block_type);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if all_blocks_good {
        log::info!("All blocks are good: {}", filename);
    } else {
        log::error!("Some blocks are not good: {} ", filename);
    }
}

fn load_library_model(filename: String) -> CompleteModel {
    let matlab_root = std::env::var("MATLAB_ROOT").unwrap_or_else(|_| {
        let pf = std::env::var("ProgramFiles").unwrap_or_default();
        pf + "/MATLAB/R2024b"
    });
    let simulink_blocks_dir = matlab_root + "/toolbox/simulink/blocks/library";
    let slx_file_path = simulink_blocks_dir.clone() + "/" + &filename + ".slx";
    let simulink_model = load_library(filename.to_string(), slx_file_path);
    simulink_model.expect(&format!("Failed to load library model: {}", filename))
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let log_level = args.log_level;
    env_logger::Builder::new().filter_level(log_level).init();
    log::info!("Starting ");

    let mut models = HashMap::new();

    models.insert(
        "simulink".to_string(),
        load_library_model("simulink".to_string()),
    );
    models.insert(
        "simulink_extras".into(),
        load_library_model("simulink_extras".to_string()),
    );

    let mut files: Vec<String> = vec![];

    if args.file.is_none() {
        log::info!("Starting dir ");
        let default_dir =
            std::env::var("USERPROFILE").unwrap_or("".into()) + "/Documents/MATLAB/Examples/R2024b";
        let the_dir = args.root_dir.unwrap_or(default_dir);
        log::debug!("Walking dir: {}", the_dir);
        for entry in WalkDir::new(the_dir) {
            if let Ok(entry) = entry {
                if entry.path().extension() == Some(std::ffi::OsStr::new("slx")) {
                    log::debug!("Walking dir: {}", entry.path().to_string_lossy());
                    files.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    } else {
        let file = args.file.unwrap();
        files.push(file);
    }

    for file in files {
        // figure out basename for file
        let basename = file.split('/').last().expect(&format!("No basename in path: {}", file));
        // remove .slx
        let basename = basename.split('.').nth(0).expect(&format!("No extension in: {}", basename));
        let model = load_library(basename.to_string(), file.clone());
        if let Some(model) = model {
            check_for_known_blocks(&file, &model, &models);
        } else {
            log::error!("Failed to load model: {}", basename.to_string());
        }
    }
    Ok(())
}
