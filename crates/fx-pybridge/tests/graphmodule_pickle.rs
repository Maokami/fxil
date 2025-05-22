use pyo3::PyResult;
use pyo3::Python;
use pyo3::types::{PyAny, PyTuple, PyModule};
use fx_pybridge::{graphmodule_from_pickle, graphmodule_to_pickle};
use std::fs;
use std::process::Command;

const ORIG_PICKLE: &str = "tests/data/test_module.pt";
const ROUNDTRIP_PICKLE: &str = "tests/data/test_module_roundtrip.pt";
const GEN_SCRIPT: &str = "tests/scripts/gen_fx_module.py";

fn setup_test_data() {
    let status = Command::new("python3")
        .arg(GEN_SCRIPT)
        .status()
        .expect("Failed to run gen_fx_module.py");
    assert!(status.success());
}

fn cleanup_test_data() {
    let _ = fs::remove_file(ORIG_PICKLE);
    let _ = fs::remove_file(ROUNDTRIP_PICKLE);
}

fn graphmodules_are_semantically_equal(py: Python, gm1: &PyAny, gm2: &PyAny) -> PyResult<bool> {
    let code1 = gm1.getattr("code")?.extract::<String>()?;
    let code2 = gm2.getattr("code")?.extract::<String>()?;

    if code1 != code2 {
        println!("Graph codes differ.");
        // fs::write("tests/data/gm1_code.py", &code1).expect("Failed to write gm1_code.py");
        // fs::write("tests/data/gm2_code.py", &code2).expect("Failed to write gm2_code.py");
        // println!("--- Code for GM1:\n{}\n--- Code for GM2:\n{}", code1, code2);
        return Ok(false);
    }

    let get_tabular_script = r#"
import io
import sys

def get_tabular_string(graph_module_or_graph):
    graph_to_print = getattr(graph_module_or_graph, 'graph', graph_module_or_graph)

    old_stdout = sys.stdout
    sys.stdout = captured_output = io.StringIO()
    graph_to_print.print_tabular()
    sys.stdout = old_stdout
    return captured_output.getvalue()
"#;
    let py_get_tabular_func = PyModule::from_code(py, get_tabular_script, "", "")?
        .getattr("get_tabular_string")?;

    let tabular1_args = PyTuple::new(py, &[gm1]);
    let tabular1 = py_get_tabular_func.call1(tabular1_args)?.extract::<String>()?;

    let tabular2_args = PyTuple::new(py, &[gm2]);
    let tabular2 = py_get_tabular_func.call1(tabular2_args)?.extract::<String>()?;

    if tabular1 != tabular2 {
        println!("Graph tabular representations differ.");
        // fs::write("tests/data/gm1_tabular.txt", &tabular1).expect("Failed to write gm1_tabular.txt");
        // fs::write("tests/data/gm2_tabular.txt", &tabular2).expect("Failed to write gm2_tabular.txt");
        // println!("--- Tabular for GM1:\n{}\n--- Tabular for GM2:\n{}", tabular1, tabular2);
        return Ok(false);
    }

    Ok(true)
}

fn ensure_python_path(py: Python) {
    if let Some(site_packages_path) = option_env!("FX_PYBRIDGE_TEST_SITE_PACKAGES") {
        if !site_packages_path.is_empty() {
            let sys = py.import("sys").expect("Failed to import sys");
            let mut path: Vec<String> = sys.getattr("path")
                .expect("Failed to get sys.path")
                .extract()
                .expect("Failed to extract sys.path as Vec<String>");

            if !path.iter().any(|p| p == site_packages_path) {
                // println!(">>> Adding to sys.path via build.rs: {}", site_packages_path);
                path.insert(0, site_packages_path.to_string());
                sys.setattr("path", path).expect("Failed to set sys.path");
            }
        }
    }
}

#[test]
fn test_graphmodule_pickle_roundtrip() {
    setup_test_data();

    let comparison_success = Python::with_gil(|py| {
        // Ensure the Python path includes the site-packages directory
        ensure_python_path(py);

        let gm_orig = graphmodule_from_pickle(py, ORIG_PICKLE)
            .expect("Failed to load GraphModule from ORIG_PICKLE");

        graphmodule_to_pickle(py, gm_orig.as_ref(py), ROUNDTRIP_PICKLE)
            .expect("Failed to save GraphModule to ROUNDTRIP_PICKLE");

        let gm_roundtripped = graphmodule_from_pickle(py, ROUNDTRIP_PICKLE)
            .expect("Failed to load GraphModule from ROUNDTRIP_PICKLE");

        graphmodules_are_semantically_equal(py, gm_orig.as_ref(py), gm_roundtripped.as_ref(py))
            .expect("Error during GraphModule comparison")
    });

    assert!(fs::metadata(ORIG_PICKLE).is_ok());
    assert!(fs::metadata(ROUNDTRIP_PICKLE).is_ok());
    assert!(comparison_success, "GraphModules differ semantically after roundtrip");
}