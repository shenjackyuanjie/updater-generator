mod changes;

use pyo3::prelude::*;

#[test]
fn diff() {
    use gix_diff::tree::{Recorder, Changes};
    // 从仓库中读取两个树对象
    let mut record = Recorder::track_location();
}
