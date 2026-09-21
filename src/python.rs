use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;

use crate::InverseNormalizer;

#[pyclass(name = "InverseNormalizer", module = "ukrainian_itn._rust", frozen)]
struct PyInverseNormalizer {
    inner: InverseNormalizer,
}

#[pymethods]
impl PyInverseNormalizer {
    #[new]
    #[pyo3(signature = (tagger_path=None, verbalizer_path=None))]
    fn new(tagger_path: Option<&str>, verbalizer_path: Option<&str>) -> PyResult<Self> {
        let normalizer = match (tagger_path, verbalizer_path) {
            (None, None) => InverseNormalizer::new(),
            (Some(tagger_path), Some(verbalizer_path)) => {
                InverseNormalizer::from_files(tagger_path, verbalizer_path)
            }
            _ => Err(anyhow::anyhow!(
                "tagger_path and verbalizer_path must be provided together"
            )),
        };
        normalizer
            .map(|inner| Self { inner })
            .map_err(|error| PyOSError::new_err(format!("{error:#}")))
    }

    #[pyo3(signature = (text, json=false))]
    fn normalize(&self, py: Python<'_>, text: &str, json: bool) -> PyResult<String> {
        py.detach(|| {
            if json {
                self.inner.normalize_json(text)
            } else {
                self.inner.normalize(text)
            }
        })
        .map_err(|error| PyValueError::new_err(format!("{error:#}")))
    }

    fn normalize_or_passthrough(&self, py: Python<'_>, text: &str) -> String {
        py.detach(|| self.inner.normalize_or_passthrough(text))
    }

    fn __repr__(&self) -> &'static str {
        "InverseNormalizer()"
    }
}

#[pymodule]
#[pyo3(name = "_rust")]
fn python_module(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyInverseNormalizer>()?;
    Ok(())
}
