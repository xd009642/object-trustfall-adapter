use glob::glob;
use object_trustfall_adapter::loader::load_object;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventLog {
    events: Vec<EventWrapper>,
    manifest_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TestBinary {
    path: PathBuf,
    // Cutting out what I don't care about.
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    ConfigLaunch(String),
    BinaryLaunch(TestBinary),
    Trace(serde_json::Value),
    Marker(Option<()>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventWrapper {
    #[serde(flatten)]
    event: Event,
    // The time this was created in seconds
    created: f64,
}

/// Stores all the program traces mapped to files and provides an interface to
/// add, query and change traces.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TraceMap {
    /// Traces in the program mapped to the given file
    traces: BTreeMap<PathBuf, Vec<Trace>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Trace {
    /// Line the trace is on in the file
    pub line: u64,
    /// Optional address showing location in the test artefact
    pub address: Vec<u64>,
    /// Length of the instruction (useful to get entire condition/branch)
    pub length: usize,
}

/// So we have a bit of an "interesting" approach to testing the project here. cargo-tarpaulin will
/// for a project output a json file in the `target/tarpaulin` directory which is a serialization
/// of it's `TraceMap` format. This has lines of code mapped to addresses via the debug
/// information. So I'm going to run tarpaulin on the project, there are no tests in any of the
/// projects so the part where tarpaulin runs the tests should be "fast". Then I grab the
/// executables, the tracemap and get to work comparing it with the adapters outputs.
///
/// Assumptions:
///
/// 1. Only one test executable - more than one might mess up addresses
fn check_addresses_match(path: &Path) {
    let traces_json = path.join("*.json").display().to_string();
    let delta_json = path.join("target/tarpaulin/*.json").display().to_string();
    // We need to clean up after ourselves!
    for path in glob(&traces_json)
        .unwrap()
        .chain(glob(&delta_json).unwrap())
        .filter_map(|x| x.ok())
    {
        fs::remove_file(&path).unwrap();
    }

    Command::new("cargo")
        .args([
            "tarpaulin",
            "--engine",
            "ptrace",
            "--dump-traces",
            "--skip-clean",
        ])
        .current_dir(path)
        .output()
        .unwrap();

    // Okay now in the root there's a traces json with the executables. There's also a json in the
    // target directory!

    let traces_json = glob(&traces_json).unwrap().next().unwrap().unwrap();
    let delta_json = glob(&delta_json).unwrap().next().unwrap().unwrap();

    // Now we should load these jsons and deserialize them (or can we just trustfall query to
    // implement our tests?
    let traces = fs::read(traces_json).unwrap();
    let traces: EventLog = serde_json::from_slice(&traces).unwrap();

    let mut to_analyse = None;
    for event in traces.events.iter() {
        if let Event::BinaryLaunch(bin) = &event.event {
            to_analyse = Some(bin.path.clone());
            break;
        }
    }
    let to_analyse = to_analyse.unwrap();

    println!("Loading: {:?}", to_analyse);
    let object = load_object(&to_analyse).unwrap();

    let file = fs::File::create(path.join("object.json")).unwrap();
    let mut writer = io::BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &object).unwrap();
}

#[test]
fn hello_world_conformance() {
    let hello_world = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/projects/hello-world");
    check_addresses_match(&hello_world);
}
