use serde;
use serde::Deserialize;
use serde_with::rust::deserialize_ignore_any;
use serde_xml_rs::from_str;
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use clap::Parser;

mod simulink_file;
use simulink_file::*;
use walkdir::WalkDir;

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq)]
enum Mode {
    DumpFile,
    ListBlocks,
    AcceptKnownBlocks,
    // List files containing only _known_ blocks mode
    ListFilesWithKnownBlocks,
    ListUnknownBlocks,
    JustRead,
    CheckBlockParams,
}

#[derive(Parser)]
struct Args {
    #[arg(short, long, group = "input")]
    file: Option<String>,

    #[arg(short, long, group = "input")]
    root_dir: Option<String>,

    #[arg(short, long, default_value = "info")]
    log_level: log::LevelFilter,

    #[arg(short, long, default_value = "dump-file")]
    mode: Mode,
}

fn dump_file(doc: &System) -> Result<(), &'static str> {
    for f in &doc.items {
        match f {
            SysItem::P(p) => {
                println!("Param: {} = {}", p.name, p.value);
            }
            SysItem::Block(b) => {
                println!("Block");
                for i in &b.items {
                    match i {
                        BlockItem::P(p) => {
                            println!("Param: {} = {}", p.name, p.value);
                        }
                        BlockItem::PortCounts(_) => {
                            println!("PortCounts");
                        }
                        BlockItem::InstanceData(_) => {
                            println!("InstanceData");
                        }
                        BlockItem::Mask(_) => {
                            println!("Mask");
                        }
                        BlockItem::PortProperties(_) => {
                            println!("PortProperties");
                        }
                        BlockItem::System(_) => {
                            println!("System");
                        }
                        BlockItem::List(_) => {
                            println!("List");
                        }
                        _ => {
                            println!("BlockOther");
                        }
                    }
                }
            }
            SysItem::Line(l) => {
                println!("Line");
                for i in &l.items {
                    match i {
                        LineItem::P(p) => {
                            println!("Line Param: {} = {}", p.name, p.value);
                        }
                        _ => {
                            println!("LineOther");
                        }
                    }
                }
            }
            SysItem::Other => {
                println!("Other");
            }
            SysItem::Annotation(_) => {
                println!("Annotation");
            }
            _ => {
                println!("Other");
            }
        }
    }
    Ok(())
}

fn sanitize(name: &str) -> String {
    // replace any character outside of ascii 32-127 with a space
    name.chars()
        .map(|c| if c.is_ascii_graphic() { c } else { ' ' })
        .collect()
}

const KNOWN_BLOCKS: [&str; 17] = [
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
];

/*
const KNOWN_BLOCKS: [&str; 46] = [
    "Gain",
    "Constant",
    "InitialCondition",
    "Integrator",
    "Terminator",
    "Reference",
    "Ground",
    "SubSystem",
    "Scope",
    "Sum",
    "Product",
    "Inport",
    "Outport",
    "Mux",
    "BusSelector",
    "Fcn",
    "From",
    "Selector",
    "Demux",
    "Saturate",
    "Display",
    "Goto",
    "TransferFcn",
    "DataTypeConversion",
    "CustomCallbackButton",
    "PMIOPort",
    "BusCreator",
    "S-Function",
    "Lookup_n-D",
    "Switch",
    "ModelReference",
    "Reshape",
    "TriggerPort",
    "Math",
    "RelationalOperator",
    "Math",
    "Sin",
    "Step",
    "Abs",
    "RateTransition",
    "Concatenate",
    "Logic",
    "InportShadow",
    "ActionPort",
    "Signum",
    "MinMax",
];

 */
fn list_blocks(doc: &System, mode: Mode, blocks: &mut HashMap<String, u32>) -> Result<(), String> {
    for f in &doc.items {
        match f {
            SysItem::Block(b) => match mode {
                Mode::ListBlocks => {
                    println!("Block: {} {}", b.block_type, sanitize(&b.name));
                }
                Mode::ListUnknownBlocks => {
                    if !KNOWN_BLOCKS.contains(&b.block_type.as_str()) {
                        println!("Block: {}", b.block_type);
                        *blocks.entry(b.block_type.clone()).or_insert(0) += 1;
                    }
                }
                Mode::AcceptKnownBlocks => {
                    if !KNOWN_BLOCKS.contains(&b.block_type.as_str()) {
                        return Err(format!("Block {} is not known", b.block_type));
                    }
                }
                Mode::ListFilesWithKnownBlocks => {
                    let mut ref_is_known = false;
                    let mut subsystem_is_known = false;
                    if b.block_type == "Reference" {
                        let sb = b
                            .items
                            .iter()
                            .filter_map(|f| match f {
                                BlockItem::P(p) => {
                                    if p.name == "SourceBlock" {
                                        Some(p.value.clone())
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            })
                            .nth(0)
                            .unwrap();
                        ref_is_known = match sanitize(sb.as_str()).as_str() {
                            "simulink/Logic and Bit Operations/Compare To Zero" => true,
                            "simulink/Logic and Bit Operations/Bitwise Operator" => true,
                            "simulink/Logic and Bit Operations/Compare To Constant" => true,
                            "simulink/Sources/Ramp" => true,
                            _ => {
                                log::debug!(
                                    "Reference: {} [ {} ] ",
                                    sanitize(&sb),
                                    sanitize(&b.name)
                                );
                                false
                            }
                        };
                    }
                    if b.block_type == "SubSystem" {
                        let sb = b
                            .items
                            .iter()
                            .filter_map(|f| match f {
                                BlockItem::P(p) => {
                                    if p.name == "OpenFcn" {
                                        Some(p.value.clone())
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            })
                            .nth(0);
                        if let Some(namevalue) = sb {
                            if namevalue.starts_with("showExample(") {
                                subsystem_is_known = true;
                            } else {
                                log::info!(
                                    "SubSystem: {} {}",
                                    sanitize(&namevalue),
                                    sanitize(&b.name)
                                );
                            }
                        }
                    }
                    if KNOWN_BLOCKS.contains(&b.block_type.as_str())
                        || ref_is_known
                        || subsystem_is_known
                    {
                    } else {
                        *blocks.entry(b.block_type.clone()).or_insert(0) += 1;
                        log::debug!("Block {} is not known", b.block_type);
                        return Err(format!("Block {} is not known", b.block_type));
                    }
                }
                Mode::CheckBlockParams => {
                    let visual_params = [
                        "ZOrder",
                        "Position",
                        "BackgroundColor",
                        "ForegroundColor",
                        "BlockMirror",
                        "HideAutomaticName",
                        "ShowName",
                        "FontSize",
                        "NameLocation",
                        "BlockRotation",
                        "FontName",
                        "FontWeight",
                        "DropShadow",
                        "Description",
                        "IconShape",
                        "IconDisplay",
                        "BlockKeywords",
                    ];

                    let known_params: HashMap<&str, &[&str]> = [
                        (
                            "Reference",
                            &[
                                "AttributesFormatString",
                                "LibraryVersion",
                                "Tag",
                                "Priority",
                                "SourceProductName",
                                "UserDataPersistent",
                                "UserData",
                                "DisableCoverage",
                                "IOType",
                                "SourceType",
                                "SourceBlock", // Hey this is funny, needs checking later
                            ][..],
                        ),
                        (
                            "Sum",
                            &[
                                "Ports",
                                "Inputs",
                                "OutDataTypeStr",
                                "AccumDataTypeStr",
                                "AttributesFormatString",
                                "InputSameDT",
                                "SaturateOnIntegerOverflow",
                                "RndMeth",
                                "LockScale",
                                "OutMin",
                                "OutMax",
                                "DisableCoverage",
                                "Tag",
                            ][..],
                        ),
                        (
                            "Constant",
                            &[
                                "OutDataTypeStr",
                                "AttributesFormatString",
                                "SampleTime",
                                "OutMin",
                                "OutMax",
                                "VectorParams1D",
                                "Value",
                                "DisableCoverage",
                                "PreserveConstantTs",
                                "LockScale",
                                "IOType",
                                "DialogController",
                            ][..],
                        ),
                        (
                            "Gain",
                            &[
                                "Gain",
                                "SaturateOnIntegerOverflow",
                                "AttributesFormatString",
                                "LockScale",
                                "Multiplication",
                                "Priority",
                                "ParamDataTypeStr",
                                "OutDataTypeStr",
                                "OutMin",
                                "OutMax",
                                "ParamMin",
                                "ParamMax",
                                "DisableCoverage",
                            ][..],
                        ),
                        (
                            "Product",
                            &[
                                "Inputs",
                                "Multiplication",
                                "InputSameDT",
                                "RndMeth",
                                "OutDataTypeStr",
                                "SaturateOnIntegerOverflow",
                                "CollapseDim",
                                "CollapseMode",
                                "AttributesFormatString",
                                "OutMin",
                                "OutMax",
                                "SampleTime",
                                "DisableCoverage",
                            ][..],
                        ),
                        (
                            "Integrator",
                            &[
                                "InitialCondition",
                                "ExternalReset",
                                "ShowStatePort",
                                "UpperSaturationLimit",
                                "LowerSaturationLimit",
                                "InitialConditionSource",
                                "LimitOutput",
                                "ZeroCross",
                                "ContinuousStateAttributes",
                                "WrapState",
                                "WrappedStateUpperValue",
                                "WrappedStateLowerValue",
                                "ShowSaturationPort",
                                "AttributesFormatString",
                                "DisableCoverage",
                            ][..],
                        ),
                        ("Mux", &["Inputs", "DisplayOption"][..]),
                        (
                            "Demux",
                            &["Tag", "Outputs", "DisplayOption", "DisableCoverage"][..],
                        ),
                        ("InitialCondition", &["Value", "DisableCoverage"]),
                        (
                            "Selector",
                            &[
                                "OutputSizes",
                                "InputPortWidth",
                                "Indices",
                                "IndexOptions",
                                "NumberOfDimensions",
                                "IndexMode",
                            ][..],
                        ),
                        (
                            "Inport",
                            &[
                                "Port",
                                "PortDimensions",
                                "PartitionWidth",
                                "PartitionOffset",
                                "OutDataTypeStr",
                                "Partition",
                                "PartitionDimension",
                                "Unit",
                                "SampleTime",
                                "SignalType",
                                "SamplingMode",
                                "VarSizeSig",
                                "BusOutputAsStruct",
                                "AttributesFormatString",
                                "OutputFunctionCall",
                                "OutputPortMessageModes",
                                "RequirementInfo",
                                "LatchByDelayingOutsideSignal",
                                "Tag",
                                "AllowServiceAccess",
                                "DisableCoverage",
                            ],
                        ),
                        (
                            "Outport",
                            &[
                                "Port",
                                "VectorParamsAs1DForOutWhenUnconnected",
                                "OutDataTypeStr",
                                "InitialOutput",
                                "Unit",
                                "ConcatenationDimension",
                                "PortDimensions",
                                "SignalName",
                                "OutputWhenDisabled",
                                "BusOutputAsStruct",
                                "SignalType",
                                "SamplingMode",
                                "AttributesFormatString",
                                "SampleTime",
                                "InputPortMessageModes",
                                "Tag",
                                "RequirementInfo",
                                "OutMin",
                                "OutMax",
                                "AllowServiceAccess",
                            ],
                        ),
                        (
                            "Saturate",
                            &[
                                "UpperLimit",
                                "LowerLimit",
                                "ZeroCross",
                                "LinearizeAsGain",
                                "Commented",
                                "OutDataTypeStr",
                                "OutMin",
                                "OutMax",
                                "SampleTime",
                                "AttributesFormatString",
                            ][..],
                        ),
                    ]
                    .iter()
                    .cloned()
                    .collect();

                    if let Some(known_block_params) = known_params.get(b.block_type.as_str()) {
                        for bi in &b.items {
                            if let BlockItem::P(p) = bi {
                                if visual_params.contains(&p.name.as_str()) {
                                    continue;
                                }
                                if !known_block_params.contains(&p.name.as_str()) {
                                    println!(
                                        "[{}] Param: {} = {}",
                                        b.block_type,
                                        p.name,
                                        sanitize(&p.value)
                                    );
                                }
                            }
                        }
                    }
                }
                _ => (),
            },
            SysItem::Line(l) => {
                for i in &l.items {
                    match i {
                        LineItem::P(p) => match p.name.as_str() {
                            "ZOrder" => (),
                            "Src" | "Dst" => (),
                            "Points" => (),
                            "Name" | "Labels" => (),
                            "FontName" | "FontSize" | "FontWeight" => (),
                            "Description" => (),
                            "SrcPort" | "DstPort" => (),
                            _ => {
                                println!("Line Param: {} = {}", p.name, p.value);
                            }
                        },
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
    Ok(())
}

fn process_content(
    content: &str,
    file: &str,
    mode: Mode,
    mut blocks: &mut HashMap<String, u32>,
) -> Result<(), String> {
    let doc_result = from_str::<System>(&content);
    if doc_result.is_err() {
        let err = doc_result.err().unwrap();
        println!("Error in file {}: {}", file, err);
        return Err(err.to_string());
    }
    let doc = doc_result.unwrap();

    let res = match mode {
        Mode::DumpFile => Ok(dump_file(&doc)?),
        Mode::ListBlocks
        | Mode::AcceptKnownBlocks
        | Mode::ListFilesWithKnownBlocks
        | Mode::ListUnknownBlocks
        | Mode::CheckBlockParams => list_blocks(&doc, mode, &mut blocks),
        Mode::JustRead => Ok(()),
    };
    if let Err(e) = res {
        if mode != Mode::ListFilesWithKnownBlocks {
            println!("Error in file {}: {}", file, e);
        }
        return Err(e);
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    // Use args.log_level to set the log level
    let log_level = args.log_level;
    env_logger::Builder::new().filter_level(log_level).init();

    log::info!("Starting ");

    let user_profile = std::env::var("USERPROFILE").unwrap_or("".into());
    let mut files: Vec<String> = vec![];
    let mut zip_files: Vec<String> = vec![];

    let matlab = std::env::var("ProgramFiles").unwrap_or("".into());
    let simulink_blocks_dir = matlab + "/MATLAB/R2024b/toolbox/simulink/blocks/library";

    if args.file.is_none() {
        log::info!("Starting dir ");
        let default_dir = user_profile + "/Documents/MATLAB/Examples/R2024b";
        let the_dir = args.root_dir.unwrap_or(default_dir);
        log::debug!("Walking dir: {}", the_dir);
        for entry in WalkDir::new(the_dir) {
            if let Ok(entry) = entry {
                log::debug!("Walking dir: {}", entry.path().to_string_lossy());
                if entry
                    .path()
                    .to_str()
                    .map_or(false, |s| s.contains("systems"))
                    && entry.path().extension() == Some(std::ffi::OsStr::new("xml"))
                {
                    files.push(entry.path().to_string_lossy().to_string());
                }
                if entry.path().extension() == Some(std::ffi::OsStr::new("slx")) {
                    zip_files.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    } else {
        let file = args.file.unwrap_or(r"system_root.xml".to_string());
        files.push(file);
    }

    let mut blocks: HashMap<String, u32> = HashMap::new();

    for zip_file in &zip_files {
        log::debug!("Processing zip file: {}", zip_file);
        let mut zip = zip::ZipArchive::new(std::fs::File::open(zip_file).unwrap()).unwrap();
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            // only if file path contains /simulink/systems/ and ends with .xml
            if file.name().contains("simulink/systems/") && file.name().ends_with(".xml") {
                log::debug!("Reading zip file: {}", file.name());

                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                let tmp = zip_file.clone() + "/" + file.name();
                if args.mode == Mode::ListFilesWithKnownBlocks {
                    if file.name().ends_with("/system_root.xml") {
                        let res = process_content(&content, tmp.as_str(), args.mode, &mut blocks);
                        if res.is_ok() {
                            println!("{}/{}", zip_file, file.name());
                        }
                    }
                } else {
                    process_content(&content, tmp.as_str(), args.mode, &mut blocks)?;
                }
            } else {
                log::debug!("Skipping file: {}", file.name());
                //println!("Skipping file: {}", file.name());
            }
        }
    }
    for file in files {
        let content = std::fs::read_to_string(&file).unwrap();
        process_content(&content, &file, args.mode, &mut blocks)?;
    }
    match args.mode {
        Mode::ListUnknownBlocks | Mode::ListFilesWithKnownBlocks => {
            println!("Unknown blocks:");
            let mut blocks_vec: Vec<(&String, &u32)> = blocks.iter().collect();
            blocks_vec.sort_by(|a, b| b.1.cmp(a.1));
            for (block, count) in blocks_vec {
                println!("{}: {}", block, count);
            }
        }
        _ => (),
    }
    Ok(())
}
