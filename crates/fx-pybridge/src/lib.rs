use pyo3::prelude::*;
use torch_fx_rs::GraphModule;
use pyo3::types::IntoPyDict;

pub fn graphmodule_from_pickle(py: Python<'_>, path: &str) -> PyResult<Py<GraphModule>> {
    let torch = py.import("torch")?;
    let py_gm = torch.call_method("load", (path,), Some([("weights_only", false)].into_py_dict(py)))?;
    let gm: Py<GraphModule> = py_gm.extract()?;
    Ok(gm)
}

pub fn graphmodule_to_pickle(py: Python<'_>, gm: &GraphModule, path: &str) -> PyResult<()> {
    let torch = py.import("torch")?;
    let py_gm = gm.as_ref();
    torch.call_method("save", (py_gm, path), None)?;
    Ok(())
}